# playwright_regression_controller

Playwright-based animation regression checks for the controller-only demo.

This crate runs end-to-end browser checks against the built `example_motion_controller` app and validates `AnimationController` behavior on plain elements.

## What it tests

Current checks:

1. Builder card transition progression (start -> mid -> settled)
2. `when!` macro state transitions settling on the latest mode
3. `resolver:` retargeting mid-flight: the previous card freezes where it was interrupted, the new card inherits the energized intent
4. Timeline builder pause / resume
5. Timeline macro pause / resume / stop-and-reset
6. Auto height and auto width growth

## Prerequisites

1. Build demo assets

```bash
cd example_motion_controller
trunk build
cd ..
```

2. Install Playwright Chromium browser matching the bundled Playwright version

```bash
npx playwright@1.59.1 install chromium
```

## Run

```bash
cargo run -p playwright_regression_controller --
```

Optional flags:

- `--dist-dir <PATH>`: static files directory (default `example_motion_controller/dist`)
- `--port <PORT>`: local test server port (default `4174`)
- `--headed`: run Chromium with UI

The React parity builds (`example_motion_controller_react/dist`, `dist-mini`) still expose the older `controller-*` test ids, so the current checks do not run against them.

Example:

```bash
cargo run -p playwright_regression_controller -- --headed --port 4310
```
