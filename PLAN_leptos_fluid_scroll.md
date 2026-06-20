# Plan: `leptos_fluid_scroll` — GSAP ScrollTrigger clone

A focused GSAP ScrollTrigger clone that lives in `crates/scroll` and integrates with `crates/motion`. Dependency direction is one-way: `scroll → motion` (never reverse). With default features the crate works standalone in **pure callback mode** (no motion dep).

## Decisions (locked in)

- **Crate name:** `leptos_fluid_scroll` (mirrors `leptos_fluid_flip` / `leptos_fluid_motion`).
- **Scope:** Focused MVP. Pin / snap / markers / batch / horizontal / custom scroller / matchMedia / toggleClass / anticipatePin / fastScrollEnd / preventOverlaps / pinReparent / containerAnimation / invalidateOnRefresh / refreshPriority / normalizeScroll are **deferred** with documented module homes in `technical.md`. No empty Cargo features are declared until each feature ships.
- **Motion integration:** all three modes + pure callback mode.
- **Umbrella:** wired into `leptos_fluid` on day one (features + re-export + README).
- **Example app:** yes, `example_scroll` (trunk-based, mirrors `example_motion`).

## In scope (MVP)

- Trigger element + `start`/`end` position parsing (`"top center"`, `"bottom 80%"`, `"+=300"`, `clamp(...)`, numeric, function).
- Viewport-as-scroller only.
- Reactive `progress` / `direction` / `is_active` signals.
- Callbacks: `onEnter`, `onLeave`, `onEnterBack`, `onLeaveBack`, `onToggle`, `onUpdate`, `onRefresh`.
- `toggleActions` (e.g. `"play pause resume reset"`).
- `scrub: true | Number` (Number = smoothing seconds via rAF catch-up).
- `once: bool`.
- `kill()` / `disable()` / `enable()` / `refresh()` / `scroll(pos)` / `get_velocity()`.
- Auto-refresh on window resize + `document.documentElement` ResizeObserver (reuses `leptos_fluid_web` shared observer).
- Motion integrations (feature-gated):
  - `bind_controller(controller, Fn(f64) → FluidStyle)` — scrub a controller by progress.
  - `bind_timeline(timeline, toggle_actions)` — drive a `FluidTimeline` via toggleActions.
  - `bind_timeline_scrub(timeline, scrub)` — discrete-step MVP (see limitations).
- Typed builder (`ScrollTrigger::builder()`) mirroring `AnimationControllerBuilder` state-marker pattern.
- `scroll_trigger!` declarative macro lowering to the runtime.
- Umbrella `leptos_fluid` wiring.
- `example_scroll` trunk app.
- `crates/scroll/README.md` + `crates/scroll/technical.md`.

## Out of scope (deferred, documented module homes)

- `pin` / `pinSpacing` / `pinReparent` / `anticipatePin` → `pin.rs`
- `snap` → `snap.rs`
- `markers` → `markers.rs`
- `batch` → `batch.rs`
- `horizontal` → handled in `scroller.rs` / `position.rs`
- custom `scroller` / `scrollerProxy` → `scroller.rs`
- `matchMedia` → `match_media.rs`
- `toggleClass`, `preventOverlaps`, `fastScrollEnd`, `containerAnimation`, `invalidateOnRefresh`, `refreshPriority` / `sort`, `normalizeScroll`, ScrollSmoother → documented as future work.

## Crate layout

```
crates/scroll/
  Cargo.toml
  README.md
  technical.md
  src/
    lib.rs                  # feature gating, exports, prelude (mirrors motion/src/lib.rs)
    config.rs               # ScrollTriggerConfig, Scrub, ToggleActions, Callbacks
    position.rs             # start/end string parsing + pixel resolution
    scroller.rs             # scroller abstraction (MVP: viewport; slot for custom later)
    engine.rs               # shared scroll/resize listener, rAF batched update loop, refresh
    callbacks.rs            # ScrollTriggerEvent { progress, direction, is_active, velocity }
    toggle.rs               # toggleActions parsing + 4-phase state machine
    trigger.rs              # ScrollTrigger handle, inner, reactive signals, lifecycle
    controller_binding.rs   # cfg(controller)  — bind_controller scrub
    timeline_binding.rs     # cfg(timeline)   — bind_timeline + bind_timeline_scrub
    builders.rs             # cfg(builders)   — typed builder
    macros.rs               # cfg(macros)     — scroll_trigger! macro
    macro_support.rs        # cfg(any(builders, macros))
  tests/
    position_tests.rs
    toggle_tests.rs
    config_tests.rs
```

