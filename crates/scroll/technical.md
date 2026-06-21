# leptos_fluid_scroll technical.md

This document describes how `leptos_fluid_scroll` is organized internally so contributors can reason about behavior changes safely.

## Module map

- `src/lib.rs`: export surface and prelude
- `src/config.rs`: `ScrollTriggerConfig` (start/end/scrub/toggle/once/callbacks), `Scrub`, `ToggleActions`
- `src/position.rs`: pure start/end position resolution (`"top center"`, percentages, `"+=N"`, `clamp(...)`)
- `src/scroller.rs`: scroll source abstraction (`Scroller::Viewport`; custom scroller deferred)
- `src/engine.rs`: shared `ScrollEngine` thread-local singleton that batches scroll/resize updates via rAF
- `src/callbacks.rs`: `ScrollTriggerEvent` payload, `ScrollCallback` slot type, and pure `VelocityTracker`
- `src/toggle.rs`: `toggleActions` parsing (`Action`, `TogglePhase`, `ScrollDirection`)
- `src/trigger.rs`: `ScrollTrigger` core handle and lifecycle (create/kill/disable/enable/refresh), `TriggerTargetSource`
- `src/controller_binding.rs`: feature `controller` - bind a trigger to a `leptos_fluid_motion::AnimationController`
- `src/timeline_binding.rs`: feature `timeline` - drive a `leptos_fluid_motion::FluidTimeline` from scroll progress
- `src/builders.rs`: feature `builders` - typed builder layer over `ScrollTrigger`
- `src/macros.rs`: feature `macros` - `scroll_trigger!` declarative macro (implies `builders`)
- `src/macro_support.rs`: helper runtime used by macro-generated effects (features `builders` or `macros`)

## Core runtime model

`ScrollTrigger` is the core runtime handle. It mirrors `AnimationController` in `crates/motion/src/controller.rs`: a `#[derive(Clone, Copy)]` handle wrapping `StoredValue<ScrollTriggerInner, LocalStorage>`, with `RwSignal`s for reactive outputs and `StoredValue<..., LocalStorage>` for non-reactive interior state.

### `ScrollTrigger` handle structure

`ScrollTriggerInner` holds:

- `config: ScrollTriggerConfig` - the tunable parameters (start/end strings, scrub, toggleActions, once, callbacks)
- `scroller: Scroller` - the scroll source (viewport only in MVP)
- `target_source: StoredValue<Option<TriggerTarget>, LocalStorage>` - either a concrete `Element` or a `Rc<dyn Fn() -> Option<Element>>` resolver
- `start_pixels` / `end_pixels: StoredValue<f64, LocalStorage>` - resolved pixel positions, recomputed by `refresh()`
- `progress: RwSignal<f64>` - clamped `[0.0, 1.0]` progress exposed to consumers
- `direction: RwSignal<i8>` - `1` forward, `-1` backward, `0` initial
- `is_active: RwSignal<bool>` - whether the scroll position is within `[start_px, end_px]`
- `velocity: RwSignal<f64>` - scroll velocity in pixels per second
- `scrub_current` / `scrub_target` / `scrub_last_ms: StoredValue<..., LocalStorage>` - smoothing state for `Scrub::Number`
- `registration_id: StoredValue<Option<u32>, LocalStorage>` - the engine-assigned id used by `kill()`
- `enabled` / `killed: StoredValue<bool, LocalStorage>` - lifecycle flags
- `velocity_tracker: StoredValue<VelocityTracker, LocalStorage>` - the per-trigger rolling-window velocity estimator
- `prev_active` / `prev_progress: StoredValue<..., LocalStorage>` - previous-frame state for phase/progress transition detection

`TriggerTarget` is a private enum: `Element(Element)` for concrete elements, `Resolver(Rc<dyn Fn() -> Option<Element>>)` for `NodeRef` and dynamic lookup. `attach_node_ref` lowers to `attach_resolver` via `node_ref.get_untracked()`.

### Engine thread-local singleton

The shared `ScrollEngine` (in `engine.rs`) is backed by three thread-locals:
- `SHARED_ENGINE: RefCell<Option<SharedScrollEngine>>` - the engine slot
- `ENGINE_OUT: Cell<bool>` - flag set while the engine is taken out of the slot during `tick`/`refresh_all` so callbacks that re-enter `register`/`unregister` do not double-borrow the `RefCell`
- `PENDING_REGISTERS: RefCell<Vec<ScrollTrigger>>` - queue of triggers registered during a tick/refresh; merged back into the engine after the loop

