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

const BIND_MID_WAIT: Duration = Duration::from_millis(180);
const BIND_SETTLE_WAIT: Duration = Duration::from_millis(1300);

const POINTER_STEP_WAIT: Duration = Duration::from_millis(220);

const QUEUE_SETTLE_WAIT: Duration = Duration::from_millis(1200);

const TABS_MID_WAIT: Duration = Duration::from_millis(180);
const TABS_SETTLE_WAIT: Duration = Duration::from_millis(2200);

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
}

#[derive(Debug, Deserialize)]
struct RectSnapshot {
    left: f64,
    width: f64,
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
    run_bind_transition_check(browser, base_url).await?;
    run_tabs_underline_check(browser, base_url).await?;
    run_pointer_state_check(browser, base_url).await?;
    run_queue_latest_check(browser, base_url).await?;
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
        "[data-testid='controller-bind-toggle']",
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
            bail!("Timed out waiting for visible selector: {selector}");
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn run_bind_transition_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression_controller] Running bind transition check");
    let page = open_demo_page(browser, base_url).await?;

    let selector = "[data-testid='controller-bind-card']";
    let start = read_style(&page, selector).await?;

    page.locator("[data-testid='controller-bind-toggle']")
        .await
        .click(None)
        .await?;

    tokio::time::sleep(BIND_MID_WAIT).await;
    let mid = read_style(&page, selector).await?;

    tokio::time::sleep(BIND_SETTLE_WAIT).await;
    let end = read_style(&page, selector).await?;

    ensure!(
        style_delta(&start, &end) > 0.05,
        "Bind transition did not reach a different settled state"
    );
    ensure!(
        style_delta(&start, &mid) > 0.05,
        "Bind transition never moved away from the starting style"
    );
    ensure!(
        style_delta(&mid, &end) > 0.01,
        "Bind transition appears to jump directly to the end"
    );
    ensure!(
        end.opacity > start.opacity,
        "Expected expanded state to have higher opacity: start={:.3}, end={:.3}",
        start.opacity,
        end.opacity
    );

    page.close().await?;
    Ok(())
}

async fn run_tabs_underline_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression_controller] Running tabs underline check");
    let page = open_demo_page(browser, base_url).await?;

    wait_for_visible(
        &page,
        "[data-testid='controller-tab-button-0']",
        Duration::from_secs(8),
    )
    .await?;
    wait_for_visible(
        &page,
        "[data-testid='controller-tab-underline']",
        Duration::from_secs(8),
    )
    .await?;

    tokio::time::sleep(Duration::from_millis(140)).await;

    let start_underline = read_rect(&page, "[data-testid='controller-tab-underline']").await?;
    let tab0_rect = read_rect(&page, "[data-testid='controller-tab-button-0']").await?;
    ensure!(
        approx_eq(rect_center(&start_underline), rect_center(&tab0_rect), 12.0),
        "Underline did not initialize near first tab: underline_left={:.2}, tab_left={:.2}",
        start_underline.left,
        tab0_rect.left
    );

    page.locator("[data-testid='controller-tab-button-3']")
        .await
        .click(None)
        .await?;

    tokio::time::sleep(TABS_MID_WAIT).await;
    let mid_underline = read_rect(&page, "[data-testid='controller-tab-underline']").await?;

    tokio::time::sleep(TABS_SETTLE_WAIT).await;
    let end_underline = read_rect(&page, "[data-testid='controller-tab-underline']").await?;
    let tab3_rect = read_rect(&page, "[data-testid='controller-tab-button-3']").await?;

    ensure!(
        rect_delta(&start_underline, &mid_underline) > 4.0,
        "Underline never moved away from initial tab"
    );
    ensure!(
        rect_delta(&mid_underline, &end_underline) > 1.0,
        "Underline appears to jump straight to final tab"
    );
    ensure!(
        approx_eq(rect_center(&end_underline), rect_center(&tab3_rect), 20.0),
        "Underline did not settle near tab 3 center: underline={:.2}, tab={:.2}",
        rect_center(&end_underline),
        rect_center(&tab3_rect)
    );
    ensure!(
        approx_eq(end_underline.width, tab3_rect.width, 24.0),
        "Underline width did not settle near tab 3 width: underline={:.2}, tab={:.2}",
        end_underline.width,
        tab3_rect.width
    );

    page.locator("[data-testid='controller-tab-button-1']")
        .await
        .click(None)
        .await?;
    page.locator("[data-testid='controller-tab-button-2']")
        .await
        .click(None)
        .await?;

    tokio::time::sleep(TABS_SETTLE_WAIT).await;
    let final_underline = read_rect(&page, "[data-testid='controller-tab-underline']").await?;
    let tab2_rect = read_rect(&page, "[data-testid='controller-tab-button-2']").await?;
    let tab_content = read_text(&page, "[data-testid='controller-tab-content']").await?;

    ensure!(
        approx_eq(rect_center(&final_underline), rect_center(&tab2_rect), 20.0),
        "Rapid retarget did not settle on last clicked tab center: underline={:.2}, tab={:.2}",
        rect_center(&final_underline),
        rect_center(&tab2_rect)
    );
    ensure!(
        approx_eq(final_underline.width, tab2_rect.width, 24.0),
        "Rapid retarget did not settle on last clicked tab width: underline={:.2}, tab={:.2}",
        final_underline.width,
        tab2_rect.width
    );
    ensure!(
        tab_content.to_lowercase().contains("retargeting"),
        "Tab content did not update to final selected tab: {tab_content:?}"
    );

    page.close().await?;
    Ok(())
}

