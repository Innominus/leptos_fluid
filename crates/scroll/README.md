# leptos_fluid_scroll

Scroll-triggered animations for Leptos (CSR): a focused [GSAP ScrollTrigger](https://gsap.com/docs/v3/Plugins/ScrollTrigger/) clone that integrates with `leptos_fluid_motion`. With `default-features = false` the crate ships a pure-callback mode that has no dependency on `leptos_fluid_motion`; opting into the `controller`, `timeline`, `builders`, or `macros` features pulls in motion integrations.

## Install

Via the umbrella crate:

```toml
[dependencies]
leptos_fluid = { version = "0.1", features = ["scroll"] }
# Add more umbrella scroll features as needed:
# features = ["scroll", "scroll-controller", "scroll-builders"]
# features = ["scroll-controller", "scroll-timeline", "scroll-builders", "scroll-macros"]
# features = ["scroll-full"]
# Element-resize auto-refresh (ResizeObserver on document.documentElement) is opt-in.
# The umbrella scroll-full/full features do NOT forward it; to enable it, depend on
# leptos_fluid_scroll directly with features = ["resize-observer"] (or ["full"]).
```

Or depend on this crate directly:

```toml
[dependencies]
leptos_fluid_scroll = { version = "0.1", default-features = false, features = ["controller", "builders"] }
# Add `timeline`, `macros`, or use `features = ["full"]` for the complete surface.
```

Feature split:

- `controller`: bind a `ScrollTrigger` to a `leptos_fluid_motion::AnimationController`.
- `timeline`: drive a `leptos_fluid_motion::FluidTimeline` from scroll progress.
- `builders`: typed builder API over `ScrollTrigger`.
- `macros`: `scroll_trigger!` declarative macro (implies `builders`).
- `resize-observer`: element-resize auto-refresh via `leptos_fluid_web` ResizeObserver on `document.documentElement` (opt-in; viewport resize refresh via `window.on_resize` is always on).
- `full`: convenience aggregate of all of the above (including `resize-observer`).
- Pure-callback mode (no features) needs none of these and has no `leptos_fluid_motion` dependency.

## What this crate provides

- always available: `ScrollTrigger`, `ScrollTriggerConfig`, `Scrub`, `ToggleActions`, `ScrollTriggerEvent`, `VelocityTracker`, position parsing (`parse_start_end`, `resolve_start`, ...), toggle parsing (`Action`, `TogglePhase`, `ScrollDirection`)
- `start`/`end` parsing: `"top center"`, `"bottom 80%"`, numeric, `"+=300"`, `"-=50%"`, and `clamp(...)` forms
- reactive readouts: `progress`, `direction`, `is_active`, `velocity` signals
- callbacks: `onEnter` / `onLeave` / `onEnterBack` / `onLeaveBack` / `onToggle` / `onUpdate` / `onRefresh`
- `toggleActions` four-phase mapping (`"onEnter onLeave onEnterBack onLeaveBack"`)
- `scrub` (boolean direct-link or `Number(t)` catch-up smoothing)
- `once` single-shot triggers that auto-kill after the forward leave
- lifecycle: `kill`, `disable`, `enable`, `refresh`, `scroll_position`, `get_velocity`
- viewport-resize auto-refresh always on via `window.on_resize`; element-resize auto-refresh (ResizeObserver on `document.documentElement`) via the `resize-observer` feature
- `controller`: `bind_controller`, `bind_controller_with`
- `timeline`: `bind_timeline` (toggleActions), `bind_timeline_scrub` (discrete-step)
- `builders`: `ScrollTrigger::builder()` typed builder with state-marker install guarantee
- `macros`: `scroll_trigger!` declarative macro
- convenience re-exports: `leptos_fluid_scroll::prelude::*`

## Quick start

This API requires the `builders` feature. The example below also uses `controller` for the motion binding.

```rust
use leptos::prelude::*;
use leptos_fluid_motion::{AnimationController, Easing, FluidStyle, Transition};
use leptos_fluid_scroll::prelude::*;

#[component]
fn ScrubDemo() -> impl IntoView {
    let card_ref = NodeRef::<leptos::html::Div>::new();

    let controller = AnimationController::builder()
        .target(card_ref)
        .transition(Transition::new().duration_ms(120).easing(Easing::Linear))
        .initial(FluidStyle::new().opacity(0.0).scale(0.8).y(100.0))
        .install();

    let trigger = ScrollTrigger::builder()
        .target(card_ref)
        .start("top center")
        .end("bottom center")
        .scrub(Scrub::Bool(true))
        .bind_controller(controller, |p| {
            FluidStyle::new()
                .opacity(p)
                .scale(0.8 + p * 0.2)
                .y(100.0 - p * 100.0)
        })
        .install();

    let progress = trigger.progress();

    view! {
        <div class="card" node_ref=card_ref>
            {move || format!("progress {:.2}", progress.get())}
        </div>
    }
}
```

The typed builder keeps `install()` unavailable until you call `.target(...)` or `.resolver(...)`. Call `.scrub(Scrub::Bool(true))` to link progress directly to scroll, or `.scrub(Scrub::Number(0.3))` for catch-up smoothing.

## Pure-callback mode

This is the `default-features = false` mode and pulls in no `leptos_fluid_motion` dependency. The example below uses the typed builder (`builders` feature); if you also want to avoid that, use `ScrollTrigger::create(config, target)` directly.

```rust
use leptos::prelude::*;
use leptos_fluid_scroll::prelude::*;

#[component]
fn PureCallbackDemo() -> impl IntoView {
    let card_ref = NodeRef::<leptos::html::Div>::new();
    let enters = RwSignal::new(0u32);
    let progress_readout = RwSignal::new(0.0f64);

    let enters_handle = enters;
    let progress_handle = progress_readout;

    let trigger = ScrollTrigger::builder()
        .target(card_ref)
        .start("top center")
        .end("bottom center")
        .on_enter(move |_| {
            enters_handle.update(|v| *v += 1);
        })
        .on_update(move |event| {
            progress_handle.set(event.progress);
        })
        .install();

    let progress = trigger.progress();
    let is_active = trigger.is_active();

    view! {
        <div class="card" node_ref=card_ref>
            <span>{move || if is_active.get() { "active" } else { "idle" }}</span>
            <span>{move || format!("progress {:.2}", progress.get())}</span>
            <span>{move || format!("enters {}", enters.get())}</span>
            <span>{move || format!("cb {:.2}", progress_readout.get())}</span>
        </div>
    }
}
```

## Scrubbing an AnimationController

This API requires the `controller` feature. The typed builder form also requires `builders`.

`bind_controller` creates a Leptos `Effect` that reads `ScrollTrigger::progress()` and dispatches the derived `FluidStyle` to the controller. The first sample is applied immediately (no tween) so the controller adopts the current scroll state as its baseline; subsequent samples animate via the controller's default transition. `bind_controller_with` accepts a fixed `Transition` override per update.

```rust
use leptos::prelude::*;
use leptos_fluid_motion::{AnimationController, Easing, FluidStyle, Transition};
use leptos_fluid_scroll::prelude::*;

#[component]
fn ScrubControllerDemo() -> impl IntoView {
    let node_ref = NodeRef::<leptos::html::Div>::new();

    let controller = AnimationController::builder()
        .target(node_ref)
        .transition(Transition::new().duration_ms(120).easing(Easing::Linear))
        .initial(FluidStyle::new().opacity(0.0).y(100.0))
        .install();

    let _trigger = ScrollTrigger::builder()
        .target(node_ref)
        .start("top center")
        .end("bottom center")
        .scrub(Scrub::Number(0.3))
        .bind_controller(controller, |p| {
            FluidStyle::new().opacity(p).y(100.0 - p * 100.0)
        })
        .install();

    view! {
        <div class="card" node_ref=node_ref>"Scrub-bound controller"</div>
    }
}
```

For `scrub: Number`, the scroll engine already smooths `progress()` (see `step_scrub` in `trigger.rs`), so `style_fn` receives the smoothed value and the binding never double-smooths.

## Driving a FluidTimeline via toggleActions

This API requires the `timeline` feature. The typed builder form also requires `builders`.

`bind_timeline` maps the four-phase `toggleActions` string (`"onEnter onLeave onEnterBack onLeaveBack"`) to `FluidTimeline` methods. The binding watches `is_active()` and `direction()` and dispatches the configured `Action` on each phase transition.

```rust
use leptos::prelude::*;
use leptos_fluid_motion::{
    AnimationController, Easing, FluidStep, FluidStyle, FluidTimeline, Transition,
};
use leptos_fluid_scroll::prelude::*;

#[component]
fn TimelineToggleDemo() -> impl IntoView {
    let node_ref = NodeRef::<leptos::html::Div>::new();

    let transition = Transition::new().duration_ms(300).easing(Easing::EaseInOut);
    let controller = AnimationController::builder()
        .target(node_ref)
        .transition(transition.clone())
        .initial(FluidStyle::new().opacity(0.5).y(20.0))
        .install();

    let timeline = FluidTimeline::builder(controller)
        .initial(FluidStyle::new().opacity(0.5).y(20.0))
        .autoplay(false)
        .step(FluidStep::to(FluidStyle::new().opacity(1.0).y(0.0)).inherit_wait_from(&transition))
        .step(FluidStep::to(FluidStyle::new().opacity(0.95).x(20.0)).inherit_wait_from(&transition))
        .step(FluidStep::to(FluidStyle::new().opacity(0.9).y(-10.0)).inherit_wait_from(&transition))
        .install();

    let _trigger = ScrollTrigger::builder()
        .target(node_ref)
        .start("top center")
        .end("bottom top")
        .bind_timeline(timeline, "play pause resume none")
        .install();

    view! {
        <div class="card" node_ref=node_ref>"Timeline toggle"</div>
    }
}
```

`Reset`, `Complete`, and `Reverse` have no exact `FluidTimeline` primitive: see `technical.md` for the chosen mappings and their limitations.

## Discrete-step timeline scrubbing

This API requires the `timeline` feature. The typed builder form also requires `builders`.

`bind_timeline_scrub` maps scroll `progress()` to a discrete step index `(progress * step_count).floor()` clamped to `step_count - 1` and calls `timeline.set_immediate(style_fn(index, progress))` when the target index changes. `step_count` is supplied by the caller because `FluidTimeline` does not expose its step list for reading.

```rust
use leptos::prelude::*;
use leptos_fluid_motion::{AnimationController, FluidStyle, FluidTimeline, Transition};
use leptos_fluid_scroll::prelude::*;

const STEP_COUNT: usize = 4;

#[component]
fn TimelineScrubDemo() -> impl IntoView {
    let node_ref = NodeRef::<leptos::html::Div>::new();

    let controller = AnimationController::builder()
        .target(node_ref)
        .transition(Transition::new().duration_ms(120))
        .initial(step_style(0))
        .install();

    // FluidTimeline::new + bind (rather than the builder) because no steps are
    // needed here: bind_timeline_scrub drives style directly via set_immediate
    // on the controller, so the timeline's own step list is never sequenced.
    let timeline = FluidTimeline::new(step_style(0));
    timeline.bind(controller);

    let _trigger = ScrollTrigger::builder()
        .target(node_ref)
        .start("top center")
        .end("bottom center")
        .scrub(Scrub::Bool(true))
        .bind_timeline_scrub(timeline, STEP_COUNT, |idx, _p| step_style(idx))
        .install();

    view! {
        <div class="card" node_ref=node_ref>"Discrete-step scrub"</div>
    }
}

fn step_style(index: usize) -> FluidStyle {
    match index {
        0 => FluidStyle::new().opacity(0.6).y(40.0),
        1 => FluidStyle::new().opacity(0.85).y(0.0),
        2 => FluidStyle::new().opacity(1.0).y(-10.0),
        _ => FluidStyle::new().opacity(0.92).y(-20.0),
    }
}
```

**Limitation:** `FluidTimeline` is step-index based with `wait_ms` per step, not a continuous time-based timeline, and `FluidStyle` has no built-in lerp. This binding jumps between steps rather than interpolating. Continuous interpolated scrubbing is deferred until `FluidStyle` gains an interpolation helper. For smooth scrubbing today, use `bind_controller`.

## `scroll_trigger!` macro

This API requires the `macros` feature (which implies `builders`).

```rust
use leptos::prelude::*;
use leptos_fluid_motion::{AnimationController, FluidStyle, Transition};
use leptos_fluid_scroll::prelude::*;

#[component]
fn MacroDemo() -> impl IntoView {
    let node_ref = NodeRef::<leptos::html::Div>::new();

    let controller = AnimationController::builder()
        .target(node_ref)
        .transition(Transition::new().duration_ms(120))
        .initial(FluidStyle::new().opacity(0.0).y(100.0))
        .install();

    let _trigger = scroll_trigger! {
        trigger: node_ref,
        start: "top center",
        end: "bottom center",
        scrub: true,
        bind_controller: (controller, |p| {
            FluidStyle::new().opacity(p).y(100.0 - p * 100.0)
        }),
    };

    view! {
        <div class="card" node_ref=node_ref>"Macro-built trigger"</div>
    }
}
```

Supported fields:

- `trigger: $expr` or `resolver: $expr` (exactly one required)
- `start: $expr`, `end: $expr`, `once: $expr`, `id: $expr`
- `scrub: $expr` (accepts `true` / `false` / numeric / `Scrub`)
- `toggle_actions: $expr`
- `on_enter` / `on_leave` / `on_enter_back` / `on_leave_back` / `on_toggle` / `on_update` / `on_refresh`: `$expr`
- `bind_controller: ($controller, $style_fn)` (feature `controller`)
- `bind_controller_with: ($controller, $transition, $style_fn)` (feature `controller`)
- `bind_timeline: ($timeline, $toggle_actions_str)` (feature `timeline`)
- `bind_timeline_scrub: ($timeline, $step_count, $style_fn)` (feature `timeline`)

Each field may appear at most once; unknown fields and invalid syntax produce `compile_error!`. The `bind_controller` / `bind_timeline` fields emit the corresponding builder methods, which only exist when the corresponding feature is on, so a missing feature produces a method-not-found error rather than a macro error.

## Limitations

The following GSAP ScrollTrigger features are deferred and not yet implemented; see `technical.md` for the roadmap and the planned module home for each:

- `pin`
- `snap`
- `markers`
- `batch`
- horizontal scrolling
- custom scroller elements (only the viewport is supported)
- `matchMedia` / responsive triggers
- containerAnimation coupling

Additionally, `bind_timeline_scrub` jumps between steps rather than interpolating continuously (see the section above). For smooth scrubbing, use `bind_controller`.

## Examples

A full walkthrough of every integration mode lives in `example_scroll/` in the repo:

- `pure_callback.rs` - callback-only mode with `on_enter` / `on_update`
- `once_reveal.rs` - `once: true` one-shot reveal driving a controller
- `scrub_card.rs` - `bind_controller` with `scrub: true`
- `timeline_toggle.rs` - `bind_timeline` with `"play pause resume none"`
- `timeline_scrub.rs` - `bind_timeline_scrub` with a 4-step timeline

For workspace-level docs and cross-crate guidance, see the root `README.md`.