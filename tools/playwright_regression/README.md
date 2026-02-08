# playwright_regression

Playwright-based animation regression checks for `leptos_fluid`.

This crate runs end-to-end browser checks against the built `example_motion` demo and verifies animation behavior in ways that catch common regressions:

- state transitions must visibly interpolate (not jump)
- FLIP single-element transitions must show in-flight transform and settle cleanly
- FLIP group reorders must animate and apply the new order

## What it tests

Current checks:

1. Motion transition progression (`HeroSection`)
2. FLIP single-element move (`FlipSection`)
3. FLIP group reorder (`FlipGroupSection`)

These checks validate both intermediate animation frames and final settled states.

## Prerequisites

1. Build demo assets

```bash
cd example_motion
trunk build
cd ..
```

2. Install Playwright Chromium browser matching the bundled Playwright version

```bash
npx playwright@1.56.1 install chromium
```

## Run

```bash
cargo run -p playwright_regression --
```

Optional flags:

- `--dist-dir <PATH>`: static files directory (default `example_motion/dist`)
- `--port <PORT>`: local test server port (default `4173`)
- `--headed`: run Chromium with UI

Example:

```bash
cargo run -p playwright_regression -- --headed --port 4300
```

## CI recommendation

Run this tool as a post-build browser regression gate for animation-related changes. A typical CI job sequence:

1. build `example_motion` static output
2. install Playwright Chromium
3. run `cargo run -p playwright_regression --`

If any regression check fails, the process exits non-zero and prints a targeted failure reason.