## Feature flags (`crates/scroll/Cargo.toml`)

```toml
[features]
default = []
full = ["controller", "timeline", "builders", "macros"]
# Pure callback mode is always available with default-features = false.
controller = ["dep:leptos_fluid_motion", "leptos_fluid_motion/controller"]
timeline   = ["dep:leptos_fluid_motion", "leptos_fluid_motion/timeline", "leptos_fluid_motion/controller"]
builders   = []
macros     = ["builders"]
```

`timeline` pulls in `controller` because `FluidTimeline::bind` needs an `AnimationController`, and `bind_timeline_scrub` drives the controller directly. `macros` pulls in `builders` (same pattern as motion).

## Core runtime design

### Position resolution (`position.rs`)
`resolve_start(trigger_rect, scroller_viewport, start_point)`:
- "top center" = trigger.top vs viewport.height * 0.5 → scroll pos = trigger_top - viewport_center.
- "bottom 80%" = trigger.bottom vs viewport.height * 0.8.
- "+=300" = start + 300 (relative to start for `end`).
- `clamp(...)` wraps the computed value to `[0, max_scroll]`.
All math is pure f64; fully unit-testable without a browser.

### Engine update loop (`engine.rs`)
- Single `scroll` listener on window → sets `scroll_pending = true`, schedules one rAF.
- rAF tick: for each registered trigger, read scroll pos, compute raw progress `(scroll - start)/(end - start)` clamped to `[0,1]`. Compare to previous → direction. Determine active region. Dispatch `onUpdate` (always when progress changed), `onEnter`/`onLeave`/`onEnterBack`/`onLeaveBack`/`onToggle` per phase transitions. For `scrub: Number`, push target progress into the trigger's scrub state and run a smoothing sub-loop (separate rAF until caught up within epsilon, then stop). For `scrub: true`/`false`, set progress directly. For `once`, kill after first `onLeave` (forward, end reached).
- Resize listener (debounced 200ms) + `ResizeObserver` on `document.documentElement` (via `leptos_fluid_web::observe_resize`) → `refresh_all`: recompute start/end for every trigger from fresh geometry, re-evaluate progress.
- Velocity: rolling window of `(scroll_delta, time_delta)` over last ~100ms.

### Scrub smoothing (`engine.rs`)
For `scrub: Number(t)`:
- `current` eased toward `target` each rAF: `current += (target - current) * (1 - exp(-dt / t))`. Stop when `|target - current| < 1e-4`.
- The smoothed `current` is what `progress()` exposes and what `bind_controller` reads. Smoothing has one implementation; all bindings inherit it.
- For `scrub: true`: `current = target` (no smoothing). For `scrub: false`: progress is not exposed as a continuous signal (only callback events fire; `progress()` still reflects raw clamped value for inspection).

### Callbacks (`callbacks.rs`)
`ScrollTriggerEvent` is passed to each callback. Callbacks are `Rc<dyn Fn(ScrollTriggerEvent)>` or Leptos `Callback` where it fits the reactive owner. Stored in `ScrollTriggerInner`. Dispatch happens in engine tick.

### toggleActions state machine (`toggle.rs` + `timeline_binding.rs`)
- 4 phases:
  - forward, crossing start into active → `onEnter`
  - forward, crossing end out of active → `onLeave`
  - backward, crossing end into active → `onEnterBack`
  - backward, crossing start out of active → `onLeaveBack`
- `bind_timeline` translates the 4 actions to `FluidTimeline` calls: play→`play`, pause→`pause`, resume→`resume`, reset→`stop()`+`set_immediate(initial)`, restart→`restart`, complete→`set_immediate(final step style)`, reverse→`set_immediate(initial)` (limitation: no reverse, documented), none→no-op.

### `bind_timeline_scrub` (`timeline_binding.rs`) — discrete MVP
`floor(progress * steps.len()).min(steps.len() - 1)` → step index → `timeline.set_immediate(step.style)`. **Documented limitation:** `FluidTimeline` is step-index based with `wait_ms` per step, not a continuous time-based timeline, and `FluidStyle` has no built-in lerp. Continuous interpolated scrubbing is deferred until either (a) `FluidStyle` gains an interpolation helper, or (b) a virtual-time model is added to `FluidTimeline`. Users wanting smooth scrubbing today should use `bind_controller(|p| style_fn(p))`.

## Builder & macro design

