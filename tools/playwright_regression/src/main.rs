use anyhow::{Context, Result, bail, ensure};
use playwright_rs::api::LaunchOptions;
use playwright_rs::{Browser, PLAYWRIGHT_VERSION, Page, Playwright};
use serde::Deserialize;
use std::env;
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const DEFAULT_DIST_DIR: &str = "example_motion/dist";
const DEFAULT_PORT: u16 = 4173;
const REQUEST_BUFFER_LEN: usize = 16 * 1024;

const MOTION_MID_WAIT: Duration = Duration::from_millis(180);
const MOTION_SETTLE_WAIT: Duration = Duration::from_millis(1400);

const FLIP_MID_WAIT: Duration = Duration::from_millis(260);
const FLIP_SETTLE_WAIT: Duration = Duration::from_millis(1500);

const FLIP_HERO_MID_WAIT: Duration = Duration::from_millis(160);
const FLIP_HERO_SETTLE_WAIT: Duration = Duration::from_millis(1200);

const FLIP_GROUP_MID_WAIT: Duration = Duration::from_millis(180);
const FLIP_GROUP_SETTLE_WAIT: Duration = Duration::from_millis(1100);

#[derive(Debug, Clone)]
struct Config {
    dist_dir: PathBuf,
    port: u16,
    headed: bool,
}

impl Config {
    fn from_args() -> Result<Self> {
        let mut config = Self {
            dist_dir: PathBuf::from(DEFAULT_DIST_DIR),
            port: DEFAULT_PORT,
            headed: false,
        };

        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--dist-dir" => {
                    let value = args.next().context("Expected a path after --dist-dir")?;
                    config.dist_dir = PathBuf::from(value);
                }
                "--port" => {
                    let value = args.next().context("Expected a value after --port")?;
                    config.port = value
                        .parse::<u16>()
                        .with_context(|| format!("Invalid --port value: {value}"))?;
                }
                "--headed" => {
                    config.headed = true;
                }
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                other => {
                    bail!("Unknown argument: {other}\n\nRun with --help for usage.");
                }
            }
        }

        let index_html = config.dist_dir.join("index.html");
        ensure!(
            index_html.exists(),
            "Could not find {}. Build the demo first (for example: `cd example_motion && trunk build`).",
            index_html.display()
        );

        Ok(config)
    }
}

struct StaticServer {
    addr: SocketAddr,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl StaticServer {
    fn start(root: PathBuf, port: u16) -> Result<Self> {
        let canonical_root = root
            .canonicalize()
            .with_context(|| format!("Failed to resolve dist directory: {}", root.display()))?;
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
        let listener = TcpListener::bind(addr)
            .with_context(|| format!("Failed to bind static server on {addr}"))?;
        listener
            .set_nonblocking(true)
            .context("Failed to set non-blocking mode on static server")?;

        let stop = Arc::new(AtomicBool::new(false));
        let stop_for_thread = stop.clone();

        let handle = thread::Builder::new()
            .name("playwright-regression-server".to_string())
            .spawn(move || serve_static(listener, canonical_root, stop_for_thread))
            .context("Failed to start static server thread")?;

        Ok(Self {
            addr,
            stop,
            handle: Some(handle),
        })
    }

    fn base_url(&self) -> String {
        format!("http://{}/", self.addr)
    }
}

impl Drop for StaticServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = TcpStream::connect(self.addr);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn serve_static(listener: TcpListener, root: PathBuf, stop: Arc<AtomicBool>) {
    while !stop.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((mut stream, _peer)) => {
                if let Err(error) = stream.set_nonblocking(false) {
                    eprintln!(
                        "[playwright_regression] static server failed to set stream blocking mode: {error}"
                    );
                    continue;
                }
                if let Err(error) = handle_connection(&mut stream, &root) {
                    eprintln!("[playwright_regression] static server request error: {error:#}");
                }
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(error) => {
                eprintln!("[playwright_regression] static server failed: {error}");
                break;
            }
        }
    }
}

