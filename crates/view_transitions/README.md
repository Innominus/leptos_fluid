# leptos_fluid_view_transitions

Nested outlet route transitions for `leptos_router`.

This crate adds animated intro/outro transitions between nested route outlets while preserving a mostly standard router setup.

It also includes flat-route wrappers for apps that use `FlatRoutes` instead of nested outlets:

- `FluidFlatRoutes`
- `FluidFlatOutlet`

## Install

Via umbrella crate:

```toml
[dependencies]
leptos_fluid = { version = "0.1", features = ["view-transitions"] }
```

Or directly:

```toml
[dependencies]
leptos_fluid_view_transitions = "0.1"
```

## Required wiring

1. Provide a `FluidManager` at app root.
2. Wrap your route tree in `FluidRoutes`.
3. Use `FluidOutlet` instead of `Outlet`.

```rust
use leptos::prelude::*;
use leptos_fluid_view_transitions::{FluidManager, FluidOutlet, FluidRoutes};
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
    view! { <FluidOutlet intro_class="route-enter" outro_class="route-exit" /> }
}
```

## CSS contract

Provide the animation classes yourself:

```css
.route-enter { animation: route-enter 350ms ease; }
.route-exit { animation: route-exit 350ms ease; }
```

Transition cleanup depends on `animationend`, so outlet classes must trigger real CSS animations.

## Notes

- Direction is inferred from generated route ordering.
- Mark scrollable containers with `data-scrollable` to preserve their scroll offsets during outlet transitions.
- `FluidOutlet` sets `data-reverse` and transition classes on wrapper layers, and re-applies routed content attributes onto outro nodes by deep-cloning the intro subtree.

## Flat route support

Use `FluidFlatRoutes` and `FluidFlatOutlet` when your router tree is flat rather than nested.

The flat wrappers keep the same CSS contract and `FluidManager` requirements, but they mirror `leptos_router::FlatRoutes` instead of `Routes` + `Outlet`.

For full workspace docs and examples, see the root `README.md`.
