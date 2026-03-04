# leptos_fluid_motion technical.md

This document describes how `leptos_fluid_motion` works internally so contributors can reason about behavior changes safely.

## Module map

- `src/lib.rs`: export surface and prelude
- `src/controller.rs`: element-agnostic animation controller API
- `src/animator.rs`: shared WAAPI animation runtime used by controllers/components
- `src/components.rs`: motion components built on top of `AnimationController`
- `src/style.rs`: style builder and transform composition
- `src/transition.rs`: transition/spring configuration and easing generation
- `src/spring_value.rs`: spring solver for continuously retargeted scalar values
- `src/timeline.rs`: timeline sequencing state machine
- `src/signal.rs`: signal adapter used by component props
- `src/timing.rs`: frame-based scheduling helper
- `src/spring_math.rs`: shared small math helpers

## Core runtime model

`AnimationController` is the core runtime handle. `FluidElement` and wrappers (`FluidDiv`, `FluidSpan`, `FluidButton`) are declarative adapters over that controller.

Each motion update follows this shape:

1. derive target `FluidStyle` from `FluidSignal`
2. split into animated vs immediate props (`Transition::excluded_properties`)
3. cancel/freeze previous animation if present
4. build WAAPI keyframes from current frame to next frame
5. run animation and commit final inline props on finish

### Controller target model

`AnimationController` can target:

- a concrete DOM `Element`
- a resolver closure returning `Option<Element>` (for ref-driven attachment)

If the target is unresolved, the controller stores only the latest pending command and replays it when a target becomes available.

### Why interruption logic exists

Users can retarget quickly (hover enter/leave, rapid state toggles, timeline jumps). If we simply cancel and restart from static style state, visible snapping occurs.

To avoid this, the runtime:

- attempts `animation.commitStyles()`
- falls back to computed-style freezing where needed
- writes frozen values inline before starting next animation

This is the critical continuity mechanism.

## Style representation (`FluidStyle`)

`FluidStyle` stores:

- arbitrary key/value CSS pairs (`props`)
- structured transform components (`Transform`)

Transform helpers (`x`, `y`, `scale`, `rotate`) are composed in a deterministic order:

1. `translate3d`
2. `scale`
3. `rotate`

If the caller explicitly sets `transform`, auto composition is skipped.

### Why this design

- type-safe common operations for frequent motion paths
- still supports uncommon CSS properties without API explosion
- predictable transform order prevents subtle mismatch across updates

## Transition model (`Transition`, `Spring`, `Easing`)

`Transition` is a runtime config object, not a state machine.

Important behavior:

- spring transitions are represented as a generated `linear(...)` easing curve
- `bounce(...)` will create spring metadata if missing
- calling `.easing(...)` clears spring metadata
- excluded properties are applied immediately while others animate

The system also supports per-style transition overrides by parsing a `transition` CSS value from `FluidStyle`.

## Spring values (`use_spring`, `SpringValue`)

`SpringValue` is a frame-driven integrator for continuous retargeting (for example pointer follow).

Loop details:

- scheduled via `request_animation_frame`
- `dt` clamped to a safe range (`0.016 ..= 0.05`) for stability
- solver stops when both displacement and velocity are below `rest_delta`

This is separate from component `Transition`: spring values are usually paired with `Transition::new().duration_ms(0)` to avoid double smoothing.

## Timeline model (`FluidTimeline`)

`FluidTimeline` is a small sequencer around `Vec<FluidStep>`.

Internal controls:

- `generation` counter invalidates stale scheduled callbacks
- running/paused signals expose external state
- optional attached `node_ref` allows pausing/resuming the active WAAPI animation

Step execution model:

1. set step style into timeline value signal
2. wait `step.wait_ms` (or chain immediately for zero wait)
3. call optional `on_complete`
4. proceed to next step

`auto_loop` restarts from step `0` when a run completes.

## Signal adaptation (`FluidSignal<T>`)

`FluidSignal<T>` is an acceptance-layer type so component props can consume:

- static values
- closures
- `Signal<T>`
- `RwSignal<T>`
- `Memo<T>`

This avoids many overloads in component props while keeping type signatures strict.

## Pointer interaction layers

Component runtime tracks two transient states:

- hover state (`while_hover`)
- pressed state (`while_tap`)

Priority model:

1. pressed style while pointer is down
2. hover style while hovering and not pressed
3. base `animate` style otherwise

This is why release handlers recompute target from hover/base state.

## Browser/WAAPI assumptions

The runtime relies on WAAPI when available.

Fallback behavior when WAAPI cannot run for a path:

- immediate style application instead of animated interpolation

This keeps behavior correct even when animation fidelity is reduced.

## Contributor guidance

When changing motion behavior, validate these cases explicitly:

- rapid retargeting during active animation
- mount-time initial + animate behavior
- excluded-property behavior
- transform interpolation from/to `none`
- pointer enter/leave/down/up cancellation order

Recommended local checks:

```bash
cargo fmt --all
cargo check --workspace
```

For runtime behavior, run `example_motion` and test interruption-heavy interactions (tabs, timeline controls, hover/tap churn).
