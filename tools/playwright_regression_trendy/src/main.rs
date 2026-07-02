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

const DEFAULT_DIST_DIR: &str = "example_trendy/dist";
const DEFAULT_PORT: u16 = 4175;
const REQUEST_BUFFER_LEN: usize = 16 * 1024;

/// Wait after a programmatic scroll for the scroll engine + rAF to process.
const SCROLL_SETTLE: Duration = Duration::from_millis(200);
/// Wait after a programmatic scroll for `Scrub::Number(0.15)` smoothing to converge.
///
/// The smoothing lerps displayed progress toward raw progress by factor 0.15 per rAF
/// (~16ms). After 700ms (~44 frames) the residual error is `0.85^44 ≈ 0.0009`, i.e. the
/// displayed value has converged to within ~0.1% of the raw value. Waiting shorter than
/// this leaves the displayed transform stale from the previous scroll position, which
/// makes scrubbed-section comparisons non-deterministic.
const SCRUB_SETTLE: Duration = Duration::from_millis(700);
/// Wait after a one-shot on_enter animation (WAAPI) starts playing.
const ENTER_ANIM_WAIT: Duration = Duration::from_millis(600);
/// Wait for the staggered grid cascade to complete (6 cards * 100ms stagger + 500ms duration).
const STAGGER_WAIT: Duration = Duration::from_millis(800);
/// Wait for the rAF-driven counter to advance.
const COUNTER_WAIT: Duration = Duration::from_millis(800);
/// Wait for the spring to respond to mousemove.
const SPRING_RESPONSE_WAIT: Duration = Duration::from_millis(300);
/// Wait for the spring to settle back after mouseleave.
const SPRING_SETTLE_WAIT: Duration = Duration::from_millis(400);
/// Gap between two marquee track reads to detect continuous motion.
const MARQUEE_SAMPLE_GAP: Duration = Duration::from_millis(200);
/// Wait for the marquee rAF loop to start writing inline transforms after the section
/// becomes active. On a fresh page load the loop spins up over the first few hundred ms.
const MARQUEE_STARTUP_WAIT: Duration = Duration::from_millis(600);

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
            "Could not find {}. Build the demo first (for example: `cd example_trendy && trunk build`).",
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
            .name("playwright-regression-trendy-server".to_string())
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
                        "[playwright_regression_trendy] static server failed to set stream blocking mode: {error}"
                    );
                    continue;
                }
                if let Err(error) = handle_connection(&mut stream, &root) {
                    eprintln!("[playwright_regression_trendy] static server request error: {error:#}");
                }
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(error) => {
                eprintln!("[playwright_regression_trendy] static server failed: {error}");
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
struct PointSnapshot {
    x: f64,
    y: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_args()?;
    let server = StaticServer::start(config.dist_dir.clone(), config.port)?;
    println!(
        "[playwright_regression_trendy] Serving {} at {}",
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
    println!("[playwright_regression_trendy] All scroll animation regression checks passed");
    Ok(())
}

async fn run_regression_suite(browser: &Browser, base_url: &str) -> Result<()> {
    run_all_sections_render_check(browser, base_url).await?;
    run_sticky_hero_check(browser, base_url).await?;
    run_horizontal_gallery_check(browser, base_url).await?;
    run_text_mask_reveal_check(browser, base_url).await?;
    run_stagger_grid_check(browser, base_url).await?;
    run_counter_check(browser, base_url).await?;
    run_image_reveal_check(browser, base_url).await?;
    run_perspective_tilt_check(browser, base_url).await?;
    run_velocity_marquee_check(browser, base_url).await?;
    run_color_morph_check(browser, base_url).await?;
    run_magnetic_cta_check(browser, base_url).await?;
    run_scroll_restoration_check(browser, base_url).await?;
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

    if let Err(error) = wait_for_visible(&page, "#hero", Duration::from_secs(15)).await {
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
            "Demo did not render: {error}. ready_state={ready_state}, body_snippet={snippet:?}"
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

// ---------------------------------------------------------------------------
// Shared style / geometry helpers (copied from playwright_regression)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Trendy-specific read helpers
// ---------------------------------------------------------------------------

/// Scroll the page to a specific Y position and wait for the scroll engine + rAF to process.
async fn scroll_to(page: &Page, y: f64) -> Result<()> {
    page.evaluate::<f64, ()>("(y) => window.scrollTo(0, y)", Some(&y))
        .await
        .context("Failed to scroll")?;
    tokio::time::sleep(SCROLL_SETTLE).await;
    Ok(())
}

/// Scroll to a Y position and wait long enough for `Scrub::Number(0.15)` smoothing to
/// converge to the new raw progress. Use this for scrubbed sections (hero, gallery, tilt,
/// color) where the displayed transform lags the raw scroll position.
async fn scroll_to_scrubbed(page: &Page, y: f64) -> Result<()> {
    page.evaluate::<f64, ()>("(y) => window.scrollTo(0, y)", Some(&y))
        .await
        .context("Failed to scroll")?;
    tokio::time::sleep(SCRUB_SETTLE).await;
    Ok(())
}

/// Scroll from the top of the page (scroll 0) to `target` in small incremental steps.
///
/// `once: true` `on_enter` triggers fire when the scroll engine's rAF tick observes the
/// target crossing into the trigger band. Two conditions defeat this in the trendy demo:
///   1. A single big `scrollTo` jump from below the band to above it skips the
///      edge-detection frame and the one-shot callback never fires.
///   2. Jumping *past* an earlier scroll-trigger section (notably the pinned horizontal
///      gallery) leaves the scroll engine in a state where later `once` triggers do not
///      fire even on a subsequent incremental scroll across their band.
///
/// Scrolling incrementally from the very top of the page (in ~`STEP`-px increments with a
/// short rAF gap between each) avoids both problems: the engine observes every enter
/// transition cleanly. After reaching the target, an optional `settle` wait lets any WAAPI
/// entrance animation play.
async fn scroll_from_top_incremental(page: &Page, target: f64, settle: Duration) -> Result<()> {
    // Start from a known clean state at the top of the page.
    page.evaluate::<f64, ()>("(y) => window.scrollTo(0, y)", Some(&0.0))
        .await
        .context("Failed to reset scroll to top")?;
    tokio::time::sleep(SCROLL_SETTLE).await;

    const STEP: f64 = 90.0;
    let steps = ((target / STEP).ceil() as i64).max(1);
    for i in 1..=steps {
        let y = target * (i as f64) / (steps as f64);
        page.evaluate::<f64, ()>("(y) => window.scrollTo(0, y)", Some(&y))
            .await
            .context("Failed to scroll")?;
        // One rAF tick (~16ms) plus a small margin so the engine evaluates triggers.
        tokio::time::sleep(Duration::from_millis(40)).await;
    }
    // Ensure we land exactly on the target.
    page.evaluate::<f64, ()>("(y) => window.scrollTo(0, y)", Some(&target))
        .await
        .context("Failed to scroll")?;
    if !settle.is_zero() {
        tokio::time::sleep(settle).await;
    } else {
        tokio::time::sleep(SCROLL_SETTLE).await;
    }
    Ok(())
}

/// Get the current scroll Y position of the window.
#[allow(dead_code)]
async fn get_scroll_y(page: &Page) -> Result<f64> {
    page.evaluate::<(), f64>("() => window.scrollY", None::<&()>)
        .await
        .context("Failed to read scrollY")
}

/// Get the Y position of an element relative to the document (top + scrollY).
async fn get_element_y(page: &Page, selector: &str) -> Result<f64> {
    let selector = selector.to_string();
    page.evaluate(
        r#"(selector) => {
            const el = document.querySelector(selector);
            if (!el) throw new Error(`Missing element: ${selector}`);
            const rect = el.getBoundingClientRect();
            return rect.top + window.scrollY;
        }"#,
        Some(&selector),
    )
    .await
    .with_context(|| format!("Failed to get element Y for `{selector}`"))
}

/// Read the inline `transform` value of an element (from the style attribute, not computed).
///
/// `bind_controller` writes via `set_immediate`, which sets inline styles. The computed
/// `transform` may be normalized by the browser (e.g. `perspective(...) rotateY(...)` becomes
/// a `matrix3d(...)`), so reading the inline value gives a stable, author-string comparison.
async fn read_inline_transform(page: &Page, selector: &str) -> Result<String> {
    let selector = selector.to_string();
    page.evaluate(
        r#"(selector) => {
            const el = document.querySelector(selector);
            if (!el) throw new Error(`Missing element: ${selector}`);
            return el.style.transform || "";
        }"#,
        Some(&selector),
    )
    .await
    .with_context(|| format!("Failed to read inline transform for `{selector}`"))
}

/// Read the computed `transform` value of an element.
#[allow(dead_code)]
async fn read_transform(page: &Page, selector: &str) -> Result<String> {
    let selector = selector.to_string();
    page.evaluate(
        r#"(selector) => {
            const el = document.querySelector(selector);
            if (!el) throw new Error(`Missing element: ${selector}`);
            return window.getComputedStyle(el).transform;
        }"#,
        Some(&selector),
    )
    .await
    .with_context(|| format!("Failed to read transform for `{selector}`"))
}

/// Read the computed `background-color` of an element.
async fn read_background_color(page: &Page, selector: &str) -> Result<String> {
    let selector = selector.to_string();
    page.evaluate(
        r#"(selector) => {
            const el = document.querySelector(selector);
            if (!el) throw new Error(`Missing element: ${selector}`);
            return window.getComputedStyle(el).backgroundColor;
        }"#,
        Some(&selector),
    )
    .await
    .with_context(|| format!("Failed to read background-color for `{selector}`"))
}