```rust
ScrollTrigger::builder()
    .trigger(node_ref)
    .start("top center")
    .end("bottom 80%")
    .scrub(true)                 // or .scrub(0.5) for smoothing
    .toggle_actions("play none none none")
    .once(false)
    .on_enter(|ev| log::info!("enter {}", ev.progress))
    .bind_controller(controller, move |p| FluidStyle::new().opacity(p))   // cfg(controller)
    .install();
```

`scroll_trigger!` lowers to the same builder. Follows the exact parse/finish TT-muncher shape of `controller!` / `timeline!` (see `crates/motion/src/macros.rs`).

## Web crate dependency & new helpers

- Reuse `leptos_fluid_web::observe_resize` (shared ResizeObserver) for `document.documentElement` refresh triggers. No new web features needed for MVP.
- Scroll/resize listener wiring, `window.scroll_y()`, `Element::get_bounding_client_rect()`, `window.request_animation_frame()` are used directly via `web-sys` inside `scroller.rs` / `engine.rs`. No new `leptos_fluid_web` feature flag required.
- (Future, when custom scroller lands: add a `scroll` feature to `leptos_fluid_web` for `scrollerProxy` helpers. Not now.)

## Umbrella wiring (`/Cargo.toml`)

Root package `[dependencies]`:
```toml
leptos_fluid_scroll = { workspace = true, optional = true }
```
Root package `[features]` (added alongside existing motion-* features):
```toml
scroll            = ["dep:leptos_fluid_scroll"]
scroll-controller = ["scroll", "leptos_fluid_scroll/controller"]
scroll-timeline   = ["scroll", "leptos_fluid_scroll/timeline"]
scroll-builders   = ["scroll", "leptos_fluid_scroll/builders"]
scroll-macros    = ["scroll-builders", "leptos_fluid_scroll/macros"]
scroll-full      = ["scroll-controller", "scroll-timeline", "scroll-builders", "scroll-macros"]
full             = ["flip", "view-transitions", "motion-full", "scroll-full"]   # updated
```

Re-export under `leptos_fluid::scroll::*` (mirrors how motion / flip / view_transitions are exposed — see root `src/`).

## Example app `example_scroll`

Mirror `example_motion/`:
- `Cargo.toml` (workspace deps: `leptos_fluid_scroll` with `["controller","timeline","builders","macros"]`, plus `leptos_fluid_motion` for `FluidStyle`/`AnimationController`/`FluidTimeline`), `Trunk.toml` (copy `minify = "never"`), `index.html`, `style.css`.
- `src/main.rs`, `src/app.rs`, `src/components/{mod,hero,scrub_card,timeline_toggle,timeline_scrub,once_reveal,pure_callback,footer}.rs`.
- Add `"example_scroll"` to workspace members.
- Add a row to root README's "Examples in this repo" section.

Each demo section is a self-contained `#[component]` using a `NodeRef` + `ScrollTrigger::builder()`.

## Tests

- **Pure-logic unit tests** (run on host, no browser) — the primary safety net for MVP, mirroring how `motion` tests `transition_css`/`split_animation_props`/`parse_*`:
  - `position.rs`: `parse_point`, `parse_start_end`, `resolve_start/end`, `clamp`, `"+=N"` relative, all keyword/percent/pixel combos.
  - `toggle.rs`: `parse_toggle_actions` for all 8 action keywords, default, invalid → error, 4-phase mapping.
  - `config.rs`: `ScrollTriggerConfig` defaults, `Scrub` default, builder method chaining.
  - `callbacks.rs`: `ScrollTriggerEvent` construction + velocity computation from a synthetic delta window.
- **Reactive tests** using `any_spawner::Executor::init_futures_executor()` + `Owner::new().with(|| ...)` (same pattern as `crates/motion/src/macro_support.rs:32`) for any `Effect`-based binding logic that doesn't touch the DOM.
- **Runtime validation:** `example_scroll` + manual/Playwright. Adding `tools/playwright_regression_scroll` is **optional** (noted as a follow-up) to keep MVP focused; the existing two regression tools show the pattern if you want it later.
- **No new lint/typecheck commands** — workspace uses `cargo build`/`cargo test`; no `ruff`/`clippy` config exists to wire into. The repo has a `.ruff_cache` but that's for the Python-based Playwright tooling, not Rust.

## Open decisions deferred to implementation

- Whether `ScrollTriggerEvent` callbacks are `Rc<dyn Fn>` or Leptos `Callback<ScrollTriggerEvent>` — match `motion`'s `Callback<()>` usage in `FluidTimeline` (Leptos `Callback`) for consistency, falling back to `Rc<dyn Fn>` where the reactive owner isn't required.
- Exact `scroll_trigger!` macro keyword set — mirror `timeline!`'s `field: value` style; final keyword list settled during phase 5.
- Whether to add `tools/playwright_regression_scroll` — **not in MVP**; flagged as follow-up.