The engine batches all registered triggers on the same scroller (viewport in MVP) through a single scroll listener, a single resize listener, and a single `requestAnimationFrame` callback. This avoids per-trigger scroll listeners and lets the engine evaluate every trigger together each frame.

The engine installs listeners on first `register`:

- `Scroller::on_scroll` -> `schedule_tick` (sets `scroll_pending`, schedules a rAF if one is not already in flight)
- `Scroller::on_resize` -> `schedule_resize` (debounced: ignored if the last resize was < 200ms ago; otherwise schedules a rAF that runs `refresh_all`)
- `observe_resize` on `document.documentElement` -> `schedule_resize` (only when the `resize-observer` feature is enabled; so element-level layout changes also trigger refresh)

`schedule_tick` and `schedule_resize` take the engine OUT of the `SHARED_ENGINE` slot (`slot.borrow_mut().take()`), set `ENGINE_OUT = true`, run `tick()`/`refresh_all()`, drain `PENDING_REGISTERS` into the engine, then restore the engine and clear `ENGINE_OUT`. This take-out pattern lets callbacks invoked during `tick`/`refresh_all` re-enter `register`/`unregister` (which hit the slot) without a `RefCell` double-borrow panic. `register` during the loop queues into `PENDING_REGISTERS` (returns id `0`); `unregister` during the loop is a no-op (the `killed` flag + the post-loop `!is_killed()` filter drops it).

`tick()` clears `raf_scheduled` and `scroll_pending`, samples the current scroll position and velocity via its own `VelocityTracker`, then iterates every registered trigger and calls `trigger.engine_update(scroll_pos, velocity, now)`. Killed triggers are filtered out after the loop.

`refresh_all()` calls `trigger.refresh()` on every registered trigger (re-resolves start/end pixels from current geometry and re-evaluates progress).

### Position resolution algorithm

`resolve_start(trigger_rect, scroller_size, position)` is the core GSAP formula: `scroll_position = trigger_point_pixels - scroller_point_pixels`, where:

- `trigger_point_pixels = position.trigger.resolve(trigger_rect)` (`top` -> `rect.start`, `bottom` -> `rect.start + rect.size`, `center` -> `rect.start + rect.size * 0.5`, `Percent(p)` -> `rect.start + rect.size * p`, `Pixels(px)` -> `rect.start + px`)
- `scroller_point_pixels = position.scroller.resolve(scroller_size)` (absolute points resolve against the viewport; `Relative { pixels, percent_of_scroller }` resolves against the viewport size, with `percent_of_scroller` toggling between raw pixels and `scroller_size * (pixels / 100.0)`)

For `end`, `resolve_end_pixels` (in `trigger.rs`) handles the relative `"+=N"` / `"-=N"` form: when the end position is `Relative`, it adds the delta to the resolved `start_px` (in pixels or as a percentage of the scroller size) rather than re-running `resolve_start`. Absolute end positions fall through to `resolve_start` with the trigger rect and viewport size.

`parse_start_end` (in `position.rs`) detects the relative form by checking for the `+=` / `-=` prefix on the `end` string and synthesizes a `ScrollPosition { trigger: Top, scroller: Relative { ... } }` so the engine can apply it.

### Scrub smoothing math

For `Scrub::Number(t)` the trigger eases `scrub_current` toward `scrub_target` (the latest raw clamped progress) on each engine rAF tick using the exponential formula:

```
next = current + (target - current) * (1 - exp(-dt / t))
```

where `dt` is the seconds elapsed since the previous smoothing step. Smoothing snaps to the target when `|target - current| < 1e-4`. The smoothing step runs inside the main engine rAF `tick` (one step per frame) rather than a separate sub-loop; this is simpler and sufficient for 60fps.

`Scrub::Bool(true)` exposes raw progress with no smoothing (direct-link). `Scrub::Bool(false)` exposes raw progress and drives behavior through callbacks only - `progress()` still updates as the user scrolls, so controller/timeline bindings still work, but the trigger never smooths.

### Lifecycle

