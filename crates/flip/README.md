# leptos_fluid_flip

FLIP animations for Leptos layout changes.

Use this crate when elements are moving or resizing because of layout updates and you want smooth interpolation instead of snapping.

## Install

Via umbrella crate:

```toml
[dependencies]
leptos_fluid = { version = "0.1", features = ["flip"] }
```

Or directly:

```toml
[dependencies]
leptos_fluid_flip = "0.1"
```

## Quick start

```rust
use leptos::prelude::*;
use leptos_fluid_flip::{Flip, FlipOptions, ScaleMode};

#[component]
fn Demo() -> impl IntoView {
    let right = RwSignal::new(false);
    let pill_ref = NodeRef::<leptos::html::Div>::new();
    let flip = Flip::builder()
        .target(pill_ref)
        .options(
            FlipOptions::new()
                .duration_ms(260)
                .scale_mode(ScaleMode::PositionAndScale),
        )
        .install();

    view! {
        <button on:click=move |_| {
            flip.run(move || right.update(|v| *v = !*v));
        }>
            "Toggle"
        </button>
        <div class="lane" class:right=move || right.get()>
            <div node_ref=pill_ref>"FLIP"</div>
        </div>
    }
}
```

## API overview

- `Flip`: single element FLIP animation (`target`, `resolver`, or id lookup).
- `FlipGroup`: group FLIP animation (selector lookup).
- `FlipOptions`: duration (default `240ms`), delay, stagger, easing (default `EaseInOut`), scale mode, and optional scale correction selector.
- `Flip::builder()` / `FlipGroup::builder()`: typed install path.
- `Easing`: FLIP easing presets (`EaseInOut`, `Linear`, `Custom`).
- `ScaleMode::PositionOnly`: animate translation only.
- `ScaleMode::PositionAndScale`: animate translation + size deltas.
- `FlipValues`: measured layout snapshot type used by FLIP internals (public for inspection).

`FlipGroup` identity order:

1. `data-flip-id`
2. `id`
3. fallback index key

Prefer `data-flip-id` for stable reorders.

## Notes

- Put layout-changing mutations inside `flip.run(...)` or `flip_group.run(...)`.
- Use `scale_correction_selector` for descendants (for example text containers) that should remain visually crisp while parent scale animates.

`animate(...)` remains as a compatibility alias for `run(...)`.

For full workspace docs and examples, see the root `README.md`.
