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
    let flip = Flip::new_with_options(
        "flip-pill".to_string(),
        FlipOptions {
            duration: 600,
            scale_mode: ScaleMode::PositionAndScale,
            ..Default::default()
        },
    );

    view! {
        <button on:click=move |_| {
            flip.animate(move || right.update(|v| *v = !*v));
        }>
            "Toggle"
        </button>
        <div class="lane" class:right=move || right.get()>
            <div id="flip-pill">"FLIP"</div>
        </div>
    }
}
```

## API overview

- `Flip`: single element FLIP animation (id-based lookup).
- `FlipGroup`: group FLIP animation (CSS selector lookup).
- `FlipOptions`: duration, delay, stagger, easing, scale mode, and optional scale correction selector.
- `Easing`: FLIP easing presets (`Linear`, `EaseInOut`, `Custom`).
- `ScaleMode::PositionOnly`: animate translation only.
- `ScaleMode::PositionAndScale`: animate translation + size deltas.
- `FlipValues`: measured layout snapshot type used by FLIP internals (public for inspection).

`FlipGroup` identity order:

1. `data-flip-id`
2. `id`
3. fallback index key

Prefer `data-flip-id` for stable reorders.

## Notes

- Put layout-changing mutations inside `flip.animate(...)` or `flip_group.animate(...)`.
- Use `scale_correction_selector` for descendants (for example text containers) that should remain visually crisp while parent scale animates.

For full workspace docs and examples, see the root `README.md`.
