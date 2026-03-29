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

const DEFAULT_DIST_DIR: &str = "example_motion_controller/dist";
const DEFAULT_PORT: u16 = 4174;
const REQUEST_BUFFER_LEN: usize = 16 * 1024;

const BUILDER_CARD_MID_WAIT: Duration = Duration::from_millis(180);
const BUILDER_CARD_SETTLE_WAIT: Duration = Duration::from_millis(1300);

const MACRO_STATE_SETTLE_WAIT: Duration = Duration::from_millis(1400);

const RESOLVER_MID_WAIT: Duration = Duration::from_millis(220);
const RESOLVER_SETTLE_WAIT: Duration = Duration::from_millis(1800);

const TIMELINE_MID_WAIT: Duration = Duration::from_millis(260);
const TIMELINE_PAUSE_WAIT: Duration = Duration::from_millis(560);
const TIMELINE_RESUME_WAIT: Duration = Duration::from_millis(520);

const AUTO_SIZE_SETTLE_WAIT: Duration = Duration::from_millis(900);

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
            "Could not find {}. Build the demo first (for example: `cd example_motion_controller && trunk build`).",
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
            .name("playwright-regression-controller-server".to_string())
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
                        "[playwright_regression_controller] static server failed to set stream blocking mode: {error}"
                    );
                    continue;
                }
                if let Err(error) = handle_connection(&mut stream, &root) {
                    eprintln!(
                        "[playwright_regression_controller] static server request error: {error:#}"
                    );
                }
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(error) => {
                eprintln!("[playwright_regression_controller] static server failed: {error}");
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
    background_color: String,
    width: f64,
    height: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_args()?;
    let server = StaticServer::start(config.dist_dir.clone(), config.port)?;
    println!(
        "[playwright_regression_controller] Serving {} at {}",
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
    println!("[playwright_regression_controller] All regression checks passed");
    Ok(())
}

async fn run_regression_suite(browser: &Browser, base_url: &str) -> Result<()> {
    run_builder_card_check(browser, base_url).await?;
    run_macro_state_check(browser, base_url).await?;
    run_resolver_deck_check(browser, base_url).await?;
    run_timeline_builder_check(browser, base_url).await?;
    run_timeline_macro_check(browser, base_url).await?;
    run_auto_size_check(browser, base_url).await?;
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

    wait_for_visible(
        &page,
        "[data-testid='builder-card-toggle']",
        Duration::from_secs(12),
    )
    .await?;

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
            let body = page.content().await.unwrap_or_default();
            bail!("Timed out waiting for visible selector: {selector}\n\nPage content:\n{body}");
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn run_builder_card_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression_controller] Running builder card check");
    let page = open_demo_page(browser, base_url).await?;

    let selector = "[data-testid='builder-card-preview']";
    let start = read_style(&page, selector).await?;

    page.locator("[data-testid='builder-card-toggle']")
        .await
        .click(None)
        .await?;

    tokio::time::sleep(BUILDER_CARD_MID_WAIT).await;
    let mid = read_style(&page, selector).await?;

    tokio::time::sleep(BUILDER_CARD_SETTLE_WAIT).await;
    let end = read_style(&page, selector).await?;
    let status = read_text(&page, "[data-testid='builder-card-status']").await?;

    ensure!(
        style_delta(&start, &end) > 0.05,
        "Builder card did not reach a different settled state"
    );
    ensure!(
        style_delta(&start, &mid) > 0.05,
        "Builder card never moved away from the starting style"
    );
    ensure!(
        style_delta(&mid, &end) > 0.01,
        "Builder card appears to jump directly to the end"
    );
    ensure!(
        status.to_lowercase().contains("lifted"),
        "Builder card status did not reflect the lifted state: {status:?}"
    );

    page.close().await?;
    Ok(())
}

async fn run_macro_state_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression_controller] Running macro state check");
    let page = open_demo_page(browser, base_url).await?;

    let selector = "[data-testid='macro-state-preview']";
    let start = read_style(&page, selector).await?;

    page.locator("[data-testid='macro-state-review']")
        .await
        .click(None)
        .await?;
    page.locator("[data-testid='macro-state-live']")
        .await
        .click(None)
        .await?;
    tokio::time::sleep(MACRO_STATE_SETTLE_WAIT).await;

    let end = read_style(&page, selector).await?;
    let status = read_text(&page, "[data-testid='macro-state-status']").await?;

    ensure!(
        style_delta(&start, &end) > 0.05,
        "Macro state preview did not reach a different settled state"
    );
    ensure!(
        status.to_lowercase().contains("live"),
        "Macro state status did not settle on live mode: {status:?}"
    );

    page.close().await?;
    Ok(())
}

