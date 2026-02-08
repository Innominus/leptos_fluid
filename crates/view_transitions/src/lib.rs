//! Nested route outlet transitions for `leptos_router`.

mod fluid_manager;
mod fluid_outlet;
mod fluid_route;
mod utils;

/// Transition manager shared through Leptos context.
pub use fluid_manager::FluidManager;
/// Outlet replacement that renders intro/outro route layers.
pub use fluid_outlet::FluidOutlet;
/// Route wrapper that captures route patterns and forwards to `Routes`.
pub use fluid_route::FluidRoutes;