/// Read the computed `opacity` of an element as a float.
async fn read_opacity(page: &Page, selector: &str) -> Result<f64> {
    let selector = selector.to_string();
    page.evaluate(
        r#"(selector) => {
            const el = document.querySelector(selector);
            if (!el) throw new Error(`Missing element: ${selector}`);
            return Number.parseFloat(window.getComputedStyle(el).opacity || "1");
        }"#,
        Some(&selector),
    )
    .await
    .with_context(|| format!("Failed to read opacity for `{selector}`"))
}

/// Read the computed `clip-path` of an element.
async fn read_clip_path(page: &Page, selector: &str) -> Result<String> {
    let selector = selector.to_string();
    page.evaluate(
        r#"(selector) => {
            const el = document.querySelector(selector);
            if (!el) throw new Error(`Missing element: ${selector}`);
            return window.getComputedStyle(el).clipPath;
        }"#,
        Some(&selector),
    )
    .await
    .with_context(|| format!("Failed to read clip-path for `{selector}`"))
}

/// Read the trimmed inner text of an element.
async fn read_text(page: &Page, selector: &str) -> Result<String> {
    let text = page
        .locator(selector)
        .await
        .inner_text()
        .await
        .with_context(|| format!("Failed to read inner text for `{selector}`"))?;
    Ok(text)
}