async fn run_pointer_state_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression_controller] Running pointer state check");
    let page = open_demo_page(browser, base_url).await?;

    let pill_selector = "[data-testid='controller-pointer-pill']";
    let idle_style = read_style(&page, pill_selector).await?;

    page.locator("[data-testid='controller-pointer-arm-toggle']")
        .await
        .click(None)
        .await?;
    tokio::time::sleep(POINTER_STEP_WAIT).await;
    let armed_style = read_style(&page, pill_selector).await?;

    ensure!(
        normalize_color(&idle_style.background_color)
            != normalize_color(&armed_style.background_color),
        "Pointer arm toggle did not change base visual state"
    );

    dispatch_pointer_event(&page, pill_selector, "pointerenter").await?;
    tokio::time::sleep(POINTER_STEP_WAIT).await;
    let hover_style = read_style(&page, pill_selector).await?;
    ensure!(
        !is_identity_transform(&hover_style.transform),
        "Hover state did not enter a transformed style"
    );

    dispatch_pointer_event(&page, pill_selector, "pointerdown").await?;
    tokio::time::sleep(POINTER_STEP_WAIT).await;
    let pressed_style = read_style(&page, pill_selector).await?;
    ensure!(
        normalize_transform(&pressed_style.transform)
            != normalize_transform(&hover_style.transform),
        "Press state did not diverge from hover style"
    );

    dispatch_pointer_event(&page, pill_selector, "pointerup").await?;
    tokio::time::sleep(POINTER_STEP_WAIT).await;
    let released_style = read_style(&page, pill_selector).await?;
    ensure!(
        !is_identity_transform(&released_style.transform),
        "Release while hovered should return to hover style, not base"
    );

    dispatch_pointer_event(&page, pill_selector, "pointerleave").await?;
    tokio::time::sleep(POINTER_STEP_WAIT).await;
    let settled_style = read_style(&page, pill_selector).await?;
    ensure!(
        is_identity_transform(&settled_style.transform),
        "Pointer leave should settle back to base transform"
    );
    ensure!(
        normalize_color(&settled_style.background_color)
            == normalize_color(&armed_style.background_color),
        "Pointer leave should keep the armed base background"
    );

    page.close().await?;
    Ok(())
}

