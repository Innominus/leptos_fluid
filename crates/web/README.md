# leptos_fluid_web

Shared browser and WAAPI helpers used by `leptos_fluid_motion` and `leptos_fluid_flip`.

This crate exists to keep duplicated DOM/WAAPI utility logic in one place.

## Intended use

- Primary: internal dependency for the `leptos_fluid` workspace crates.
- Secondary: advanced consumers who want low-level helpers for style/property parsing and WAAPI calls.

## Exposed helpers

Examples of exported functions:

- style access (`html_style`, `computed_style`)
- keyframe/options builders (`object_from_str_pairs`, `keyframes_from_two`, `waapi_options`)
- animation controls (`animate_with_waapi`, `animation_cancel`, `animation_pause`, `animation_play`)
- value parsing/formatting helpers (`parse_js_f64`, `css_px_string`, `safe_f64_ratio`)

## Stability note

`leptos_fluid_web` is a support crate. APIs may evolve with internal runtime needs.

For stable, higher-level integration points, prefer:

- `leptos_fluid_motion`
- `leptos_fluid_flip`
- `leptos_fluid_view_transitions`
