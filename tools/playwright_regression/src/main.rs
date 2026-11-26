use anyhow::{Context, Result, bail, ensure};
use playwright_rs::api::LaunchOptions;
use playwright_rs::{Browser, PLAYWRIGHT_VERSION, Page, Playwright};
use serde::{Deserialize, Serialize};
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

// Sample offsets in ms after the click, measured in-page (see CLICK_AND_SAMPLE_JS)
// so CDP round-trips can't push the mid sample past the end of a short animation.
// `HeroSection`: 260ms controller transition.
const MOTION_SAMPLE_DELAYS_MS: [u32; 2] = [90, 800];
// `FlipCardSection`: one rAF to measure, then a 260ms FLIP.
const FLIP_SAMPLE_DELAYS_MS: [u32; 2] = [90, 800];
// `FlipBoardSection`: 240ms FLIP plus 20ms stagger across 6 tiles (~340ms).
const FLIP_GROUP_SAMPLE_DELAYS_MS: [u32; 2] = [90, 900];

/// Clicks the unique button whose trimmed text is `button`, then samples every
/// element matching `selector` once before the click and once per `delays`
/// entry (ms since the click).
///
/// Mid-animation samples run in a rAF scheduled from a task, so they land after
/// the library's own per-frame rAF work (FLIP border-radius / scale correction)
/// for that frame and read the same animation time it used.
const CLICK_AND_SAMPLE_JS: &str = r#"async ({ button, selector, delays }) => {
    const parseScale = (transform) => {
        if (!transform || transform === "none") {
            return [1, 1];
        }
        const values = transform
            .slice(transform.indexOf("(") + 1, -1)
            .split(",")
            .map((value) => Number.parseFloat(value));
        return transform.startsWith("matrix3d(") ? [values[0], values[5]] : [values[0], values[3]];
    };
    const sample = () => {
        const elements = [...document.querySelectorAll(selector)];
        if (elements.length === 0) {
            throw new Error(`Missing element: ${selector}`);
        }
        return elements.map((element) => {
            const style = window.getComputedStyle(element);
            const rect = element.getBoundingClientRect();
            const radii = (style.borderTopLeftRadius || "0px")
                .trim()
                .split(/\s+/)
                .map((value) => Number.parseFloat(value));
            const [scaleX, scaleY] = parseScale(style.transform);
            return {
                id: element.id || "",
                transform: style.transform,
                opacity: Number.parseFloat(style.opacity || "1"),
                left: rect.left,
                width: rect.width,
                order: Number.parseInt(style.order, 10) || 0,
                radius_x: radii[0],
                radius_y: radii.length > 1 ? radii[1] : radii[0],
                scale_x: scaleX,
                scale_y: scaleY
            };
        });
    };
    const nextFrameAfterTask = async () => {
        await new Promise((resolve) => setTimeout(resolve, 0));
        await new Promise((resolve) => requestAnimationFrame(resolve));
    };

    const matches = [...document.querySelectorAll("button")].filter(
        (element) => element.textContent.trim() === button
    );
    if (matches.length !== 1) {
        throw new Error(`Expected one button labelled "${button}", found ${matches.length}`);
    }

    const frames = [sample()];
    matches[0].click();
    const clickedAt = performance.now();
    for (const delay of delays) {
        do {
            await nextFrameAfterTask();
        } while (performance.now() - clickedAt < delay);
        frames.push(sample());
    }
    return frames;
}"#;

/// Resolves once `selector` has settled at `opacity` with no running animation,
/// polling per frame until `timeoutMs`.
const WAIT_FOR_OPACITY_JS: &str = r#"async ({ selector, opacity, timeoutMs }) => {
    const deadline = performance.now() + timeoutMs;
    let current = NaN;
    for (;;) {
        const element = document.querySelector(selector);
        if (element) {
            current = Number.parseFloat(window.getComputedStyle(element).opacity || "1");
            const running = element.getAnimations().some((animation) => animation.playState === "running");
            if (!running && Math.abs(current - opacity) < 0.002) {
                return true;
            }
        }
        if (performance.now() > deadline) {
            throw new Error(`${selector} did not settle at opacity ${opacity} (last ${current})`);
        }
        await new Promise((resolve) => requestAnimationFrame(resolve));
    }
}"#;