async fn run_queue_latest_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression_controller] Running queue-latest check");
    let page = open_demo_page(browser, base_url).await?;

    wait_for_visible(
        &page,
        "[data-testid='controller-queue-detached']",
        Duration::from_secs(8),
    )
    .await?;

    for _ in 0..3 {
        page.locator("[data-testid='controller-queue-next']")
            .await
            .click(None)
            .await?;
    }

    page.locator("[data-testid='controller-queue-mount']")
        .await
        .click(None)
        .await?;
    wait_for_visible(
        &page,
        "[data-testid='controller-queue-chip']",
        Duration::from_secs(6),
    )
    .await?;
    tokio::time::sleep(QUEUE_SETTLE_WAIT).await;

    let first_label = read_text(&page, "[data-testid='controller-queue-label']").await?;
    let first_style = read_style(&page, "[data-testid='controller-queue-chip']").await?;

    ensure!(
        first_label.contains("flare"),
        "Expected queued step label to settle at flare, got: {first_label:?}"
    );
    ensure!(
        approx_eq(first_style.opacity, 0.92, 0.08),
        "Expected first mounted opacity near 0.92, got {:.3}",
        first_style.opacity
    );

    page.locator("[data-testid='controller-queue-mount']")
        .await
        .click(None)
        .await?;
    wait_for_visible(
        &page,
        "[data-testid='controller-queue-detached']",
        Duration::from_secs(6),
    )
    .await?;

    for _ in 0..2 {
        page.locator("[data-testid='controller-queue-next']")
            .await
            .click(None)
            .await?;
    }

    page.locator("[data-testid='controller-queue-mount']")
        .await
        .click(None)
        .await?;
    wait_for_visible(
        &page,
        "[data-testid='controller-queue-chip']",
        Duration::from_secs(6),
    )
    .await?;
    tokio::time::sleep(QUEUE_SETTLE_WAIT).await;

    let second_label = read_text(&page, "[data-testid='controller-queue-label']").await?;
    let second_style = read_style(&page, "[data-testid='controller-queue-chip']").await?;

    ensure!(
        second_label.contains("anchor"),
        "Expected queued step label to settle at anchor, got: {second_label:?}"
    );
    ensure!(
        approx_eq(second_style.opacity, 1.0, 0.08),
        "Expected second mounted opacity near 1.0, got {:.3}",
        second_style.opacity
    );
    ensure!(
        (first_style.opacity - second_style.opacity).abs() > 0.04,
        "Queue replay did not produce distinct settled states across mounts"
    );

    page.close().await?;
    Ok(())
}

async fn dispatch_pointer_event(page: &Page, selector: &str, event_type: &str) -> Result<()> {
    let payload = [selector, event_type];
    page.evaluate::<[&str; 2], ()>(
        r#"([selector, eventType]) => {
            const element = document.querySelector(selector);
            if (!element) {
                throw new Error(`Missing element: ${selector}`);
            }
            const event = new PointerEvent(eventType, {
                bubbles: true,
                cancelable: true,
                composed: true,
                pointerId: 1,
                isPrimary: true,
                pointerType: "mouse"
            });
            element.dispatchEvent(event);
        }"#,
        Some(&payload),
    )
    .await
    .with_context(|| format!("Failed to dispatch pointer event `{event_type}` for `{selector}`"))
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
                opacity: Number.parseFloat(style.opacity || "1"),
                background_color: style.backgroundColor || ""
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
                left: rect.left,
                width: rect.width
            };
        }"#,
        Some(&selector),
    )
    .await
    .with_context(|| format!("Failed to read rect for selector `{selector}`"))
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

fn rect_delta(a: &RectSnapshot, b: &RectSnapshot) -> f64 {
    (a.left - b.left).abs() + (a.width - b.width).abs()
}

fn rect_center(rect: &RectSnapshot) -> f64 {
    rect.left + (rect.width / 2.0)
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

fn is_identity_transform(value: &str) -> bool {
    let normalized = normalize_transform(value);
    if normalized.is_empty() || normalized == "none" {
        return true;
    }

    normalized == "matrix(1,0,0,1,0,0)" || normalized == "matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1)"
}

fn approx_eq(value: f64, expected: f64, tolerance: f64) -> bool {
    (value - expected).abs() <= tolerance
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
