# playwright_regression_trendy

Playwright-based scroll animation regression checks for `leptos_fluid`.

This crate runs end-to-end browser checks against the built `example_trendy` demo and verifies scroll-driven animation behavior in ways that catch common regressions:

- sticky parallax layers must move with scroll progress
- horizontal galleries must translate as the section scrolls
- one-shot entrance reveals (text mask, stagger grid, counter, image) must animate when scrolled into view
- scrubbed bindings (perspective tilt, color morph) must interpolate with scroll
- velocity-driven marquee must keep moving while active
- magnetic CTA must react to hover and settle back on leave

## What it tests

Current checks (12):

1. All sections render (`run_all_sections_render_check`) — all 10 sections + footer present and visible
2. Sticky hero parallax (`run_sticky_hero_check`) — hero title transform changes with scroll
3. Horizontal gallery (`run_horizontal_gallery_check`) — gallery track translates horizontally
4. Text mask reveal (`run_text_mask_reveal_check`) — text lines animate on enter
5. Stagger grid (`run_stagger_grid_check`) — cards fade/lift/scale in with stagger
6. Counter (`run_counter_check`) — number counts up from 0 on enter
7. Image reveal (`run_image_reveal_check`) — clip-path opens and label opacity increases
8. Perspective tilt (`run_perspective_tilt_check`) — card 3D transform changes with scroll
9. Velocity marquee (`run_velocity_marquee_check`) — marquee track keeps moving via rAF
10. Color morph (`run_color_morph_check`) — block background-color interpolates with scroll
11. Magnetic CTA (`run_magnetic_cta_check`) — wrap moves on hover and settles back on leave
12. Scroll restoration (`run_scroll_restoration_check`) — hero parallax resets when scrolling back to top

## Prerequisites

1. Build demo assets

```bash
cd example_trendy
trunk build
cd ..
```

2. Install Playwright Chromium browser matching the bundled Playwright version

```bash
npx playwright@1.56.1 install chromium
```

## Run

```bash
cargo run -p playwright_regression_trendy --
```

Optional flags:

- `--dist-dir <PATH>`: static files directory (default `example_trendy/dist`)
- `--port <PORT>`: local test server port (default `4175`)
- `--headed`: run Chromium with UI

Example:

```bash
cargo run -p playwright_regression_trendy -- --headed --port 4300
```

## CI recommendation

Run this tool as a post-build browser regression gate for scroll-animation-related changes. A typical CI job sequence:

1. build `example_trendy` static output
2. install Playwright Chromium
3. run `cargo run -p playwright_regression_trendy --`

If any regression check fails, the process exits non-zero and prints a targeted failure reason.