// `hero_card_style(false)` opacity; the mount animation settles here.
const HERO_BASELINE_OPACITY: f64 = 0.86;

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
struct Sample {
    id: String,
    transform: String,
    opacity: f64,
    left: f64,
    width: f64,
    order: i32,
    radius_x: f64,
    radius_y: f64,
    scale_x: f64,
    scale_y: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WaitForOpacity<'a> {
    selector: &'a str,
    opacity: f64,
    timeout_ms: u32,
}

#[derive(Serialize)]
struct ClickAndSample<'a> {
    button: &'a str,
    selector: &'a str,
    delays: &'a [u32],
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
    run_flip_border_radius_correction_check(browser, base_url).await?;
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

    if let Err(error) = wait_for_visible(&page, ".hero-showcase", Duration::from_secs(12)).await {
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

    page.evaluate::<_, bool>(
        WAIT_FOR_OPACITY_JS,
        Some(&WaitForOpacity {
            selector: ".hero-showcase",
            opacity: HERO_BASELINE_OPACITY,
            timeout_ms: 3000,
        }),
    )
    .await
    .context("Hero showcase never settled at its baseline")?;

    let samples = click_and_sample_one(
        &page,
        "Energize showcase",
        ".hero-showcase",
        &MOTION_SAMPLE_DELAYS_MS,
    )
    .await?;
    let [start, mid, end] = samples.as_slice() else {
        bail!("Expected 3 hero samples, got {}", samples.len());
    };

    ensure!(
        style_delta(start, end) > 0.01,
        "Hero motion target did not change after toggle"
    );
    ensure!(
        style_delta(start, mid) > 0.01,
        "Hero motion never moved away from start state"
    );
    ensure!(
        style_delta(mid, end) > 0.01,
        "Hero motion jumped directly to end state (no observable interpolation)"
    );
    ensure!(
        start.opacity + 0.005 < mid.opacity && mid.opacity < end.opacity - 0.005,
        "Hero motion opacity did not pass through an intermediate value: start={:.3}, mid={:.3}, end={:.3}",
        start.opacity,
        mid.opacity,
        end.opacity
    );

    page.close().await?;
    Ok(())
}

async fn run_single_flip_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression] Running single FLIP transition check");
    let page = open_demo_page(browser, base_url).await?;

    let samples = click_and_sample_one(
        &page,
        "Dock right",
        ".flip-workbench-card",
        &FLIP_SAMPLE_DELAYS_MS,
    )
    .await?;
    let [start, mid, end] = samples.as_slice() else {
        bail!("Expected 3 FLIP samples, got {}", samples.len());
    };

    ensure!(
        !is_identity_transform(&mid.transform),
        "FLIP single-element animation never entered an in-flight transform state"
    );
    ensure!(
        end.left > start.left + 60.0,
        "FLIP single-element layout position did not move far enough: start_left={:.2}, end_left={:.2}",
        start.left,
        end.left
    );
    ensure!(
        start.left < mid.left && mid.left < end.left,
        "FLIP single-element did not pass through an intermediate visual position: start_left={:.2}, mid_left={:.2}, end_left={:.2}",
        start.left,
        mid.left,
        end.left
    );
    ensure!(
        is_identity_transform(&end.transform),
        "FLIP single-element transform did not settle back to identity after animation"
    );

    page.close().await?;
    Ok(())
}