fn handle_connection(stream: &mut TcpStream, root: &Path) -> Result<()> {
    let mut request_buffer = [0u8; REQUEST_BUFFER_LEN];
    let read = stream
        .read(&mut request_buffer)
        .context("Failed to read HTTP request")?;
    if read == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&request_buffer[..read]);
    let first_line = request.lines().next().unwrap_or_default();
    let mut parts = first_line.split_whitespace();

    let method = parts.next().unwrap_or_default();
    let raw_path = parts.next().unwrap_or("/");

    if method != "GET" && method != "HEAD" {
        write_response(
            stream,
            "405 Method Not Allowed",
            "text/plain; charset=utf-8",
            b"Only GET/HEAD are supported",
            method == "HEAD",
        )?;
        return Ok(());
    }

    let path = raw_path.split('?').next().unwrap_or("/");
    let Some(file_path) = resolve_request_path(root, path) else {
        write_response(
            stream,
            "404 Not Found",
            "text/plain; charset=utf-8",
            b"Not Found",
            method == "HEAD",
        )?;
        return Ok(());
    };

    let body = fs::read(&file_path)
        .with_context(|| format!("Failed to read static asset: {}", file_path.display()))?;
    write_response(
        stream,
        "200 OK",
        content_type_for(&file_path),
        &body,
        method == "HEAD",
    )?;

    Ok(())
}

fn resolve_request_path(root: &Path, request_path: &str) -> Option<PathBuf> {
    let relative = request_path.trim_start_matches('/');
    let relative = if relative.is_empty() {
        PathBuf::from("index.html")
    } else {
        PathBuf::from(relative)
    };

    let candidate = root.join(relative);
    let candidate = if candidate.is_dir() {
        candidate.join("index.html")
    } else {
        candidate
    };

    let canonical = candidate.canonicalize().ok()?;
    if !canonical.starts_with(root) {
        return None;
    }
    Some(canonical)
}

fn content_type_for(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("wasm") => "application/wasm",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
}

fn write_response(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
    head_only: bool,
) -> Result<()> {
    let header = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    );

    stream
        .write_all(header.as_bytes())
        .context("Failed to write response headers")?;
    if !head_only {
        stream
            .write_all(body)
            .context("Failed to write response body")?;
    }
    stream.flush().context("Failed to flush response")?;
    Ok(())
}

#[derive(Debug, Deserialize)]
struct StyleSnapshot {
    transform: String,
    opacity: f64,
}

#[derive(Debug, Deserialize)]
struct RectSnapshot {
    left: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_args()?;
    let server = StaticServer::start(config.dist_dir.clone(), config.port)?;
    println!(
        "[playwright_regression] Serving {} at {}",
        config.dist_dir.display(),
        server.base_url()
    );

    let playwright = Playwright::launch()
        .await
        .with_context(|| format!(
            "Failed to launch Playwright driver. If this is a fresh machine, install browsers with: npx playwright@{PLAYWRIGHT_VERSION} install chromium"
        ))?;

    let browser = playwright
        .chromium()
        .launch_with_options(LaunchOptions::new().headless(!config.headed))
        .await
        .with_context(|| format!(
            "Failed to launch Chromium. Install browsers with: npx playwright@{PLAYWRIGHT_VERSION} install chromium"
        ))?;

    run_regression_suite(&browser, &server.base_url()).await?;

    browser.close().await?;
    println!("[playwright_regression] All animation regression checks passed");
    Ok(())
}

async fn run_regression_suite(browser: &Browser, base_url: &str) -> Result<()> {
    run_motion_state_transition_check(browser, base_url).await?;
    run_single_flip_check(browser, base_url).await?;
    run_flip_hero_border_radius_check(browser, base_url).await?;
    run_flip_group_check(browser, base_url).await?;
    Ok(())
}

async fn open_demo_page(browser: &Browser, base_url: &str) -> Result<Page> {
    let page = browser.new_page().await?;

    let response = page
        .goto(base_url, None)
        .await?
        .context("Navigation returned no response")?;

    ensure!(
        response.ok(),
        "Failed to load demo page: status {}",
        response.status()
    );

    if let Err(error) = wait_for_visible(
        &page,
        "[data-testid='hero-toggle']",
        Duration::from_secs(12),
    )
    .await
    {
        let has_wasm_bindings = page
            .evaluate::<(), bool>("() => Boolean(window.wasmBindings)", None::<&()>)
            .await
            .unwrap_or(false);
        let ready_state = page
            .evaluate::<(), String>("() => document.readyState", None::<&()>)
            .await
            .unwrap_or_else(|_| "unknown".to_string());
        let body_html = page
            .evaluate::<(), String>(
                "() => document.body ? document.body.innerHTML : ''",
                None::<&()>,
            )
            .await
            .unwrap_or_default();
        let snippet = body_html.chars().take(240).collect::<String>();
        bail!(
            "Demo did not render expected controls: {error}. ready_state={ready_state}, has_wasm_bindings={has_wasm_bindings}, body_html_snippet={snippet:?}"
        );
    }

    Ok(page)
}

