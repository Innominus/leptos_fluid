# leptos_fluid

`leptos_fluid` is a feature-gated umbrella crate for animation primitives in Leptos CSR apps.

It ships three focused modules:

- `view_transitions`: nested outlet route transitions for `leptos_router`.
- `flip`: FLIP animations for layout moves, resizes, and list/grid reorders.
- `motion`: reactive element motion with style builders, transitions, timelines, and spring values.

The crate is intentionally modular. You enable only what you need.

## Table of contents

- [Crate layout and feature flags](#crate-layout-and-feature-flags)
- [Installation](#installation)
- [Quick start](#quick-start)
- [View transitions API guide](#view-transitions-api-guide)
- [FLIP API guide](#flip-api-guide)
- [Motion API guide](#motion-api-guide)
- [Choosing the right tool](#choosing-the-right-tool)
- [Examples in this repo](#examples-in-this-repo)
- [Benchmarks](#benchmarks)
- [Known constraints and gotchas](#known-constraints-and-gotchas)
- [Back-compat paths](#back-compat-paths)

## Crate layout and feature flags

`leptos_fluid` has no default features.

| Feature | Module path | Purpose |
| --- | --- | --- |
| `view-transitions` | `leptos_fluid::view_transitions::*` | Nested route outlet transitions using intro/outro CSS animation classes |
| `flip` | `leptos_fluid::flip::*` | FLIP capture/invert/play for layout changes |
| `motion` | `leptos_fluid::motion::*` | Common motion surface (`AnimationController`, `FluidElement`, wrappers) |
| `motion-core` | `leptos_fluid::motion::*` | Core motion types like `FluidStyle`, `Transition`, and `style!` |
| `motion-controller` | `leptos_fluid::motion::*` | `AnimationController` runtime |
| `motion-auto-size` | `leptos_fluid::motion::*` | ResizeObserver-backed height/width helpers |
| `motion-timeline` | `leptos_fluid::motion::*` | `FluidTimeline` and `FluidStep` |
| `motion-components` | `leptos_fluid::motion::*` | `FluidElement` |
| `motion-wrappers` | `leptos_fluid::motion::*` | `FluidDiv`, `FluidSpan`, `FluidButton` |
| `motion-builders` | `leptos_fluid::motion::*` | typed builder APIs |
| `motion-macros` | `leptos_fluid::motion::*` | `controller!`, `when!`, `timeline!` |
| `motion-spring` | `leptos_fluid::motion::*` | `use_spring`, `SpringValue` |
| `motion-full` | `leptos_fluid::motion::*` | Full motion surface without flip/view-transitions |
| `full` | all above | Convenience feature to enable everything |

This repository also includes internal helper crates:

- `leptos_fluid_web`: browser and WAAPI helpers used internally by `flip` and `motion`.

## Installation

### Umbrella crate (recommended)

```toml
[dependencies]
leptos_fluid = { path = "../leptos_fluid", features = ["motion"] }
# or: features = ["view-transitions", "flip", "motion"]
# or: features = ["full"]

# narrower motion builds via the umbrella crate:
# features = ["motion-core", "motion-controller", "motion-macros"]
# features = ["motion-core", "motion-controller", "motion-auto-size", "motion-timeline", "motion-builders"]
# features = ["motion-full"]
```

### Direct subcrate dependencies

If you only want one module without the umbrella crate:

```toml
[dependencies]
leptos_fluid_motion = { path = "../leptos_fluid/crates/motion" }
leptos_fluid_flip = { path = "../leptos_fluid/crates/flip" }
leptos_fluid_view_transitions = { path = "../leptos_fluid/crates/view_transitions" }
```

For smaller wasm builds, depend on `leptos_fluid_motion` directly with `default-features = false` and only the pieces you need, for example `features = ["controller", "builders"]`.

The umbrella crate now forwards those fine-grained motion features too, so you can stay on `leptos_fluid` without paying for the entire motion surface.

## Quick start

### 1) Motion component

```rust
use leptos::prelude::*;
use leptos_fluid::motion::{FluidDiv, FluidStyle, Transition};

#[component]
fn Card() -> impl IntoView {
    let open = RwSignal::new(false);
    let animate = move || {
        if open.get() {
            FluidStyle::new().opacity(1.0).y(0.0).scale(1.0)
        } else {
            FluidStyle::new().opacity(0.7).y(20.0).scale(0.96)
        }
    };

    view! {
        <button on:click=move |_| open.update(|v| *v = !*v)>"Toggle"</button>
        <FluidDiv
            class="card"
            initial=FluidStyle::new().opacity(0.0).y(24.0)
            animate=animate
            transition=Transition::new().duration_ms(220)
            while_hover=FluidStyle::new().scale(1.02)
            while_tap=FluidStyle::new().scale(0.98)
        >
            "Hello"
        </FluidDiv>
    }
}
```

### 2) Route outlet transitions

```rust
use leptos::prelude::*;
use leptos_fluid::view_transitions::{FluidManager, FluidOutlet, FluidRoutes};
use leptos_router::{
    components::{ParentRoute, Route, Router},
    StaticSegment,
};

#[component]
fn App() -> impl IntoView {
    provide_context(FluidManager::new());

    view! {
        <Router>
            <FluidRoutes fallback=|| "Not found">
                <ParentRoute path=StaticSegment("/") view=Shell>
                    <Route path=StaticSegment("") view=Home />
                    <Route path=StaticSegment("about") view=About />
                </ParentRoute>
            </FluidRoutes>
        </Router>
    }
}

#[component]
fn Shell() -> impl IntoView {
    view! {
        <main>
            <FluidOutlet intro_class="route-enter" outro_class="route-exit" />
        </main>
    }
}
```

Define your classes in CSS (`route-enter`, `route-exit`) with keyframe animations.

### 3) FLIP (single element)

```rust
use leptos::prelude::*;
use leptos_fluid::flip::{Flip, FlipOptions, ScaleMode};

#[component]
fn FlipCard() -> impl IntoView {
    let right = RwSignal::new(false);
    let flip = Flip::new_with_options(
        "flip-card".to_string(), // element id (without '#')
        FlipOptions {
            duration: 260,
            scale_mode: ScaleMode::PositionAndScale,
            ..Default::default()
        },
    );

    let toggle = move |_| {
        flip.animate(move || right.update(|v| *v = !*v));
    };

    view! {
        <button on:click=toggle>"Move"</button>
        <div class="lane" class:right=move || right.get()>
            <div id="flip-card">"FLIP me"</div>
        </div>
    }
}
```

## View transitions API guide

### Mental model

`view_transitions` focuses on nested router outlet transitions. During navigation:

1. The currently visible outlet content is cloned into an outgoing layer.
2. New route content renders in the incoming layer.
3. Intro/outro CSS animation classes are applied.
4. After both animationend events fire, outgoing content is cleaned up.

This approach lets you keep your existing route components and only replace `Routes` + `Outlet` wiring.

### Required wiring

1. Provide a single `FluidManager` context at app root.
2. Wrap routes with `FluidRoutes`.
3. Replace each `Outlet` with `FluidOutlet`.

```rust
provide_context(FluidManager::new());
```

`FluidRoutes` is a transparent wrapper around `leptos_router::Routes` and also stores generated route patterns for direction detection.

`FluidOutlet` takes:

- `intro_class: Signal<&'static str>`
- `outro_class: Signal<&'static str>`

Class names are static string signals, so use CSS class constants or closures returning static string literals.

### CSS contract

`FluidOutlet` does not ship animation styles. You provide them.

Example:

```css
.route-enter { animation: slide-in 320ms ease; }
.route-exit { animation: fade-out 320ms ease; }
```

Important behavior:

- Outgoing layer also gets `no-animations`, which forces child animations/transitions to zero duration to avoid duplicated inner motion during cloning.
- Cleanup depends on `animationend` on outlet root wrappers. If your class does not run an animation, cleanup will not trigger.

### Nested outlets and scroll restoration

`FluidManager` tracks outlet routes in a hierarchy and truncates deeper caches when upper routes change.

If you mark inner scroll containers with `data-scrollable`, the manager captures and restores their scroll positions during the transition clone phase:

```html
<section data-scrollable class="overflow-y-scroll">...</section>
```

### Navigation direction and back behavior

Direction is determined by route declaration order collected from `FluidRoutes`.

- Navigating to a route with a lower generated index marks transition as backward.
- On backward navigation, intro/outro class assignment is reversed.

The manager also installs a `popstate` compatibility fallback:

- On iOS and Safari contexts detected as incompatible, the next back navigation skips transition once to avoid broken behavior.

### Public API summary

- `FluidManager::new()`: create manager context and install compatibility listener.
- `FluidManager::get_manager()`: retrieve manager from context.
- `FluidRoutes(...)`: route wrapper that forwards to `Routes` and captures route patterns for direction detection. Its optional `transition` prop is forwarded to `leptos_router::Routes`.
- `FluidOutlet(intro_class, outro_class)`: animated outlet replacement.

When to use this module:

- You want animated nested route outlets with minimal router rewrite.
- You are comfortable controlling animation look fully in CSS.

When not to use it:

- You need spring physics between pages instead of CSS-keyframe-based route transitions.
- You need SSR/hydration-specific routing behavior (this crate currently targets CSR usage).

## FLIP API guide

### Mental model

FLIP here follows the standard sequence:

1. **First**: measure initial layout.
2. **Last**: run your state mutation.
3. **Invert**: apply inverse translate/scale transform.
4. **Play**: animate transform back to identity.

This is ideal for layout changes that would otherwise "jump":

- moving a card between lanes
- expanding/collapsing tile size
- reordering list/grid children

### `Flip` (single element)

Use `Flip` when one specific element (selected by **id**) is changing layout.

```rust
let flip = Flip::new("my-id".to_string());
flip.animate(move || {
    // mutate signal/state that changes the layout
});
```

Public methods:

- `Flip::new(id_selector: String)`
- `Flip::new_with_options(id_selector: String, options: FlipOptions)`
- `set_id_selector`, `set_options`
- `get_is_animating_signal() -> Signal<bool>`
- `animate(f)`
- `measure(element)` and `rect(element)` for manual measurement workflows

`id_selector` is passed to `document().get_element_by_id`, so provide the raw id (for example `"card-a"`, not `"#card-a"`).

### `FlipGroup` (multiple elements)

Use `FlipGroup` when many elements move/reorder in one mutation.

```rust
let group = FlipGroup::new(".tile".to_string());
group.animate(move || {
    items.update(|v| v.rotate_left(1));
});
```

Identity resolution for group items:

1. `data-flip-id`
2. `id`
3. fallback index key (`__flip-index-N`)

Always prefer stable `data-flip-id` for correctness during reorder/insert/remove operations.

### `FlipOptions`

`FlipOptions` is a plain struct:

```rust
pub struct FlipOptions {
    pub duration: usize,
    pub delay: usize,
    pub stagger: usize,
    pub easing: Easing,
    pub scale_mode: ScaleMode,
    pub scale_correction_selector: Option<&'static str>,
}
```

Notes:

- `duration` default from `FlipOptions::new()` is `240` ms.
- `stagger` is index-based delay in group mode (`delay + stagger * index`).
- `easing` uses FLIP-local enum and defaults to `Easing::EaseInOut`:
  - `Easing::EaseInOut`
  - `Easing::Linear`
  - `Easing::Custom(&'static str)`
- `scale_mode`:
  - `PositionOnly`: only translate
  - `PositionAndScale`: translate + scale (handles resize transitions)
- `scale_correction_selector`: optional descendant selector to counter parent scaling so child content stays crisp.

### Scale and border-radius corrections

When `scale_mode = PositionAndScale`:

- root element animates with scale.
- optional descendant scale correction can run every frame.
- border radius is corrected every frame to avoid stretched rounded corners.

Use correction selector for text-heavy content that looks blurry under parent scale:

```rust
FlipOptions {
    scale_mode: FlipScaleMode::PositionAndScale,
    scale_correction_selector: Some(".flip-tile-content"),
    ..Default::default()
}
```

### FLIP usage checklist

- Keep layout-changing state mutation inside `flip.animate(...)`.
- Keep identity stable (`data-flip-id`).
- Use `PositionAndScale` only when size interpolation is needed.
- Add `scale_correction_selector` only where visual quality requires it.
- Use `get_is_animating_signal()` to disable conflicting controls mid-flight if needed.

## Motion API guide

### Core exports

`motion` exports:

- controller: `AnimationController`
- builders: `AnimationController::builder()`, `FluidTimeline::builder(...)`
- macros: `controller!`, `when!`, `timeline!`, `style!`
- components: `FluidElement`, `FluidDiv`, `FluidSpan`, `FluidButton`
- styles: `FluidStyle`, `FluidValue`, `style!`
- transitions: `Transition`, `Spring`, `Easing`
- signal bridge: `FluidSignal<T>`
- timelines: `FluidTimeline`, `FluidStep`
- spring values: `SpringValue`, `use_spring`
- convenience: `motion::prelude::*`

### Motion components

`FluidDiv`, `FluidSpan`, and `FluidButton` are typed wrappers over `FluidElement`.

`FluidElement` props:

| Prop | Type | Default | What it does |
| --- | --- | --- | --- |
| `tag` | `&'static str` | `"div"` | Underlying HTML tag (`FluidElement` only) |
| `initial` | `FluidStyle` | empty | Style applied before first animate run |
| `animate` | `FluidSignal<FluidStyle>` | empty | Reactive target style |
| `transition` | `Transition` | `Transition::default()` | Animation runtime config |
| `reset` | `Signal<u32>` | `0` | Re-runs initial behavior when value changes |
| `while_hover` | `Option<FluidStyle>` | `None` | Temporary style while pointer is over element |
| `while_tap` | `Option<FluidStyle>` | `None` | Temporary style while pointer is pressed |
| `class` | `FluidSignal<String>` | `""` | Static or reactive `class` attribute |
| `style` | `FluidSignal<String>` | `""` | Raw non-animated style string |
| `node_ref` | `Option<FluidNodeRef>` | auto | Node ref to underlying element |

Minimal pattern:

```rust
<FluidDiv
    initial=FluidStyle::new().opacity(0.0).y(16.0)
    animate=move || FluidStyle::new().opacity(1.0).y(0.0)
    transition=Transition::new()
/>
```

### `FluidStyle` and `FluidValue`

`FluidStyle` is a builder for animated CSS props.

Built-in helpers:

- transforms: `x`, `y`, `translate_x`, `translate_y`, `scale`, `rotate`
- dimensions: `width`, `height`, `size`
- common prop: `opacity`
- generic props: `set`, `with`, `set_prop`, `with_prop`

Transform order is deterministic:

1. `translate3d(...)`
2. `scale(...)`
3. `rotate(...)`

If you set a raw `transform` prop yourself, auto transform composition is skipped.

`FluidValue` supports:

- numbers (`f64`, `f32`, `i32`, `u32`)
- text (`&'static str`, `String`, `Cow<'static, str>`)

`style!` macro (from `leptos_fluid_motion`):

```rust
use leptos_fluid::motion::FluidStyle;
use leptos_fluid_motion::style;

let s = style!(
    "opacity" => 1.0,
    "background" => "linear-gradient(120deg, #0b0d18, #111827)",
)
.y(0.0)
.scale(1.0);
```

### `Transition`, `Spring`, `Easing`

`Transition` drives animation timing:

- `Transition::new()` default: `200ms`, ease-out cubic.
- `Transition::snappy()` default: `150ms`.
- `Transition::spring()` default: `500ms`, bounce `0.2`.
- `Transition::spring_with(duration_ms, bounce)`.

Spring transitions use a live rAF-driven solver. Unsupported properties are applied immediately and do not interpolate in spring mode.

Use `Transition::new()` for most product UI transitions. Reach for `use_spring(...)` when the target changes continuously and you need the motion to stay interruptible.

Other controls:

- `duration_ms`, `delay_ms`
- `easing(Easing::...)`
- `bounce(...)`
- `exclude_properties(&["width", "height"])`
- `add_excluded_properties(...)`
- `without_properties(...)` (alias of `exclude_properties`)

Behavior notes:

- `bounce(...)` enables spring behavior when spring is not already set.
- `easing(...)` disables spring behavior and uses the easing directly.
- Excluded properties are applied immediately while other properties animate.

Advanced: per-style transition override

If `animate` includes a `"transition"` property in CSS form (`all <duration> <easing> <delay>`), runtime parses and uses it for that update:

```rust
FluidStyle::new()
    .opacity(1.0)
    .with("transition", "all 180ms ease-out 40ms")
```

### `FluidSignal<T>`

`FluidSignal` lets props accept:

- static values
- closures (`Fn() -> T`)
- `Signal<T>`, `RwSignal<T>`, `Memo<T>`

Used by `animate`, `class`, and `style` props.

### `use_spring` and `SpringValue`

Use spring values for continuously retargeted values (pointer, drag, scrolling indicators).

```rust
let x = use_spring(0.0, Spring::new(600, 0.5));
x.set(120.0);
```

`SpringValue` API:

- `get()`
- `set(target)`
- `set_immediate(value)`
- `signal() -> Signal<f64>`

For spring-driven transforms, use `Transition::new().duration_ms(0)` on the motion element so you do not double-interpolate (spring + CSS transition).

### `FluidTimeline` and `FluidStep`

Use timelines when you want sequenced state independent of component internals.

```rust
let paused = RwSignal::new(false);
let controller = controller! {
    target: node_ref,
    transition: Transition::new().duration_ms(240),
    initial: FluidStyle::new().opacity(0.4),
};
let timeline = timeline! {
    controller: controller,
    initial: FluidStyle::new().opacity(0.4),
    autoplay: true,
    steps: [
        { to: FluidStyle::new().opacity(1.0) },
        { to: FluidStyle::new().x(20.0), wait_ms: 200 },
    ],
    triggers: [
        on(paused.get()) {
            true => pause(),
            false => resume(),
        },
    ],
};
```

Key API:

- `FluidStep::new(style)`
- `FluidStep::to(style)`
- `wait_ms`, `wait_for`, `on_complete`
- `FluidTimeline::new(initial)`
- `set_steps`, `bind`, `play_steps`, `play`, `restart`, `pause`, `resume`, `stop`, `set_immediate`
- `set_auto_loop`, `toggle_auto_loop`, `auto_loop`
- state signals: `is_running`, `is_paused`, `step_index`
- `signal()` for direct binding to `animate`
- `attach_node_ref(node_ref)` or `bind(controller)` for pause/resume behavior

### Controller-first macros

Use macros when you want controller-first motion wiring with minimal effect boilerplate.

```rust
let card = controller! {
    target: node_ref,
    transition: Transition::new().duration_ms(220),
    initial: collapsed_style(),
};

when! {
    controller: card,
    on(open.get()) {
        true => animate(open_style()),
        false => animate(closed_style()),
    },
}
```

`when!` only runs after the watched value changes; the initial sample is recorded but does not trigger an action.

Use `target:` for stable elements or `NodeRef`s, and `resolver:` when the active element needs to be looked up dynamically.

### Typed builders

Use builders when you want IDE-friendly method completion and compile-time install guarantees without macro syntax.

```rust
let card = AnimationController::builder()
    .target(node_ref)
    .transition(Transition::new().duration_ms(220))
    .initial(collapsed_style())
    .install();

card.on_change(move || open.get(), move |open, controller| {
    if open {
        controller.animate(open_style());
    } else {
        controller.animate(closed_style());
    }
});

let intro = FluidTimeline::builder(card)
    .initial(collapsed_style())
    .autoplay(true)
    .step(FluidStep::to(open_style()))
    .step(FluidStep::to(closed_style()).wait_ms(180))
    .install();
```

Builder install is intentionally typed:

- controller builders cannot `install()` until you call `target(...)` or `resolver(...)`
- timeline builders cannot `install()` until you add at least one `.step(...)`

When to use timeline:

- multi-step choreographies
- reusable scripted sequences
- play/pause/resume controls exposed to users

### Interruption behavior

`motion` is built to handle rapid retargeting:

- active animation is canceled safely.
- current visual state is frozen from computed styles.
- next animation starts from that frozen state.

This is what keeps repeated tab clicks, toggle spam, and hover/tap interruptions smooth instead of snapping.

## Choosing the right tool

| Use case | Preferred module |
| --- | --- |
| Animated nested route outlet transitions | `view_transitions` |
| Move/resize/reorder existing layout nodes | `flip` |
| Animate style/state of a component over time | `motion` |
| Cursor-follow or continuous target smoothing | `motion` + `use_spring` |
| Sequenced multi-step UI choreography | `motion` + `FluidTimeline` |

You can combine modules safely. Example in this repo: `example_motion` uses both `motion` and `flip`, while `example_motion_controller` focuses on controller-only motion wiring.

## Examples in this repo

### Route + motion example

```bash
cd example
trunk serve --open
```

### Motion + FLIP playground

```bash
cd example_motion
trunk serve --open
```

### Controller-only motion playground

```bash
cd example_motion_controller
trunk serve --open
```

### React + Motion parity playground

```bash
cd example_motion_controller_react
npm install
npm run dev
```

Build both React variants (`motion/react-m` and `motion/react-mini`) and compare output size against the Leptos controller demo:

```bash
cd example_motion_controller_react
npm run build:size
```

### Controller regression checks (Playwright)

```bash
cd example_motion_controller
trunk build
cd ..
cargo run -p playwright_regression_controller --
```

## Benchmarks

Motion microbenchmarks:

```bash
cargo bench -p leptos_fluid_motion --features bench
```

Covers spring stepping, `FluidStyle` prop generation, and transition CSS generation.

Release wasm size snapshots can be captured and compared with:

```bash
python3 tools/wasm_size_report.py capture \
  --repo-root . \
  --target-dir /tmp/leptos_fluid_wasm/current \
  --output /tmp/leptos_fluid_wasm/current.json
```

There is also a PR workflow in `.github/workflows/wasm-size.yml` that builds the two wasm examples with `twiggy`, compares them against the base branch, and writes the markdown table to the GitHub step summary.

## Known constraints and gotchas

- This workspace is currently configured around `leptos` CSR usage.
- `view_transitions` depends on CSS animations you provide; no default theme is included.
- `FluidOutlet` cleanup expects `animationend` events. Avoid infinite animations on outlet wrappers.
- `Flip` single-element constructor uses element `id` lookup, not CSS selector syntax.
- `FlipGroup` works best with stable `data-flip-id`; index fallback is only a safety net.
- `scale_correction_selector` should target descendants that visually distort under parent scale.
- `motion` and `flip` rely on WAAPI when available; unsupported paths degrade to immediate style application in affected branches.

## Back-compat paths

`leptos_fluid` includes a compatibility shim for older module paths:

- `leptos_fluid::animators::flip::*`
- `leptos_fluid::animators::view_transitions::*`

New code should prefer:

- `leptos_fluid::flip::*`
- `leptos_fluid::view_transitions::*`
- `leptos_fluid::motion::*`