- `ScrollTrigger::create(config, target)` / `ScrollTrigger::new(...)` - entry point. Calls `with_config(config)` (registers with the engine, installs `on_cleanup`), then `target.attach_to(self)`, then `refresh()`.
- `ScrollTrigger::with_config(config)` - `pub(crate)` builder entry point. Builds the inner, calls `engine::register(self)` to get a `registration_id`, installs `on_cleanup(move || trigger.kill())` so the trigger is unregistered when its reactive owner scope dies.
- `attach_element` / `attach_node_ref` / `attach_resolver` - set `target_source`. `NodeRef` lowers to a resolver that calls `node_ref.get_untracked()`.
- `refresh()` - recomputes `start_px` / `end_px` from current geometry via `resolve_start` / `resolve_end_pixels`, recomputes `progress` and `is_active` from the current scroll position, dispatches `on_refresh`.
- `kill()` - idempotent. Sets `killed = true`, `enabled = false`, calls `engine::unregister(id)`.
- `disable()` / `enable()` - flip `enabled`. `enable()` also runs `refresh()` so geometry is recomputed after a disable gap. The engine skips disabled triggers in `tick()`.
- `scroll_position()` / `get_velocity()` - read-through accessors to the scroller's current state and the trigger's `VelocityTracker`.
- `engine_update(scroll_pos, velocity, now_ms)` - `pub(crate)`, called by the engine on each rAF tick. Skips if killed or disabled. Computes raw progress, detects phase transitions, dispatches callbacks, updates reactive signals, steps scrub smoothing, and handles `once` auto-kill.

### toggleActions 4-phase state machine

Each `engine_update` computes `active = start_px <= scroll_pos && scroll_pos <= end_px` and `direction = sign(clamped - prev_progress)`, falling back to the previous direction when `clamped == prev_progress`. When `active != prev_active`, the engine computes the `TogglePhase` from `(prev_active, active, direction)`:

| prev_active | active | direction | phase |
| --- | --- | --- | --- |
| false | true | 1 | `OnEnter` |
| true | false | 1 | `OnLeave` |
| false | true | -1 | `OnEnterBack` |
| true | false | -1 | `OnLeaveBack` |

`OnEnter` / `OnLeave` / `OnEnterBack` / `OnLeaveBack` dispatch the corresponding callback (`on_enter`, `on_leave`, `on_enter_back`, `on_leave_back`). `on_toggle` also fires on every active transition. `on_update` fires whenever `progress_changed` (clamped progress differs from `prev_progress`). `toggleActions` itself is not used by the bare trigger - it is consumed by `bind_timeline` (see Motion integration below).

`once: true` triggers auto-kill after the forward `OnLeave` (active false, direction 1), so the trigger fires `on_enter` once on first forward entry and never again.

## Feature split

The crate is feature-split for wasm-size-sensitive builds:

- `controller`: `bind_controller`, `bind_controller_with`
- `timeline`: `bind_timeline`, `bind_timeline_scrub`
- `builders`: `ScrollTrigger::builder()`, `ScrollTriggerBuilder<State>`, `ReadyScrollTriggerBuilder`
- `macros`: `scroll_trigger!` (implies `builders`)
- `resize-observer`: element-resize auto-refresh via `leptos_fluid_web` ResizeObserver on `document.documentElement` (opt-in; viewport `window.on_resize` refresh is always on)
- `full`: convenience aggregate of all of the above (including `resize-observer`)

The pure-callback mode (`default-features = false`) has no `leptos_fluid_motion` dependency and exposes only the callback/progress surface. The umbrella crate forwards these as `scroll-controller`, `scroll-timeline`, `scroll-builders`, `scroll-macros`, `scroll-full` (the umbrella `scroll-full` does NOT forward `leptos_fluid_scroll/full` and so does NOT include `resize-observer`; to enable element-resize auto-refresh via the umbrella, users must add `leptos_fluid_scroll/resize-observer` directly, or depend on `leptos_fluid_scroll` with `features = ["full"]`).

Each engine update follows this shape:

1. compute raw progress `(scroll_pos - start_px) / (end_px - start_px)`
2. clamp to `[0.0, 1.0]` and compute `active` range
3. detect `direction` from `clamped vs prev_progress`
4. dispatch phase callbacks (`on_enter` etc.) and `on_toggle` on active transitions
5. dispatch `on_update` on progress changes
6. step scrub smoothing (`Scrub::Number` only) and write the exposed `progress` signal
7. handle `once` auto-kill

## Motion integration

Two feature-gated modules connect `ScrollTrigger` to `leptos_fluid_motion`. Both add methods to `ScrollTrigger` via feature-gated `impl` blocks compiled only when the corresponding feature is on.

### `controller_binding.rs` (feature = "controller")

`ScrollTrigger::bind_controller(controller, style_fn)` and `ScrollTrigger::bind_controller_with(controller, transition, style_fn)` create a Leptos `Effect` that reads `ScrollTrigger::progress()` and dispatches the derived `FluidStyle` to the controller. The pattern mirrors `crates/motion/src/controller.rs` `bind_signal`: an `initialized: StoredValue<bool>` flag applies the first sample immediately via `controller.set_immediate` (no tween, so the controller adopts the current scroll state as its baseline), then subsequent samples animate via `controller.animate` (`bind_controller`) or `controller.animate_with` with a fixed transition override (`bind_controller_with`).