async fn wait_for_visible(page: &Page, selector: &str, timeout: Duration) -> Result<()> {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        let locator = page.locator(selector).await;
        match locator.is_visible().await {
            Ok(true) => return Ok(()),
            Ok(false) | Err(_) => {}
        }
        if tokio::time::Instant::now() >= deadline {
            bail!("Timed out waiting for visible selector: {selector}");
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn run_motion_state_transition_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression] Running motion transition check");
    let page = open_demo_page(browser, base_url).await?;

    let selector = "[data-testid='hero-section'] .glass";
    let start = read_style(&page, selector).await?;

    page.locator("[data-testid='hero-toggle']")
        .await
        .click(None)
        .await?;

    tokio::time::sleep(MOTION_MID_WAIT).await;
    let mid = read_style(&page, selector).await?;

    tokio::time::sleep(MOTION_SETTLE_WAIT).await;
    let end = read_style(&page, selector).await?;

    ensure!(
        style_delta(&start, &end) > 0.01,
        "Hero motion target did not change after toggle"
    );
    ensure!(
        style_delta(&start, &mid) > 0.01,
        "Hero motion never moved away from start state"
    );
    ensure!(
        style_delta(&mid, &end) > 0.01,
        "Hero motion jumped directly to end state (no observable interpolation)"
    );

    page.close().await?;
    Ok(())
}

async fn run_single_flip_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression] Running single FLIP transition check");
    let page = open_demo_page(browser, base_url).await?;

    let start_rect = read_rect(&page, "#flip-pill").await?;

    page.locator("[data-testid='flip-move-right']")
        .await
        .click(None)
        .await?;

    tokio::time::sleep(FLIP_MID_WAIT).await;
    let mid_style = read_style(&page, "#flip-pill").await?;
    ensure!(
        !is_identity_transform(&mid_style.transform),
        "FLIP single-element animation never entered an in-flight transform state"
    );

    tokio::time::sleep(FLIP_SETTLE_WAIT).await;
    let end_rect = read_rect(&page, "#flip-pill").await?;
    let end_style = read_style(&page, "#flip-pill").await?;

    ensure!(
        end_rect.left > start_rect.left + 60.0,
        "FLIP single-element layout position did not move far enough: start_left={:.2}, end_left={:.2}",
        start_rect.left,
        end_rect.left
    );

    ensure!(
        is_identity_transform(&end_style.transform),
        "FLIP single-element transform did not settle back to identity after animation"
    );

    page.close().await?;
    Ok(())
}

async fn run_flip_hero_border_radius_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression] Running FLIP hero border-radius check");
    let page = open_demo_page(browser, base_url).await?;

    let start_radius = read_visual_border_radius(&page, "#flip-hero-image").await?;

    page.locator("#flip-hero-image").await.click(None).await?;

    tokio::time::sleep(FLIP_HERO_MID_WAIT).await;
    let mid_radius = read_visual_border_radius(&page, "#flip-hero-image").await?;

    tokio::time::sleep(FLIP_HERO_SETTLE_WAIT).await;
    let end_radius = read_visual_border_radius(&page, "#flip-hero-image").await?;

    ensure!(
        end_radius > start_radius + 1.0,
        "Hero FLIP border radius did not move toward open-state value: start={start_radius:.2}, end={end_radius:.2}"
    );
    ensure!(
        mid_radius > start_radius + 0.05,
        "Hero FLIP border radius did not progress after animation started: start={start_radius:.2}, mid={mid_radius:.2}"
    );
    ensure!(
        end_radius - mid_radius > 0.2,
        "Hero FLIP border radius appears to jump too quickly to final value: mid={mid_radius:.2}, end={end_radius:.2}"
    );

    page.close().await?;
    Ok(())
}

async fn run_flip_group_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression] Running FLIP group reorder check");
    let page = open_demo_page(browser, base_url).await?;

    let start_order = read_css_order(&page, "#mix-a").await?;

    page.locator("[data-testid='flip-group-rotate']")
        .await
        .click(None)
        .await?;

    tokio::time::sleep(FLIP_GROUP_MID_WAIT).await;

    let status_text = page
        .locator("[data-testid='flip-group-status']")
        .await
        .inner_text()
        .await?;
    ensure!(
        status_text.contains("Animating group"),
        "FLIP group status did not report active animation"
    );

    let mid_style = read_style(&page, "#mix-a").await?;
    ensure!(
        !is_identity_transform(&mid_style.transform),
        "FLIP group animation did not produce an in-flight transform"
    );

    tokio::time::sleep(FLIP_GROUP_SETTLE_WAIT).await;
    let end_order = read_css_order(&page, "#mix-a").await?;

    ensure!(
        end_order != start_order,
        "FLIP group reorder did not change CSS order for #mix-a ({} -> {})",
        start_order,
        end_order
    );

    page.close().await?;
    Ok(())
}

