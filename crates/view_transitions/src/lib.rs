#[cfg(not(web_sys_unstable_apis))]
compile_error!(
    "leptos_fluid_view_transitions requires web_sys_unstable_apis. Build with RUSTFLAGS=\"--cfg=web_sys_unstable_apis\"."
);

mod fluid_manager;
mod fluid_outlet;
mod fluid_route;
mod utils;

pub use {
    fluid_manager::FluidManager,
    fluid_outlet::{FluidFlatOutlet, FluidOutlet},
    fluid_route::{FluidFlatRoutes, FluidRoutes},
};
