# leptos_fluid_motion

Reactive motion primitives for Leptos (CSR), built around lightweight style updates and WAAPI-driven transitions.

## Install

Via umbrella crate:

```toml
[dependencies]
leptos_fluid = { version = "0.1", features = ["motion"] }
```

Or depend on this crate directly:

```toml
[dependencies]
leptos_fluid_motion = { version = "0.1", default-features = false, features = ["controller", "builders"] }
```

Feature split:

- `spring`: `use_spring`, `SpringValue`
- `controller`: `AnimationController`
- `auto-size`: ResizeObserver-backed `bind_auto_height`, `bind_auto_width`, `bind_auto_size`
- `timeline`: `FluidTimeline`, `FluidStep`
- `components`: `FluidElement`
- `wrappers`: `FluidDiv`, `FluidSpan`, `FluidButton`
- `builders`: typed builder APIs
- `macros`: `controller!`, `when!`, `timeline!`

`leptos_fluid_motion` now defaults to no features. Opt into `full` if you want the old all-in surface.

## What this crate provides

- motion components: `FluidElement`, `FluidDiv`, `FluidSpan`, `FluidButton`
- element-agnostic controller: `AnimationController`
- style primitives: `FluidStyle`, `FluidValue`, `Transform`, `style!`
- transition primitives: `Transition`, `Spring`, `Easing`
- sequencing: `FluidTimeline`, `FluidStep`
- spring values: `use_spring`, `SpringValue`
- reactive adapters: `FluidSignal<T>`
- prelude: `leptos_fluid_motion::prelude::*`

## Quick start

```rust
use leptos::prelude::*;
use leptos_fluid_motion::{FluidDiv, FluidStyle, Transition};

#[component]
fn Demo() -> impl IntoView {
    let expanded = RwSignal::new(false);
    let animate = move || {
        if expanded.get() {
            FluidStyle::new().opacity(1.0).y(0.0).scale(1.0)
        } else {
            FluidStyle::new().opacity(0.6).y(24.0).scale(0.96)
        }
    };

    view! {
        <FluidDiv
            class="card"
            initial=FluidStyle::new().opacity(0.0).y(16.0)
            animate=animate
            transition=Transition::new().duration_ms(220)
            while_hover=FluidStyle::new().scale(1.02)
            while_tap=FluidStyle::new().scale(0.98)
        >
            <button on:click=move |_| expanded.update(|v| *v = !*v)>
                "Toggle"
            </button>
        </FluidDiv>
    }
}
```

## `FluidElement` and wrappers

Use wrappers for common tags:

- `FluidDiv`
- `FluidSpan`
- `FluidButton`

Use `FluidElement` for custom tags:

```rust
use leptos::prelude::*;
use leptos_fluid_motion::{FluidElement, FluidStyle, Transition};

view! {
    <FluidElement
        tag="section"
        class="panel"
        initial=FluidStyle::new().opacity(0.0).y(12.0)
        animate=FluidStyle::new().opacity(1.0).y(0.0)
        transition=Transition::new()
    >
        "Hello motion"
    </FluidElement>
}
```

`animate`, `class`, and `style` props accept static values, signals, memos, and closures via `FluidSignal`.

Use `Transition::new()` / `Transition::default()` for most enter/exit UI. Reserve `use_spring(...)` for continuously retargeted motion such as pointer follow or drag-like interactions.

## `AnimationController` (without motion elements)

Use `AnimationController` when you want to animate a plain element/ref without `FluidElement` wrappers:

```rust
use leptos::prelude::*;
use leptos_fluid_motion::{AnimationController, FluidStyle, Transition};

#[component]
fn ControllerDemo() -> impl IntoView {
    let expanded = RwSignal::new(false);
    let node_ref = NodeRef::<leptos::html::Div>::new();
    let controller = AnimationController::builder()
        .target(node_ref)
        .transition(Transition::new().duration_ms(220))
        .initial(FluidStyle::new().opacity(0.65).y(20.0).scale(0.96))
        .install();

    controller.on_change(move || expanded.get(), move |expanded, controller| {
        if expanded {
            controller.animate(FluidStyle::new().opacity(1.0).y(0.0).scale(1.0));
        } else {
            controller.animate(FluidStyle::new().opacity(0.65).y(20.0).scale(0.96));
        }
    });

    view! {
        <button on:click=move |_| expanded.update(|v| *v = !*v)>
            "Toggle"
        </button>
        <div node_ref=node_ref class="card">"Animated by controller"</div>
    }
}
```