// ---------------------------------------------------------------------------
// Regression checks
// ---------------------------------------------------------------------------

/// Verify all 10 sections + footer are present and visible in the DOM.
async fn run_all_sections_render_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression_trendy] Running all sections render check");
    let page = open_demo_page(browser, base_url).await?;

    let sections = [
        "#hero", "#gallery", "#text", "#grid", "#counter", "#image", "#tilt", "#marquee", "#color",
        "#cta",
    ];

    for selector in &sections {
        let visible = page.locator(selector).await.is_visible().await;
        ensure!(
            visible.unwrap_or(false),
            "Section {selector} is not visible"
        );
    }

    let footer_visible = page.locator(".footer").await.is_visible().await;
    ensure!(footer_visible.unwrap_or(false), "Footer is not visible");

    page.close().await?;
    Ok(())
}

/// Scroll through the hero section and verify the title has a non-identity transform
/// (parallax active) that progresses with scroll.
///
/// The hero trigger is `start: top top, end: bottom top` over the section height, scrubbed
/// with `Scrub::Number(0.15)`. Raw progress sweeps 0→1 across the first viewport of scroll.
/// We compare the title's inline transform at a low-progress and a high-progress scroll
/// position, waiting for the scrub smoothing to converge at each.
async fn run_sticky_hero_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression_trendy] Running sticky hero parallax check");
    let page = open_demo_page(browser, base_url).await?;

    // Low raw progress (~0.24): title still near its initial large/solid state.
    scroll_to_scrubbed(&page, 150.0).await?;
    let title_low = read_inline_transform(&page, ".hero-title").await?;

    // High raw progress (~0.89): title scaled down and shifted up.
    scroll_to_scrubbed(&page, 550.0).await?;
    let title_high = read_inline_transform(&page, ".hero-title").await?;

    ensure!(
        title_low != title_high,
        "Hero title transform did not progress with scroll: low={title_low:?}, high={title_high:?}"
    );
    // At high progress the title should be noticeably transformed (not identity).
    ensure!(
        !is_identity_transform(&title_high),
        "Hero title never entered a non-identity parallax state: high={title_high:?}"
    );

    page.close().await?;
    Ok(())
}

