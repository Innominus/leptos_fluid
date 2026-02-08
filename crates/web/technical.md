# leptos_fluid_web technical.md

This document describes the low-level helper layer used by `leptos_fluid_motion` and `leptos_fluid_flip`.

## Purpose

`leptos_fluid_web` centralizes browser interop logic so higher-level crates can focus on animation/runtime policy instead of JS reflection and DOM plumbing.

The crate is intentionally lightweight and mostly function-based.

## Module layout

- `src/lib.rs`: all helpers live in one file

Helpers are grouped by concern:

- DOM style access
- JS object/keyframe construction
- WAAPI invocation and animation controls
- numeric parsing/serialization
- per-element active animation association

## DOM and style helpers

Key functions:

- `html_style(element) -> Option<CssStyleDeclaration>`
- `computed_style(element) -> Option<CssStyleDeclaration>`
- `restore_inline_property(style, property, value)`
- `node_list_to_elements(NodeList) -> Vec<Element>`

Design notes:

- APIs return `Option` instead of panicking on missing window/style handles.
- conversion utilities skip non-element nodes quietly.
- restore helper removes a property when stored value is empty, preserving pre-animation inline state semantics.

## JS object/keyframe construction

The crate wraps repetitive `js_sys::Object` + `Reflect::set` work:

- `object_set_string`
- `object_set_f64`
- `object_from_str_pairs`
- `keyframes_from_two`
- `waapi_options`

This keeps WAAPI call sites in motion/flip concise and consistent.

## WAAPI invocation and controls

`animate_with_waapi` is deliberately reflective:

1. fetch `element.animate` via `Reflect`
2. cast to `Function`
3. call with keyframes/options
4. cast result to `web_sys::Animation`

If any step fails, returns `None`.

Companion helpers (`animation_cancel`, `animation_commit_styles`, `animation_set_onfinish`, `animation_pause`, `animation_play`) isolate minor API-compatibility differences and call-failure handling.

### Why `commitStyles` reflection is separate

`Animation::commitStyles` is not uniformly available across browser engines. `animation_commit_styles` probes method presence and returns `false` when unsupported.

Higher layers then decide fallback behavior.

## Numeric and CSS serialization helpers

Functions:

- `js_number_to_string`
- `css_push_number`
- `css_push_px`
- `css_px_string`
- `parse_js_f64`
- `safe_f64_ratio`

These provide stable formatting and guard rails around NaN/inf/div-by-zero behavior.

`safe_f64_ratio` returns `1.0` on invalid denominator/result, which is a safe neutral factor for scale calculations.

## Active animation attachment model

The crate stores a runtime animation handle directly on the DOM element using a hidden JS key:

- key constant: `__fluidActiveAnimation`
- `element_set_active_animation`
- `element_get_active_animation`

This allows motion runtime to retrieve/cancel in-flight animations tied to a specific element without maintaining a global map.

## Error-handling policy

Most helpers are best-effort and non-throwing:

- `Reflect::set` errors are ignored
- invalid casts return `None`
- missing methods degrade to fallback behavior

That policy is intentional for UI animation code where resilience is preferred over strict failure.

## Guidance for contributors

When adding new helpers:

1. keep functions side-effect scoped and single-purpose
2. prefer `Option`/`bool` return values over panics
3. keep naming explicit (`object_*`, `animation_*`, `css_*`)
4. avoid leaking high-level policy into this crate

If a helper starts requiring app-specific behavior, it probably belongs in `motion` or `flip` instead.