async fn read_style(page: &Page, selector: &str) -> Result<StyleSnapshot> {
    let selector = selector.to_string();
    page.evaluate(
        r#"(selector) => {
            const element = document.querySelector(selector);
            if (!element) {
                throw new Error(`Missing element: ${selector}`);
            }
            const style = window.getComputedStyle(element);
            return {
                transform: style.transform,
                opacity: Number.parseFloat(style.opacity || "1")
            };
        }"#,
        Some(&selector),
    )
    .await
    .with_context(|| format!("Failed to read style for selector `{selector}`"))
}

async fn read_rect(page: &Page, selector: &str) -> Result<RectSnapshot> {
    let selector = selector.to_string();
    page.evaluate(
        r#"(selector) => {
            const element = document.querySelector(selector);
            if (!element) {
                throw new Error(`Missing element: ${selector}`);
            }
            const rect = element.getBoundingClientRect();
            return {
                left: rect.left
            };
        }"#,
        Some(&selector),
    )
    .await
    .with_context(|| format!("Failed to read rect for selector `{selector}`"))
}

async fn read_css_order(page: &Page, selector: &str) -> Result<i32> {
    let selector = selector.to_string();
    page.evaluate(
        r#"(selector) => {
            const element = document.querySelector(selector);
            if (!element) {
                throw new Error(`Missing element: ${selector}`);
            }
            const orderValue = window.getComputedStyle(element).order;
            return Number.parseInt(orderValue, 10);
        }"#,
        Some(&selector),
    )
    .await
    .with_context(|| format!("Failed to read CSS order for selector `{selector}`"))
}

async fn read_visual_border_radius(page: &Page, selector: &str) -> Result<f64> {
    let selector = selector.to_string();
    page.evaluate(
        r#"(selector) => {
            const element = document.querySelector(selector);
            if (!element) {
                throw new Error(`Missing element: ${selector}`);
            }
            const style = window.getComputedStyle(element);
            const radius = Number.parseFloat(style.borderTopLeftRadius || "0");

            const transform = style.transform || "none";
            let scaleX = 1.0;
            if (transform !== "none") {
                if (transform.startsWith("matrix3d(")) {
                    const raw = transform.slice("matrix3d(".length, -1).split(",");
                    scaleX = Number.parseFloat(raw[0] || "1");
                } else if (transform.startsWith("matrix(")) {
                    const raw = transform.slice("matrix(".length, -1).split(",");
                    scaleX = Number.parseFloat(raw[0] || "1");
                }
            }

            return radius * Math.abs(scaleX || 1.0);
        }"#,
        Some(&selector),
    )
    .await
    .with_context(|| format!("Failed to read visual border radius for selector `{selector}`"))
}

fn style_delta(a: &StyleSnapshot, b: &StyleSnapshot) -> f64 {
    let transform_changed = normalize_transform(&a.transform) != normalize_transform(&b.transform);
    let transform_score = if transform_changed { 1.0 } else { 0.0 };
    transform_score + (a.opacity - b.opacity).abs()
}

fn normalize_transform(value: &str) -> String {
    value.split_whitespace().collect::<String>()
}

fn is_identity_transform(value: &str) -> bool {
    let normalized = normalize_transform(value);
    if normalized.is_empty() || normalized == "none" {
        return true;
    }

    normalized == "matrix(1,0,0,1,0,0)" || normalized == "matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1)"
}

fn print_help() {
    println!(
        "playwright_regression\n\n\
Runs animation regression checks against the built example_motion app.\n\n\
USAGE:\n\
  cargo run -p playwright_regression -- [OPTIONS]\n\n\
OPTIONS:\n\
  --dist-dir <PATH>   Directory containing built static files (default: {DEFAULT_DIST_DIR})\n\
  --port <PORT>       Local HTTP port used by the static test server (default: {DEFAULT_PORT})\n\
  --headed            Run Chromium in headed mode\n\
  -h, --help          Show this help\n\n\
PREREQUISITES:\n\
  1) Build the demo: `cd example_motion && trunk build`\n\
  2) Install browsers: `npx playwright@{PLAYWRIGHT_VERSION} install chromium`"
    );
}
