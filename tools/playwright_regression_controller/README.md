# playwright_regression_controller

Playwright-based animation regression checks for the controller-only demo.

This crate runs end-to-end browser checks against the built `example_motion_controller` app and validates `AnimationController` behavior on plain elements.

## What it tests

Current checks:

1. Declarative `bind` transition progression (start -> mid -> settled)
2. Tab underline retargeting across selected tabs (including rapid re-selection)
3. App-managed pointer interaction states (base -> hover -> press -> release)
4. Queue-latest replay while target is detached

## Prerequisites

1. Build demo assets

```bash
cd example_motion_controller
trunk build
cd ..
```

2. Install Playwright Chromium browser matching the bundled Playwright version

```bash
npx playwright@1.56.1 install chromium
```

## Run

```bash
cargo run -p playwright_regression_controller --
```

Optional flags:

- `--dist-dir <PATH>`: static files directory (default `example_motion_controller/dist`)
- `--port <PORT>`: local test server port (default `4174`)
- `--headed`: run Chromium with UI

You can point `--dist-dir` at the React parity builds as well:

- `example_motion_controller_react/dist` (`motion/react-m` + `LazyMotion`)
- `example_motion_controller_react/dist-mini` (`useAnimate` from `motion/react-mini`)

Example:

```bash
cargo run -p playwright_regression_controller -- --headed --port 4310
```
