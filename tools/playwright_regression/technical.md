# playwright_regression technical.md

This document explains how the Playwright regression tool is structured and why each check exists.

## Purpose

`playwright_regression` is an internal browser-level guardrail for animation regressions in `leptos_fluid`.

Unit tests in motion/flip verify algorithmic pieces, but this crate validates real browser behavior:

- DOM layout movement
- computed style interpolation during active animation
- settled final state after animation completion

## Runtime architecture

`src/main.rs` has four layers:

1. CLI config parsing (`Config`)
2. lightweight static file server (`StaticServer`)
3. Playwright browser lifecycle
4. regression check execution

## Static server design

The tool serves `example_motion/dist` directly from a small built-in HTTP server.

Why this exists instead of `file://`:

- WASM/module loading is more reliable over HTTP
- mirrors real deployment behavior more closely
- avoids external tooling requirements for serving files

Implementation details:

- non-blocking `TcpListener` loop on localhost
- strict path resolution (`canonicalize` + root-prefix check) to prevent traversal
- minimal content-type mapping for wasm/html/js/css/assets

## Browser lifecycle

The tool uses `playwright-rs` and launches Chromium with configurable headed/headless mode.

The failure messages intentionally include the matching browser install command:

`npx playwright@<PLAYWRIGHT_VERSION> install chromium`

This keeps onboarding friction low for local and CI runs.

## Regression checks

### 1) Motion transition progression

Targets hero card animation.

Assertions:

- start and end styles differ after toggle
- mid-sample differs from start
- mid-sample differs from end

This catches regressions where transitions stop interpolating and snap instantly.

### 2) FLIP single-element transition

Targets `#flip-pill` move-right flow.

Assertions:

- in-flight transform is non-identity during animation
- final layout position moved as expected
- final transform settles back to identity

This catches broken inversion/playback and cleanup failures.

### 3) FLIP group reorder transition

Targets rotate flow in group FLIP demo.

Assertions:

- status reports active animation
- sampled tile has non-identity transform mid-flight
- CSS order for `#mix-a` changes after settle

This catches regressions in keyed matching, group animation execution, and final ordering.

## Sampling strategy

Each check samples at three points:

1. pre-action baseline
2. short mid-animation delay
3. post-settle delay

This gives stronger signal than only asserting final state.

## Selector stability

The tool relies on explicit `data-testid` hooks added in `example_motion` for critical controls/status values (`hero-toggle`, `flip-move-right`, `flip-group-rotate`, `flip-group-status`).

These selectors are part of the test contract and should be preserved when refactoring demo markup.

## Contributor workflow

When adding a new animation regression check:

1. add stable selector hooks in demo markup if needed
2. assert both in-flight behavior and settled behavior
3. keep waits scoped to known animation durations
4. ensure failure messages explain what behavior regressed