async fn run_flip_border_radius_correction_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression] Running FLIP border-radius correction check");
    let page = open_demo_page(browser, base_url).await?;

    // Standard -> Panorama resizes the card, so the FLIP plays a non-unit scale
    // and the library must counter-scale border-radius to keep corners round.
    let samples = click_and_sample_one(
        &page,
        "Panorama",
        ".flip-workbench-card",
        &FLIP_SAMPLE_DELAYS_MS,
    )
    .await?;
    let [start, mid, end] = samples.as_slice() else {
        bail!("Expected 3 FLIP resize samples, got {}", samples.len());
    };

    ensure!(
        end.width > start.width + 40.0,
        "FLIP card did not resize to the panorama layout: start_width={:.2}, end_width={:.2}",
        start.width,
        end.width
    );
    ensure!(
        (mid.scale_x - 1.0).abs() > 0.02 && (mid.scale_y - 1.0).abs() > 0.02,
        "FLIP resize was not scaling when sampled (scale_x={:.3}, scale_y={:.3}); border-radius correction was not exercised",
        mid.scale_x,
        mid.scale_y
    );
    ensure!(
        (mid.radius_x - start.radius_x).abs() > 0.5,
        "FLIP border radius was not counter-scaled during the resize: rest={:.2}px, mid={:.2}px (scale_x={:.3})",
        start.radius_x,
        mid.radius_x,
        mid.scale_x
    );
    let visual_x = mid.radius_x * mid.scale_x;
    let visual_y = mid.radius_y * mid.scale_y;
    ensure!(
        (visual_x - start.radius_x).abs() < 0.5 && (visual_y - start.radius_y).abs() < 0.5,
        "FLIP border radius was visually distorted during the resize: rest={:.2}px x {:.2}px, visual={visual_x:.2}px x {visual_y:.2}px",
        start.radius_x,
        start.radius_y
    );
    ensure!(
        is_identity_transform(&end.transform)
            && (end.radius_x - start.radius_x).abs() < 0.01
            && (end.radius_y - start.radius_y).abs() < 0.01,
        "FLIP border radius correction did not restore the resting radius after settle: rest={:.2}px, end={:.2}px x {:.2}px, end_transform={}",
        start.radius_x,
        end.radius_x,
        end.radius_y,
        end.transform
    );

    page.close().await?;
    Ok(())
}

async fn run_flip_group_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression] Running FLIP group reorder check");
    let page = open_demo_page(browser, base_url).await?;

    let frames = click_and_sample(
        &page,
        "Rotate",
        ".flip-board-tile",
        &FLIP_GROUP_SAMPLE_DELAYS_MS,
    )
    .await?;
    let [start, mid, end] = frames.as_slice() else {
        bail!("Expected 3 FLIP group samples, got {}", frames.len());
    };

    let in_flight = mid
        .iter()
        .filter(|tile| !is_identity_transform(&tile.transform))
        .count();
    ensure!(
        in_flight >= 2,
        "FLIP group animation did not move multiple tiles in flight: {in_flight} of {} tiles had a non-identity transform",
        mid.len()
    );
    ensure!(
        end.iter()
            .all(|tile| is_identity_transform(&tile.transform)),
        "FLIP group tiles did not settle back to identity transforms after animation"
    );

    let order_of = |tiles: &[Sample]| {
        tiles
            .iter()
            .find(|tile| tile.id == "suite-a")
            .map(|tile| tile.order)
    };
    let (Some(start_order), Some(end_order)) = (order_of(start), order_of(end)) else {
        bail!("FLIP group board is missing tile #suite-a");
    };
    ensure!(
        end_order != start_order,
        "FLIP group reorder did not change CSS order for #suite-a ({} -> {})",
        start_order,
        end_order
    );

    page.close().await?;
    Ok(())
}

/// Runs [`CLICK_AND_SAMPLE_JS`]; frame 0 is the pre-click baseline.
async fn click_and_sample(
    page: &Page,
    button: &str,
    selector: &str,
    delays: &[u32],
) -> Result<Vec<Vec<Sample>>> {
    page.evaluate(
        CLICK_AND_SAMPLE_JS,
        Some(&ClickAndSample {
            button,
            selector,
            delays,
        }),
    )
    .await
    .with_context(|| format!("Failed to click `{button}` and sample `{selector}`"))
}

/// [`click_and_sample`] for a selector that must match exactly one element.
async fn click_and_sample_one(
    page: &Page,
    button: &str,
    selector: &str,
    delays: &[u32],
) -> Result<Vec<Sample>> {
    click_and_sample(page, button, selector, delays)
        .await?
        .into_iter()
        .map(|mut frame| {
            ensure!(
                frame.len() == 1,
                "Expected exactly one element for `{selector}`, found {}",
                frame.len()
            );
            Ok(frame.remove(0))
        })
        .collect()
}

fn style_delta(a: &Sample, b: &Sample) -> f64 {
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