/// Scroll to the gallery section and verify the track translates horizontally as the
/// section is scrubbed.
///
/// The gallery trigger is `start: top top, end: bottom bottom`, scrubbed with
/// `Scrub::Number(0.15)`. The track translateX maps p=0..1 onto the scrollable track width.
/// We sample at three increasing raw-progress points and require monotonic movement.
async fn run_horizontal_gallery_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression_trendy] Running horizontal gallery check");
    let page = open_demo_page(browser, base_url).await?;

    let gallery_y = get_element_y(&page, "#gallery").await?;

    // Near the start of the scrub range: track close to translateX(0).
    scroll_to_scrubbed(&page, gallery_y + 50.0).await?;
    let track_start = read_inline_transform(&page, ".gallery-track").await?;

    // Mid progress: track should have translated negatively.
    scroll_to_scrubbed(&page, gallery_y + 400.0).await?;
    let track_mid = read_inline_transform(&page, ".gallery-track").await?;
    ensure!(
        track_mid != track_start,
        "Gallery track did not move horizontally: start={track_start:?}, mid={track_mid:?}"
    );

    // Higher progress: track should keep translating.
    scroll_to_scrubbed(&page, gallery_y + 800.0).await?;
    let track_late = read_inline_transform(&page, ".gallery-track").await?;
    ensure!(
        track_late != track_mid,
        "Gallery track did not continue moving: mid={track_mid:?}, late={track_late:?}"
    );

    page.close().await?;
    Ok(())
}

/// Scroll to the text section and verify the lines animate (opacity/transform changes)
/// when the one-shot on_enter trigger fires.
///
/// The text trigger is `start: top 80%, once: true`. Lines start at `translateY(120px)`
/// (CSS initial) and animate to `translateY(0)` via WAAPI on enter. We read the computed
/// transform before the trigger band and after the on_enter animation has played.
async fn run_text_mask_reveal_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression_trendy] Running text mask reveal check");
    let page = open_demo_page(browser, base_url).await?;

    let text_y = get_element_y(&page, "#text").await?;

    // Before the trigger band (top 80%): lines still offset down at their initial state.
    // The trigger fires when section top reaches 80% of the viewport, i.e. at
    // scroll = text_y - 0.8*vh. Reach a position below that band by scrolling incrementally
    // from the top so no earlier pinned section poisons the once-trigger.
    scroll_from_top_incremental(&page, text_y - 650.0, Duration::ZERO).await?;
    let line_before = read_style(&page, ".line").await?;

    // Scroll incrementally from the top across the trigger band so the once:on_enter fires
    // and the staggered WAAPI tweens play.
    scroll_from_top_incremental(&page, text_y + 100.0, ENTER_ANIM_WAIT).await?;
    let line_after = read_style(&page, ".line").await?;

    ensure!(
        style_delta(&line_before, &line_after) > 0.01,
        "Text lines did not animate after entering section: before={line_before:?}, after={line_after:?}"
    );

    page.close().await?;
    Ok(())
}

/// Scroll to the grid section and verify the cards animate in (opacity + transform).
///
/// Each card owns a `ScrollTrigger` with `start: top 85%, once: true` that animates it from
/// `opacity(0) y(60) scale(0.95)` to `opacity(1) y(0) scale(1)` with a per-card 100ms stagger.
/// We read the first card's computed style before the trigger band and after the cascade.
async fn run_stagger_grid_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression_trendy] Running stagger grid check");
    let page = open_demo_page(browser, base_url).await?;

    let grid_y = get_element_y(&page, "#grid").await?;

    // Before the trigger band (top 85%): cards in their initial hidden state.
    // The trigger fires when a card top reaches 85% of the viewport, i.e. at
    // scroll = card_y - 0.85*vh. Reach a position below that band by scrolling incrementally
    // from the top so no earlier pinned section poisons the once-triggers.
    scroll_from_top_incremental(&page, grid_y - 750.0, Duration::ZERO).await?;
    let card_before = read_style(&page, ".stagger-card").await?;

    // Scroll incrementally from the top across each card's trigger band so every
    // once:on_enter fires, then wait for the staggered cascade to complete.
    scroll_from_top_incremental(&page, grid_y + 100.0, STAGGER_WAIT).await?;
    let card_after = read_style(&page, ".stagger-card").await?;

    ensure!(
        style_delta(&card_before, &card_after) > 0.01,
        "Stagger cards did not animate: before={card_before:?}, after={card_after:?}"
    );

    // Cards should be visible (opacity near 1) after the entrance animation settles.
    ensure!(
        card_after.opacity > card_before.opacity || card_after.opacity >= 0.99,
        "Stagger cards did not become visible: before_opacity={:.2}, after_opacity={:.2}",
        card_before.opacity,
        card_after.opacity
    );

    page.close().await?;
    Ok(())
}