---

# Checklist

## Phase 1 — Skeleton

- [ ] Create `crates/scroll/` directory.
- [ ] Write `crates/scroll/Cargo.toml` (package metadata + feature flags + deps per §"Feature flags").
- [ ] Write `crates/scroll/src/lib.rs` with feature gating + empty exports + `prelude` module (mirror `motion/src/lib.rs`).
- [ ] Write `crates/scroll/README.md` stub (install + feature split placeholders).
- [ ] Write `crates/scroll/technical.md` stub (module map + deferred-features module homes).
- [ ] Add `"crates/scroll"` to workspace `members` in root `Cargo.toml`.
- [ ] Add `leptos_fluid_scroll = { version = "0.1.1", path = "crates/scroll" }` to root `[workspace.dependencies]`.
- [ ] Run `cargo build -p leptos_fluid_scroll` and confirm it is green.

## Phase 2 — Pure-logic core

- [ ] Implement `src/config.rs` (`ScrollTriggerConfig`, `Scrub`, `ToggleActions`, callback slot types).
- [ ] Implement `src/position.rs` (`ScrollPoint`, `parse_point`, `parse_start_end`, `resolve_start/end`, `clamp`, `"+=N"` relative).
- [ ] Implement `src/toggle.rs` (`Action` enum, `parse_toggle_actions`, 4-phase mapping).
- [ ] Implement `src/callbacks.rs` (`ScrollTriggerEvent` + velocity computation from synthetic delta window).
- [ ] Add `tests/position_tests.rs` covering all keyword/percent/pixel/relative/clamp combos.
- [ ] Add `tests/toggle_tests.rs` covering all 8 action keywords + default + invalid.
- [ ] Add `tests/config_tests.rs` covering defaults, `Scrub` default, builder chaining.
- [ ] Wire the new modules into `src/lib.rs` exports (no-std-cfg pure paths).
- [ ] Run `cargo test -p leptos_fluid_scroll` and confirm all unit tests pass.

## Phase 3 — Runtime (pure callback mode)

- [ ] Implement `src/scroller.rs` (`Scroller::Viewport`, `scroll_position`, `max_scroll`, `viewport_size`, `on_scroll`, `on_resize`).
- [ ] Implement `src/engine.rs` (`SharedScrollEngine`, single scroll listener, rAF-batched tick, resize → `refresh_all`, ResizeObserver on `document.documentElement` via `leptos_fluid_web::observe_resize`, velocity rolling window).
- [ ] Implement `src/trigger.rs` (`ScrollTrigger` `#[derive(Clone, Copy)]` handle + `ScrollTriggerInner`, `create`, `kill/disable/enable/refresh/scroll/get_velocity`, reactive `progress() / direction() / is_active()`, `start() / end()`, auto `on_cleanup(kill)` on create).
- [ ] Wire callbacks dispatch in engine tick (`onEnter/Leave/EnterBack/LeaveBack/Toggle/Update/Refresh`).
- [ ] Implement scrub smoothing sub-loop for `scrub: Number`.
- [ ] Implement `once` kill-after-leave semantics.
- [ ] Wire engine + trigger + scroller + config into `src/lib.rs`.
- [ ] Run `cargo build -p leptos_fluid_scroll` green.
- [ ] Build a manual smoke check via `example_scroll`'s `pure_callback` section (phase 7 dependency — write a throwaway check or wait for phase 7).

## Phase 4 — Motion integration

- [ ] Implement `src/controller_binding.rs` (`cfg(controller)`) — `bind_controller(controller, Fn(f64) → FluidStyle, Option<Transition>)` reading `progress()` and calling `controller.animate_with` / `set_immediate`.
- [ ] Implement `src/timeline_binding.rs` (`cfg(timeline)`) — `bind_timeline(timeline, &str toggle_actions)` mapping 4-phase actions to `FluidTimeline` methods; document reverse limitation.
- [ ] Implement `bind_timeline_scrub(timeline, Scrub)` discrete-step MVP (`floor(progress * steps.len()).min(steps.len()-1)` → `set_immediate(step.style)`); document continuous-interpolation limitation.
- [ ] Wire `controller` / `timeline` features + re-exports in `src/lib.rs` and `prelude`.
- [ ] Run `cargo build -p leptos_fluid_scroll --features controller` and `--features timeline` green.
- [ ] Add reactive unit tests for binding logic where DOM-free (use `any_spawner` + `Owner` pattern).

