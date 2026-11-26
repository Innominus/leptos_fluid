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

Targets `.hero-showcase` after "Energize showcase".

Assertions:

- start and end styles differ after toggle
- mid-sample differs from start and from end
- mid-sample opacity lies strictly between start and end

This catches regressions where transitions stop interpolating and snap instantly.

### 2) FLIP single-element transition

Targets `.flip-workbench-card` after "Dock right" (position-only move).

Assertions:

- in-flight transform is non-identity during animation
- mid-sample visual left lies between start and end
- final layout position moved as expected
- final transform settles back to identity

This catches broken inversion/playback and cleanup failures.

### 3) FLIP border-radius correction

Targets `.flip-workbench-card` after "Panorama" (Standard -> Panorama resize, so the FLIP plays a non-unit scale).

Assertions:

- the card actually grew
- mid-sample transform is scaling
- mid-sample computed radius is counter-scaled away from the resting radius
- computed radius x transform scale stays within 0.5px of the resting radius on both axes
- after settle the transform is identity and the resting radius is restored

This catches regressions where corners visibly stretch during a scaled FLIP, or correction styles leak after cleanup. The demo's card has the same radius in both layouts, so from -> to radius interpolation is not covered here.

### 4) FLIP group reorder transition

Targets all `.flip-board-tile` elements after "Rotate".

Assertions:

- at least two tiles have non-identity transforms mid-flight
- every tile settles back to identity
- CSS order for `#suite-a` changes after settle

This catches regressions in keyed matching, group animation execution, and final ordering.

## Sampling strategy

Each check clicks and samples in one `page.evaluate` (`CLICK_AND_SAMPLE_JS`):

1. pre-action baseline
2. short mid-animation delay
3. post-settle delay

Delays are measured in-page from the click, so Playwright/CDP round-trips cannot push the mid sample past the end of a 240-260ms animation. Each sample runs in a rAF scheduled from a task, so it lands after the library's own per-frame rAF corrections and reads the same animation time they used.

## Selector stability

`example_motion` has no `data-testid` hooks. The tool clicks buttons by exact visible label ("Energize showcase", "Dock right", "Panorama", "Rotate"; each must match exactly one button) and samples by demo classes/ids (`.hero-showcase`, `.flip-workbench-card`, `.flip-board-tile`, `#suite-a`).

These labels and selectors are part of the test contract; update the tool when demo copy or markup changes.

## Contributor workflow

When adding a new animation regression check:

1. add stable selector hooks in demo markup if needed
2. assert both in-flight behavior and settled behavior
3. keep waits scoped to known animation durations
4. ensure failure messages explain what behavior regressed
