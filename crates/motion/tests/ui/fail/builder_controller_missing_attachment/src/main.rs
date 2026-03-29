use leptos_fluid_motion::{AnimationController, FluidStyle};

fn main() {
    let _ = AnimationController::builder()
        .initial(FluidStyle::new())
        .install();
}