For `scrub: Number`, the scroll engine already smooths `progress()` in `step_scrub` (see `trigger.rs`), so `style_fn` receives the smoothed value and the binding never double-smooths. For `scrub: Bool(false)` (callback-only mode), `progress()` still updates as the user scrolls, so the binding works identically; whether callbacks also fire is independent of the controller binding. The effect's lifetime follows the current reactive owner scope, matching `AnimationController::bind`.

### `timeline_binding.rs` (feature = "timeline")

`ScrollTrigger::bind_timeline(timeline, toggle_actions)` maps the four-phase `toggleActions` string (`"onEnter onLeave onEnterBack onLeaveBack"`) to `FluidTimeline` methods. The binding watches `is_active()` and `direction()`, tracks the previous `is_active` value in a `StoredValue<bool>`, and on each `prev != active` transition determines the `TogglePhase` from `(prev, active, direction)` and dispatches the configured `Action` via `apply_timeline_action`.

`ScrollTrigger::bind_timeline_scrub(timeline, step_count, style_fn)` maps scroll `progress()` to a discrete step index `(progress * step_count).floor()` clamped to `step_count - 1` and calls `timeline.set_immediate(style_fn(index, progress))` when the target index changes. `step_count` is supplied by the caller because `FluidTimeline` does not expose its step list for reading (`set_steps` is write-only and `step_index()` returns the running index, not the list length). **Limitation:** `FluidTimeline` is step-index based with `wait_ms` per step, not a continuous time-based timeline, and `FluidStyle` has no built-in lerp, so this binding jumps between steps rather than interpolating. Continuous interpolated scrubbing is deferred until `FluidStyle` gains an interpolation helper. For smooth scrubbing today, use `bind_controller`.

### `Reset` / `Complete` / `Reverse` mapping limitations

`FluidTimeline` has no exact primitive for three `toggleActions` keywords:

- `Reset` maps to `FluidTimeline::stop`. `FluidTimeline` does not expose the initial step style from outside, so the timeline halts at its current position rather than rewinding to the initial state. Callers that need a true rewind should pair this with an explicit `set_immediate(initial_style)`.
- `Complete` maps to `FluidTimeline::play`, letting the sequence run to its final step naturally. `FluidTimeline` has no public "jump to last step" primitive, and reading the last step's style would require access to the step list that the timeline intentionally keeps write-only.
- `Reverse` maps to `FluidTimeline::stop`. `FluidTimeline` has no reverse primitive. For progress-controlled scrubbing in both directions, use `bind_timeline_scrub`.

## Builders and macros

### `builders.rs` (feature = "builders")

`ScrollTriggerBuilder<State>` mirrors `AnimationControllerBuilder<State>` in `crates/motion/src/builders.rs`: a `PhantomData<State>`-parameterized struct with two state markers, `ScrollTriggerBuilderNeedsTarget` and `ScrollTriggerBuilderReady`. The builder starts in `NeedsTarget`; calling `.target(t)` or `.resolver(f)` attaches a deferred target installer and transitions to `ReadyScrollTriggerBuilder`.

Config setters (`start`, `end`, `scrub`, `toggle_actions`, `once`, `id`) and callback setters (`on_enter` through `on_refresh`) are available in any state and set the underlying `ScrollTriggerConfig` fields directly via the internal `map` helper. Motion bindings (`bind_controller`, `bind_controller_with`, `bind_timeline`, `bind_timeline_scrub`) are feature-gated `impl` blocks that store a `Box<dyn FnOnce(&ScrollTrigger)>` installer.

`ReadyScrollTriggerBuilder::install` finalizes the trigger via `ScrollTrigger::with_config` (the `pub(crate)` refactor of `create` that builds the inner, registers with the engine, and installs `on_cleanup` without attaching a target), then runs the deferred target installer, any motion bindings, and finally `refresh()` so the freshly-attached target's geometry is measured. `ScrollTrigger::create` is preserved unchanged for backward compatibility: it now delegates to `with_config` + `target.attach_to` + `refresh`.

### `macros.rs` (feature = "macros")