## Phase 5 — Ergonomics (builders + macros)

- [ ] Implement `src/builders.rs` (`cfg(builders)`) — `ScrollTriggerBuilder<State>` with `NeedsTrigger` / `Ready` state markers; methods `trigger / start / end / scrub / toggle_actions / once / on_enter / ... / bind_controller / bind_timeline` (cfg-gated); `install()` only on `Ready`.
- [ ] Implement `src/macro_support.rs` (`cfg(any(builders, macros))`) — shared scroll helpers (e.g. `watch_progress`).
- [ ] Implement `src/macros.rs` (`cfg(macros)`) — `scroll_trigger!` TT-muncher parse/finish mirroring `controller!` / `timeline!`.
- [ ] Wire `builders` / `macros` features + re-exports in `src/lib.rs` and `prelude`.
- [ ] Run `cargo build -p leptos_fluid_scroll --features macros` green.
- [ ] Add doc-test-style snippets in `README.md` (compile-checked where feasible via `cargo test --doc`).

## Phase 6 — Umbrella wiring

- [ ] Add `leptos_fluid_scroll = { workspace = true, optional = true }` to root package `[dependencies]`.
- [ ] Add `scroll`, `scroll-controller`, `scroll-timeline`, `scroll-builders`, `scroll-macros`, `scroll-full` features to root package `[features]`.
- [ ] Update `full` to include `scroll-full`.
- [ ] Add `leptos_fluid::scroll::*` re-export in root `src/` (mirror motion/flip/view_transitions exposure).
- [ ] Run `cargo build -p leptos_fluid --features scroll-full` green.
- [ ] Run `cargo build -p leptos_fluid --features full` green.

## Phase 7 — Example app `example_scroll`

- [ ] Create `example_scroll/` with `Cargo.toml` (deps: `leptos_fluid_scroll` + `leptos_fluid_motion`, `console_error_panic_hook`), `index.html`, `style.css`, `Trunk.toml` (`minify = "never"`).
- [ ] Add `"example_scroll"` to workspace `members`.
- [ ] Write `src/main.rs` (mirror `example_motion/src/main.rs`).
- [ ] Write `src/app.rs` mounting all demo sections.
- [ ] Write `src/components/mod.rs` exporting all sections.
- [ ] Write `src/components/hero.rs` (intro / feature summary).
- [ ] Write `src/components/scrub_card.rs` (scrub an `AnimationController` by progress).
- [ ] Write `src/components/timeline_toggle.rs` (drive `FluidTimeline` via toggleActions).
- [ ] Write `src/components/timeline_scrub.rs` (discrete-step scrub of a `FluidTimeline`).
- [ ] Write `src/components/once_reveal.rs` (one-shot on-enter animation).
- [ ] Write `src/components/pure_callback.rs` (standalone `ScrollTrigger::create` with only callbacks).
- [ ] Write `src/components/footer.rs`.
- [ ] Run `cargo build -p leptos_fluid_scroll_example` green (or chosen package name).
- [ ] Optionally run `trunk serve` manually and sanity-check the demos.

## Phase 8 — Docs polish

- [ ] Fill `crates/scroll/README.md`: install (umbrella + direct), feature split table, quick start for each integration mode, `scroll_trigger!` example, limitations section.
- [ ] Fill `crates/scroll/technical.md`: module map, core runtime model, position resolution algorithm, engine update loop, scrub smoothing math, refresh strategy, lifecycle, toggleActions mapping table, motion integration details, deferred-features module homes.
- [ ] Update root `README.md`: ToC entry, feature matrix rows (`scroll` / `scroll-controller` / `scroll-timeline` / `scroll-builders` / `scroll-macros` / `scroll-full`).
- [ ] Add "Scroll (ScrollTrigger) API guide" section to root README with code samples.
- [ ] Update "Choosing the right tool" note in root README (scroll = scroll-linked; motion = interaction/state-driven; compose via `bind_controller`).
- [ ] Add `example_scroll` row to "Examples in this repo" section.

## Phase 9 — Final validation

- [ ] `cargo build -p leptos_fluid_scroll --all-features` green.
- [ ] `cargo test -p leptos_fluid_scroll --all-features` green.
- [ ] `cargo build -p leptos_fluid --features full` green.
- [ ] `cargo build -p leptos_fluid_scroll_example` green.
- [ ] Spot-check `crates/scroll/README.md` code samples against the actual API.
- [ ] Optional follow-up: add `tools/playwright_regression_scroll` (mirrors existing regression tools) — **not in MVP**.