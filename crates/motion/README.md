# leptos_fluid_motion

A tiny, WASM-friendly motion layer for Leptos. It uses CSS transitions with a small
Rust-first API and keeps the bundle as lean as possible.

## Install

Use via the workspace crate:

```toml
[dependencies]
leptos_fluid = { path = "../leptos_fluid", features = ["motion"] }
```

Or depend on the motion crate directly:

```toml
[dependencies]
leptos_fluid_motion = { path = "../leptos_fluid/crates/motion" }
```

## Quick start

```rust
use leptos::prelude::*;
use leptos_fluid::motion::{MotionDiv, MotionStyle, Transition};

#[component]
fn Demo() -> impl IntoView {
    let expanded = RwSignal::new(false);

    let animate = move || {
        if expanded.get() {
            MotionStyle::new().opacity(1.0).y(0.0).scale(1.0)
        } else {
            MotionStyle::new().opacity(0.6).y(24.0).scale(0.96)
        }
    };

    view! {
        <MotionDiv
            class="card"
            initial=MotionStyle::new().opacity(0.0).y(16.0)
            animate=animate
            transition=Transition::spring()
            while_hover=MotionStyle::new().scale(1.02)
            while_tap=MotionStyle::new().scale(0.98)
        >
            <button on:click=move |_| expanded.update(|v| *v = !*v)>
                "Toggle"
            </button>
        </MotionDiv>
    }
}
```

## MotionElement (dynamic tag)

If you want a single reusable element for any tag, use `MotionElement` and pass the tag name:

```rust
use leptos::prelude::*;
use leptos_fluid::motion::{MotionElement, MotionStyle, Transition};

view! {
    <MotionElement
        tag="section"
        class="panel"
        initial=MotionStyle::new().opacity(0.0).y(12.0)
        animate=MotionStyle::new().opacity(1.0).y(0.0)
        transition=Transition::spring()
    >
        "Hello motion"
    </MotionElement>
}
```

If you need a node ref, use `MotionNodeRef` (it maps to `web_sys::HtmlElement` for any tag).

## MotionStyle

`MotionStyle` is a small builder that produces CSS keyframes + inline styles.

```rust
use leptos_fluid::motion::{MotionStyle, style};

let base = MotionStyle::new()
    .opacity(0.8)
    .x(12.0)
    .y(-4.0)
    .scale(1.05)
    .rotate(8.0)
    .with("filter", "blur(2px)");

let custom = style!(
    "opacity" => 0.4,
    "filter" => "blur(6px)",
);
```

If you set `transform` manually via `set("transform", ...)`, the auto-generated transform chain is not appended.

## Transition

```rust
use leptos_fluid::motion::{Transition, Easing, Spring};

let quick = Transition::new().duration_ms(150).easing(Easing::EaseOut);
let springy = Transition::spring();
let bouncy = Transition::spring_with(600, 0.3);
let tuned = Transition::new().duration_ms(260).bounce(0.2);
let custom = Spring::new(500, 0.2);
let spring = Transition::spring_with(custom.duration_ms, custom.bounce);
```

By default transitions use `all` for implicit animation. You can opt out of specific properties:

```rust
let no_layout = Transition::spring().exclude_properties(["width", "height"]);
let no_blur = Transition::new().without_properties(["filter"]);
```

## MotionSignal

`animate`, `class`, and `style` accept static values, Leptos signals/memos, or closures.
This keeps the DX ergonomic while staying light.

```rust
use leptos_fluid::motion::MotionSignal;

let class = MotionSignal::from("card");
let style = MotionSignal::from(move || format!("opacity:{};", opacity.get()));
```

You can also pass a `node_ref` if you need direct access to the underlying element.

## Spring values

For pointer-follow or drag experiences, use a spring value that is driven by duration + bounce.

```rust
use leptos::prelude::*;
use leptos_fluid::motion::{use_spring, Spring, MotionDiv, MotionStyle};

let x = use_spring(0.0, Spring::new(500, 0.2));
let y = use_spring(0.0, Spring::new(500, 0.2));

let ball_style = move || MotionStyle::new().x(x.get()).y(y.get());

view! {
    <MotionDiv
        class="ball"
        animate=ball_style
        initial=MotionStyle::new()
    ></MotionDiv>
}
```

## AnimatePresence

Keep an element mounted long enough to play an exit animation.

```rust
use leptos::prelude::*;
use leptos_fluid::motion::{AnimatePresence, MotionStyle, Transition};

let open = RwSignal::new(false);

view! {
    <button on:click=move |_| open.update(|v| *v = !*v)>"Toggle"</button>
    <AnimatePresence
        show=open
        initial=MotionStyle::new().opacity(0.0).y(12.0)
        animate=MotionStyle::new().opacity(1.0).y(0.0)
        exit=MotionStyle::new().opacity(0.0).y(-12.0)
        transition=Transition::spring()
    >
        <div class="panel">"I'm here until exit finishes"</div>
    </AnimatePresence>
}
```

## Design goals

- Minimal runtime and deps (CSS transitions only)
- Fast updates (cancels/replaces active animations)
- Ergonomic API with predictable defaults

## Benchmarks

Run microbenchmarks for spring math and style/transition generation:

```bash
cargo bench -p leptos_fluid_motion --features bench
```