async fn run_resolver_deck_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression_controller] Running resolver deck check");
    let page = open_demo_page(browser, base_url).await?;

    let first_selector = "[data-testid='resolver-card-0']";
    let second_selector = "[data-testid='resolver-card-1']";

    let first_start = read_style(&page, first_selector).await?;
    let second_start = read_style(&page, second_selector).await?;

    page.locator("[data-testid='resolver-pulse']")
        .await
        .click(None)
        .await?;
    tokio::time::sleep(RESOLVER_MID_WAIT).await;
    let first_mid = read_style(&page, first_selector).await?;

    ensure!(
        style_delta(&first_start, &first_mid) > 0.08,
        "Active resolver card never moved away from its starting state"
    );

    page.locator("[data-testid='resolver-next']")
        .await
        .click(None)
        .await?;
    tokio::time::sleep(RESOLVER_SETTLE_WAIT).await;

    let first_end = read_style(&page, first_selector).await?;
    let second_end = read_style(&page, second_selector).await?;

    ensure!(
        style_delta(&second_start, &second_end) > 0.12,
        "Replacement resolver card never animated after retargeting"
    );
    ensure!(
        second_end.opacity > first_end.opacity + 0.04,
        "Resolver retargeting did not leave the new card more energized than the old one: first={:.3}, second={:.3}",
        first_end.opacity,
        second_end.opacity
    );

    page.close().await?;
    Ok(())
}

async fn run_timeline_builder_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression_controller] Running timeline builder check");
    let page = open_demo_page(browser, base_url).await?;

    let selector = "[data-testid='timeline-builder-glyph']";
    let start = read_style(&page, selector).await?;

    page.locator("[data-testid='timeline-builder-restart']")
        .await
        .click(None)
        .await?;
    tokio::time::sleep(TIMELINE_MID_WAIT).await;
    let mid = read_style(&page, selector).await?;

    ensure!(
        style_delta(&start, &mid) > 0.08,
        "Timeline builder glyph never left its starting state"
    );

    page.locator("[data-testid='timeline-builder-pause']")
        .await
        .click(None)
        .await?;
    tokio::time::sleep(Duration::from_millis(180)).await;
    let paused = read_style(&page, selector).await?;
    let paused_status = read_text(&page, "[data-testid='timeline-builder-status']").await?;

    ensure!(
        paused_status.to_lowercase().contains("paused"),
        "Timeline builder status did not report a paused state: {paused_status:?}"
    );

    page.locator("[data-testid='timeline-builder-pause']")
        .await
        .click(None)
        .await?;
    tokio::time::sleep(TIMELINE_RESUME_WAIT).await;
    let resumed = read_style(&page, selector).await?;

    ensure!(
        style_delta(&paused, &resumed) > 0.04,
        "Timeline builder glyph did not resume after unpausing"
    );

    page.close().await?;
    Ok(())
}

async fn run_timeline_macro_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression_controller] Running timeline macro check");
    let page = open_demo_page(browser, base_url).await?;

    let selector = "[data-testid='timeline-macro-glyph']";
    let start = read_style(&page, selector).await?;

    page.locator("[data-testid='timeline-macro-toggle']")
        .await
        .click(None)
        .await?;

    tokio::time::sleep(TIMELINE_MID_WAIT).await;
    let mid = read_style(&page, selector).await?;
    ensure!(
        style_delta(&start, &mid) > 0.08,
        "Timeline macro glyph never left its starting state"
    );

    page.locator("[data-testid='timeline-macro-pause']")
        .await
        .click(None)
        .await?;
    tokio::time::sleep(Duration::from_millis(180)).await;
    let paused = read_style(&page, selector).await?;
    let paused_status = read_text(&page, "[data-testid='timeline-macro-status']").await?;

    ensure!(
        paused_status.to_lowercase().contains("paused"),
        "Timeline macro status did not report a paused state: {paused_status:?}"
    );

    page.locator("[data-testid='timeline-macro-pause']")
        .await
        .click(None)
        .await?;
    tokio::time::sleep(TIMELINE_RESUME_WAIT).await;
    let resumed = read_style(&page, selector).await?;

    ensure!(
        style_delta(&paused, &resumed) > 0.04,
        "Timeline macro glyph did not resume after unpausing"
    );

    page.locator("[data-testid='timeline-macro-toggle']")
        .await
        .click(None)
        .await?;
    tokio::time::sleep(TIMELINE_PAUSE_WAIT).await;
    let stopped = read_style(&page, selector).await?;
    let stopped_status = read_text(&page, "[data-testid='timeline-macro-status']").await?;

    ensure!(
        style_delta(&start, &stopped) < 0.08,
        "Stopping the macro timeline did not reset the glyph close to its resting state"
    );
    ensure!(
        stopped_status.to_lowercase().contains("stopped"),
        "Timeline macro status did not report a stopped state: {stopped_status:?}"
    );

    page.close().await?;
    Ok(())
}