`scroll_trigger!` is a TT-muncher declarative macro mirroring `controller!` / `timeline!` in `crates/motion/src/macros.rs`. The entry macro seeds an accumulator with `[field unset ()]` slots for every supported field and dispatches to `__fluid_scroll_parse!`, which walks each `field: value,` pair. Each field has a `set`/`unset` pair of arms; duplicate fields, unknown fields, and invalid syntax produce `compile_error!`. The terminal arm dispatches to `__fluid_scroll_finish!`, which requires `trigger:` or `resolver:` (compile_error if neither) and assembles the builder calls.

Supported fields:

- `trigger: $expr` or `resolver: $expr` (exactly one required)
- `start: $expr`, `end: $expr`, `once: $expr`, `id: $expr`
- `scrub: $expr` (lowers via `__fluid_scroll_build_scrub!` and the `ScrubKind` runtime helper in `macro_support.rs`, which dispatches `bool` -> `Scrub::Bool`, numeric -> `Scrub::Number`, and `Scrub` passthrough)
- `toggle_actions: $expr` (lowers via `__fluid_scroll_build_ta!` to `ToggleActions::parse`, falling back to the default)
- `on_enter` / `on_leave` / `on_enter_back` / `on_leave_back` / `on_toggle` / `on_update` / `on_refresh`: `$expr`
- `bind_controller: ($controller, $style_fn)` (feature `controller`)
- `bind_controller_with: ($controller, $transition, $style_fn)` (feature `controller`)
- `bind_timeline: ($timeline, $toggle_actions_str)` (feature `timeline`)
- `bind_timeline_scrub: ($timeline, $step_count, $style_fn)` (feature `timeline`)

The `scrub:` field uses a runtime `ScrubKind` / `ScrubAuto` dispatch helper rather than a pure macro-level match because `macro_rules!` cannot reliably distinguish `true` (a `literal` that is also a valid `expr`) from numeric literals once the value is captured as `$expr`. The `bind_controller`/`bind_timeline` fields do not feature-gate inside the macro; they emit `.bind_controller(...)` etc. which only exist when the corresponding feature is on, producing a method-not-found error otherwise.

### `macro_support.rs` (any(builders, macros))

A thin, doc-hidden home for shared runtime helpers. `watch_progress` mirrors `watch_on_change` from `crates/motion/src/macro_support.rs`: a skip-initial `Effect` that fires `on_change` only when `progress()` actually changes. `ScrubKind` and the `ScrubAuto` trait provide the runtime dispatch used by `scroll_trigger! { scrub: ... }` to accept `true` / `false` / numeric / `Scrub` values from a single `$expr` capture.

## Deferred features

The following GSAP ScrollTrigger features are out of scope for the initial implementation. The table maps each to its planned module home so future contributors know where to land the work.

| Deferred feature | Planned module |
| --- | --- |
| `pin` | `src/pin.rs` |
| `snap` | `src/snap.rs` |
| `markers` | `src/markers.rs` |
| `batch` | `src/batch.rs` |
| horizontal scrolling | `src/scroller.rs`, `src/position.rs` |
| custom scroller element | `src/scroller.rs` |
| `matchMedia` / responsive triggers | `src/match_media.rs` |
| containerAnimation coupling | `src/container_animation.rs` |

## Testing

The crate has 68 unit tests (excluding the timeline-binding wasm-gated tests when not on `wasm32`). Coverage:

- `position.rs`: pure parsing and `resolve_start` tests - host-runnable, no DOM
- `toggle.rs`: `Action` parsing and `toggleActions` 4-token parsing
- `config.rs`: `ScrollTriggerConfig` builder chaining and `parse_positions`
- `callbacks.rs`: `VelocityTracker` rolling-window math
- `trigger.rs`: `raw_progress`, `phase_transition`, `resolve_end_pixels`, `step_scrub` smoothing - host-runnable
- `controller_binding.rs`: `bind_controller` / `bind_controller_with` effect dispatch via `host_test_trigger` - host-runnable
- `timeline_binding.rs`: `apply_timeline_action` and `bind_timeline` / `bind_timeline_scrub` - gated on `target_arch = "wasm32"` because they drive `FluidTimeline` which depends on wasm-only scheduling
- `builders.rs`: typed builder state transitions and field population - host-runnable
- `macro_support.rs`: `watch_progress` skip-initial / dedupe behavior - host-runnable

There is no browser-based test runner (e.g. Playwright) configured for this crate. Runtime behavior should be validated manually via `example_scroll/` (run with `trunk serve`).

Recommended local checks:

```bash
cargo fmt --all
cargo test -p leptos_fluid_scroll --features full
cargo build -p leptos_fluid --features full
```

For runtime behavior, run `example_scroll` and exercise each section (pure callback, once reveal, scrub card, timeline toggle, timeline scrub) across forward and backward scrolling.