# leptos_fluid_example technical.md

This document explains how the `example` crate is wired and what it is intended to validate.

## Purpose

`example` is the integration demo for:

- nested route transitions from `leptos_fluid::view_transitions`
- basic motion usage from `leptos_fluid::motion`

It focuses on route/outlet behavior and nested outlet hierarchy rather than visual polish.

## Entrypoints

- `src/main.rs`: CSR mount entrypoint
- `src/app.rs`: top-level router and context setup

`main.rs` mounts `App` and enables `console_error_panic_hook` in debug builds.

## Router and transition wiring

`src/app.rs` is the critical integration surface.

It does three required steps for route transitions:

1. `provide_context(FluidManager::new())`
2. wraps routes with `FluidRoutes`
3. uses `Overlay` (which renders `FluidOutlet`) as parent route view

The route tree intentionally includes nested `ParentRoute` levels and a dynamic param segment (`ParamSegment("id")`) to stress transition direction and cache hierarchy logic.

## Overlay/outlet structure

`src/components/overlay.rs` provides:

- navigation controls to jump between top/middle/deep routes
- one `FluidOutlet` configured with:
  - `intro_class = "fly-up-transition"`
  - `outro_class = "scale-down-transition"`

This component is intentionally simple so transition behavior is easy to observe.

## Scroll restoration test surface

`src/components/common.rs` defines `PageShell` with `data-scrollable` and constrained height. This is deliberate: it validates manager scroll snapshot/restore behavior during outlet cloning.

## Pages

- `pages/home/view.rs`: basic `FluidDiv` state animation + hover/tap variants
- `pages/motion/view.rs`: richer style composition (`box-shadow`, glow layer)
- `pages/about/view.rs`: minimal static content route

The page set is intentionally uneven (static + animated) to catch transition cleanup and mount timing issues.

## CSS contract for transitions

Route transition classes are defined in `input.css`:

- `.fly-up-transition`
- `.scale-down-transition`

If these classes stop using real CSS animations, `FluidOutlet` cleanup will not receive `animationend` and transitions can stall.

## Contributor workflow

Use this crate when changing:

- `view_transitions` manager/outlet behavior
- nested route matching/direction logic
- scroll preservation behavior

Suggested manual checks:

1. Navigate between top-level routes repeatedly.
2. Navigate through `/new-route/...` nested paths.
3. Use browser back/forward across nested routes.
4. Scroll inside `PageShell` before navigating and verify offsets are preserved.
5. Confirm intro/outro classes visibly switch on backward navigation.