async fn run_auto_size_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression_controller] Running auto size check");
    let page = open_demo_page(browser, base_url).await?;

    let height_selector = "[data-testid='auto-height-shell']";
    let width_selector = "[data-testid='auto-width-shell']";

    let height_start = read_style(&page, height_selector).await?;
    page.locator("[data-testid='auto-height-next']")
        .await
        .click(None)
        .await?;
    page.locator("[data-testid='auto-height-next']")
        .await
        .click(None)
        .await?;
    tokio::time::sleep(AUTO_SIZE_SETTLE_WAIT).await;
    let height_end = read_style(&page, height_selector).await?;

    ensure!(
        height_end.height > height_start.height + 24.0,
        "Auto height shell did not grow as expected: start={:.2}, end={:.2}",
        height_start.height,
        height_end.height
    );

    let width_start = read_style(&page, width_selector).await?;
    page.locator("[data-testid='auto-width-next']")
        .await
        .click(None)
        .await?;
    tokio::time::sleep(AUTO_SIZE_SETTLE_WAIT).await;
    let width_end = read_style(&page, width_selector).await?;
    let width_label = read_text(&page, "[data-testid='auto-width-label']").await?;

    ensure!(
        width_end.width > width_start.width + 40.0,
        "Auto width shell did not grow as expected: start={:.2}, end={:.2}",
        width_start.width,
        width_end.width
    );
    ensure!(
        width_label.to_lowercase().contains("prep"),
        "Auto width label did not advance to the next state: {width_label:?}"
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
            const rect = element.getBoundingClientRect();
            return {
                transform: style.transform,
                opacity: Number.parseFloat(style.opacity || "1"),
                background_color: style.backgroundColor || "",
                width: rect.width,
                height: rect.height
            };
        }"#,
        Some(&selector),
    )
    .await
    .with_context(|| format!("Failed to read style for selector `{selector}`"))
}

async fn read_text(page: &Page, selector: &str) -> Result<String> {
    let locator = page.locator(selector).await;
    locator
        .inner_text()
        .await
        .with_context(|| format!("Failed to read text for selector `{selector}`"))
}

fn style_delta(a: &StyleSnapshot, b: &StyleSnapshot) -> f64 {
    let transform_changed = normalize_transform(&a.transform) != normalize_transform(&b.transform);
    let transform_score = if transform_changed { 1.0 } else { 0.0 };
    let color_changed =
        if normalize_color(&a.background_color) != normalize_color(&b.background_color) {
            0.35
        } else {
            0.0
        };

    transform_score + color_changed + (a.opacity - b.opacity).abs()
}

fn normalize_transform(value: &str) -> String {
    value.split_whitespace().collect::<String>()
}

fn normalize_color(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .to_lowercase()
}

fn print_help() {
    println!(
        "playwright_regression_controller\n\n\
Runs controller-only animation regression checks against the built example_motion_controller app.\n\n\
USAGE:\n\
  cargo run -p playwright_regression_controller -- [OPTIONS]\n\n\
OPTIONS:\n\
  --dist-dir <PATH>   Directory containing built static files (default: {DEFAULT_DIST_DIR})\n\
  --port <PORT>       Local HTTP port used by the static test server (default: {DEFAULT_PORT})\n\
  --headed            Run Chromium in headed mode\n\
  -h, --help          Show this help\n\n\
PREREQUISITES:\n\
  1) Build the demo: `cd example_motion_controller && trunk build`\n\
  2) Install browsers: `npx playwright@{PLAYWRIGHT_VERSION} install chromium`"
    );
}