The typed builder gives IDE-friendly method completion and keeps `install()` unavailable until you call `target(...)` or `resolver(...)`.

Use `target(...)`/`target:` for a stable `NodeRef` or `Element`, and `resolver(...)`/`resolver:` for dynamic lookup when the active element can change over time.

If you prefer declarative sugar, `controller!` and `when!` lower to the same runtime.

## `FluidStyle` and `style!`

`FluidStyle` combines typed helpers and arbitrary CSS property pairs.

```rust
use leptos_fluid_motion::{style, FluidStyle};

let typed = FluidStyle::new()
    .opacity(0.8)
    .x(12.0)
    .y(-4.0)
    .scale(1.05)
    .rotate(8.0)
    .with("filter", "blur(2px)");

let macro_style = style!(
    "opacity" => 0.4,
    "filter" => "blur(6px)",
);
```

If you set `"transform"` manually via `set`/`with`, the builder does not append its auto-generated transform chain.

## `Transition`, `Spring`, and `Easing`

```rust
use leptos_fluid_motion::{Easing, Spring, Transition};

let quick = Transition::new().duration_ms(150).easing(Easing::EaseOut);
let springy = Transition::spring();
let bouncy = Transition::spring_with(600, 0.3);
let tuned = Transition::new().duration_ms(260).bounce(0.2);
let custom_spring = Spring::new(500, 0.2).rest_delta(0.0005);

let no_layout = Transition::spring().exclude_properties(&["width", "height"]);
let no_blur = Transition::new().without_properties(&["filter"]);
```

Excluded properties are applied immediately while other properties animate.

Spring transitions use a live rAF-driven solver. Unsupported properties are applied immediately and do not interpolate in spring mode; use tween transitions for colors, filters, shadows, and other text-valued CSS properties.

## `use_spring` for continuously retargeted values

Use `use_spring` for pointer-follow/drag-like interactions:

```rust
use leptos::prelude::*;
use leptos_fluid_motion::{use_spring, FluidDiv, FluidStyle, Spring, Transition};

let x = use_spring(0.0, Spring::new(500, 0.2));
let y = use_spring(0.0, Spring::new(500, 0.2));

let ball_style = move || FluidStyle::new().x(x.get()).y(y.get());

view! {
    <FluidDiv
        class="ball"
        initial=FluidStyle::new()
        animate=ball_style
        transition=Transition::new().duration_ms(0)
    />
}
```

Using `duration_ms(0)` avoids double interpolation when the spring already controls value smoothing.

## `FluidTimeline` for multi-step sequences

```rust
use leptos::prelude::*;
use leptos_fluid_motion::{
    AnimationController, Easing, FluidStep, FluidStyle, FluidTimeline, Transition,
};

let node_ref = NodeRef::<leptos::html::Div>::new();
let transition = Transition::new().duration_ms(240).easing(Easing::EaseInOut);
let paused = RwSignal::new(false);
let controller = AnimationController::builder()
    .target(node_ref)
    .transition(transition.clone())
    .initial(FluidStyle::new().opacity(0.45).y(18.0).scale(0.94))
    .install();
let timeline = FluidTimeline::builder(controller)
    .initial(FluidStyle::new().opacity(0.45).y(18.0).scale(0.94))
    .autoplay(true)
    .auto_loop(true)
    .step(FluidStep::to(FluidStyle::new().opacity(1.0).y(0.0).scale(1.0)))
    .step(FluidStep::to(FluidStyle::new().opacity(0.95).x(22.0).rotate(3.0)))
    .step(FluidStep::to(FluidStyle::new().opacity(0.5).y(12.0).scale(0.92)).wait_ms(180))
    .on_change(move || paused.get(), move |paused, timeline| {
        if paused {
            timeline.pause();
        } else {
            timeline.resume();
        }
    })
    .install();

view! {
    <div node_ref=node_ref class="card">"Timeline-driven controller target"</div>
}
```

`FluidTimeline` supports play/pause/resume/restart/stop, immediate set, and optional auto-loop. The typed builder keeps `install()` unavailable until you add at least one `.step(...)`.

If you prefer declarative syntax, `timeline!` offers the same behavior with structured steps and `triggers:` sugar.

## Benchmarks

```bash
cargo bench -p leptos_fluid_motion --features bench
```

For workspace-level docs and cross-crate guidance, see the root `README.md`.