/// Scroll to the counter section and verify the number counts up from 0 on enter.
///
/// The counter trigger is `start: top 80%, once: true`. On enter it kicks off a one-shot
/// rAF-driven counter from 0 to 1247 over ~1.5s. We read the number at the top of the page
/// (0) and after scrolling incrementally into the section so the once:on_enter fires.
async fn run_counter_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression_trendy] Running counter check");
    let page = open_demo_page(browser, base_url).await?;

    // At the top of the page the counter should read 0 (or its initial value).
    let counter_before = read_text(&page, ".counter-number").await?;
    let before_val: i64 = counter_before.trim().parse().unwrap_or(0);

    // Scroll incrementally from the top across the section's enter trigger (start: top 80%)
    // so the once:on_enter fires and the rAF counter starts, then wait for it to advance.
    let counter_y = get_element_y(&page, "#counter").await?;
    scroll_from_top_incremental(&page, counter_y + 100.0, COUNTER_WAIT).await?;

    let counter_after = read_text(&page, ".counter-number").await?;
    let after_val: i64 = counter_after.trim().parse().unwrap_or(0);

    ensure!(
        after_val > before_val,
        "Counter did not increase: before={before_val}, after={after_val}"
    );

    page.close().await?;
    Ok(())
}

/// Scroll to the image section and verify the clip-path opens and the label opacity rises.
///
/// The image trigger is `start: top 80%` with a one-shot on_enter that animates the frame's
/// clip-path from `inset(50% 0 50% 0)` to `inset(0% 0 0% 0)`. The controller targets the
/// `.image-frame` element (the outer frame, not the inner gradient fill). The label opacity
/// is reactively bound to `is_active` (0 when idle, 1 when active). We sample both before
/// the trigger band and after the on_enter animation has played.
async fn run_image_reveal_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression_trendy] Running image reveal check");
    let page = open_demo_page(browser, base_url).await?;

    let image_y = get_element_y(&page, "#image").await?;

    // Before the trigger band (top 80%): clip-path is the initial inset slit, label hidden.
    // The trigger fires when the frame top reaches 80% of the viewport, i.e. at
    // scroll = image_y - 0.8*vh. Reach a position below that band by scrolling incrementally
    // from the top so no earlier pinned section poisons the once-trigger.
    scroll_from_top_incremental(&page, image_y - 700.0, Duration::ZERO).await?;
    let clip_before = read_clip_path(&page, ".image-frame").await?;
    let opacity_before = read_opacity(&page, ".image-label").await?;

    // Scroll incrementally from the top across the trigger band so the once:on_enter fires
    // and the WAAPI clip-path tween plays.
    scroll_from_top_incremental(&page, image_y + 100.0, ENTER_ANIM_WAIT).await?;

    let clip_after = read_clip_path(&page, ".image-frame").await?;
    let opacity_after = read_opacity(&page, ".image-label").await?;

    ensure!(
        clip_before != clip_after,
        "Image clip-path did not change: before={clip_before:?}, after={clip_after:?}"
    );

    ensure!(
        opacity_after > opacity_before,
        "Image label opacity did not increase: before={opacity_before}, after={opacity_after}"
    );

    page.close().await?;
    Ok(())
}

/// Scroll to the tilt section and verify the card's 3D transform changes with scroll.
///
/// The tilt trigger is `start: top 90%, end: bottom 10%`, scrubbed with `Scrub::Number(0.15)`.
/// The card transform sweeps `rotateY(-45→45) rotateX(15→-15) scale(0.9→1.1)` across the range.
/// We sample the inline transform at three increasing raw-progress points.
async fn run_perspective_tilt_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression_trendy] Running perspective tilt check");
    let page = open_demo_page(browser, base_url).await?;

    let tilt_y = get_element_y(&page, "#tilt").await?;

    // Start of the section (mid-scrub): initial sampled transform.
    scroll_to_scrubbed(&page, tilt_y).await?;
    let transform_start = read_inline_transform(&page, ".tilt-card").await?;

    // Scroll further; rotateY/rotateX/scale should progress.
    scroll_to_scrubbed(&page, tilt_y + 300.0).await?;
    let transform_mid = read_inline_transform(&page, ".tilt-card").await?;
    ensure!(
        transform_mid != transform_start,
        "Tilt card transform did not change with scroll: start={transform_start:?}, mid={transform_mid:?}"
    );

    // Scroll even further; the transform should keep evolving.
    scroll_to_scrubbed(&page, tilt_y + 600.0).await?;
    let transform_late = read_inline_transform(&page, ".tilt-card").await?;
    ensure!(
        transform_late != transform_mid,
        "Tilt card transform did not continue changing: mid={transform_mid:?}, late={transform_late:?}"
    );

    page.close().await?;
    Ok(())
}

/// Scroll to the marquee section and verify the track is continuously moving via rAF.
///
/// The marquee track is driven by a recursive rAF loop that writes `translateX(offset)` via
/// `set_immediate` every frame while the section is active. The loop spins up over the first
/// few hundred milliseconds after the section becomes active, so we wait for it to start
/// writing before sampling two reads and requiring them to differ.
async fn run_velocity_marquee_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression_trendy] Running velocity marquee check");
    let page = open_demo_page(browser, base_url).await?;

    // Scroll to the marquee section so is_active becomes true and the rAF loop writes styles.
    let marquee_y = get_element_y(&page, "#marquee").await?;
    scroll_to(&page, marquee_y).await?;
    // Wait for the rAF loop to spin up and begin writing inline transforms.
    tokio::time::sleep(MARQUEE_STARTUP_WAIT).await;

    // Sample the inline transform twice; the rAF loop should have advanced it.
    let track_1 = read_inline_transform(&page, ".marquee-track").await?;
    tokio::time::sleep(MARQUEE_SAMPLE_GAP).await;
    let track_2 = read_inline_transform(&page, ".marquee-track").await?;

    ensure!(
        track_1 != track_2,
        "Marquee track is not moving: t1={track_1:?}, t2={track_2:?}"
    );

    page.close().await?;
    Ok(())
}

/// Scroll to the color section and verify the background-color interpolates with scroll.
///
/// The color trigger is `start: top 80%, end: bottom 20%`, scrubbed with `Scrub::Number(0.15)`.
/// The block background-color lerps from `rgba(255,45,117,1)` (magenta) toward
/// `rgba(0,71,255,1)` (blue) across the range. We sample the computed background-color at
/// three increasing raw-progress points.
async fn run_color_morph_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression_trendy] Running color morph check");
    let page = open_demo_page(browser, base_url).await?;

    let color_y = get_element_y(&page, "#color").await?;

    // Start of the scrub range: initial magenta-leaning color.
    scroll_to_scrubbed(&page, color_y).await?;
    let color_start = read_background_color(&page, ".color-block").await?;

    // Scroll further; the rgba lerp should produce a different computed color.
    scroll_to_scrubbed(&page, color_y + 300.0).await?;
    let color_mid = read_background_color(&page, ".color-block").await?;
    ensure!(
        color_start != color_mid,
        "Color block did not change background-color: start={color_start:?}, mid={color_mid:?}"
    );

    // Scroll even further; the color should keep morphing toward blue.
    scroll_to_scrubbed(&page, color_y + 600.0).await?;
    let color_late = read_background_color(&page, ".color-block").await?;
    ensure!(
        color_mid != color_late,
        "Color block did not continue changing: mid={color_mid:?}, late={color_late:?}"
    );

    page.close().await?;
    Ok(())
}

/// Hover over the magnetic CTA button and verify it moves (spring-driven), then settles
/// back after the mouse leaves.
///
/// The spring-driven `AnimationController` targets the `<button class="magnetic-btn">` (not
/// the outer `.magnetic-wrap`), writing `x(spring_x).y(spring_y)` via `bind_set_immediate`.
/// On `mousemove` over the wrap the springs are set to a fraction of the cursor offset from
/// the wrap center; on `mouseleave` they are reset to 0 and the spring settles back.
async fn run_magnetic_cta_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression_trendy] Running magnetic CTA check");
    let page = open_demo_page(browser, base_url).await?;

    // Scroll to the CTA section so the wrap is in the viewport.
    let cta_y = get_element_y(&page, "#cta").await?;
    scroll_to(&page, cta_y).await?;
    tokio::time::sleep(SCROLL_SETTLE).await;

    // Read the button's inline transform before any hover (should be at rest, ~identity).
    let btn_before = read_inline_transform(&page, ".magnetic-btn").await?;

    // Locate the button center in viewport coordinates so we can move the mouse onto it.
    let btn_box: PointSnapshot = page
        .evaluate(
            r#"() => {
                const el = document.querySelector('.magnetic-btn');
                if (!el) throw new Error('Missing .magnetic-btn');
                const r = el.getBoundingClientRect();
                return { x: r.x + r.width / 2, y: r.y + r.height / 2 };
            }"#,
            None::<&()>,
        )
        .await
        .context("Failed to read magnetic button center")?;

    // Move the mouse onto the button, offset slightly so the spring gets a non-zero target.
    // playwright-rs Mouse::move_to takes i32 viewport pixels.
    page.mouse()
        .move_to(
            (btn_box.x + 20.0) as i32,
            (btn_box.y + 10.0) as i32,
            None,
        )
        .await
        .context("Failed to move mouse onto magnetic button")?;
    tokio::time::sleep(SPRING_RESPONSE_WAIT).await;

    let btn_during = read_inline_transform(&page, ".magnetic-btn").await?;
    ensure!(
        btn_before != btn_during || !is_identity_transform(&btn_during),
        "Magnetic button did not move on hover: before={btn_before:?}, during={btn_during:?}"
    );

    // Move the mouse away (to the viewport origin) and let the spring settle back to 0.
    page.mouse()
        .move_to(0, 0, None)
        .await
        .context("Failed to move mouse away from magnetic button")?;
    tokio::time::sleep(SPRING_SETTLE_WAIT).await;

    let btn_after = read_inline_transform(&page, ".magnetic-btn").await?;
    ensure!(
        is_identity_transform(&btn_after) || btn_after != btn_during,
        "Magnetic button did not settle back after mouse leave: during={btn_during:?}, after={btn_after:?}"
    );

    page.close().await?;
    Ok(())
}

/// Verify that scrolling back to top resets the hero parallax (progress returns to ~0).
///
/// After scrolling down (hero progress high) and then back to the top, the scrub smoothing
/// should converge the title transform back to its initial (low-progress) state, which
/// differs from the scrolled-down state.
async fn run_scroll_restoration_check(browser: &Browser, base_url: &str) -> Result<()> {
    println!("[playwright_regression_trendy] Running scroll restoration check");
    let page = open_demo_page(browser, base_url).await?;

    // Scroll down to drive the hero parallax forward (raw progress ~0.81).
    scroll_to_scrubbed(&page, 500.0).await?;
    let title_scrolled = read_inline_transform(&page, ".hero-title").await?;

    // Scroll back to top and let the scrub smoothing converge to raw progress 0.
    scroll_to_scrubbed(&page, 0.0).await?;
    let title_top = read_inline_transform(&page, ".hero-title").await?;

    // The title transform should differ from the scrolled-down state (back to initial).
    ensure!(
        title_top != title_scrolled,
        "Hero title did not reset after scrolling back to top: scrolled={title_scrolled:?}, top={title_top:?}"
    );

    page.close().await?;
    Ok(())
}

fn print_help() {
    println!(
        "playwright_regression_trendy\n\n\
Runs scroll-driven animation regression checks against the built example_trendy app.\n\n\
USAGE:\n\
  cargo run -p playwright_regression_trendy -- [OPTIONS]\n\n\
OPTIONS:\n\
  --dist-dir <PATH>   Directory containing built static files (default: {DEFAULT_DIST_DIR})\n\
  --port <PORT>       Local HTTP port used by the static test server (default: {DEFAULT_PORT})\n\
  --headed            Run Chromium in headed mode\n\
  -h, --help          Show this help\n\n\
PREREQUISITES:\n\
  1) Build the demo: `cd example_trendy && trunk build`\n\
  2) Install browsers: `npx playwright@{PLAYWRIGHT_VERSION} install chromium`"
    );
}