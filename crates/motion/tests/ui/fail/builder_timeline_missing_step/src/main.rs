use leptos::prelude::*;
use leptos_fluid_motion::{AnimationController, FluidTimeline, Transition};

fn main() {
    let node_ref = NodeRef::<leptos::html::Div>::new();
    let controller = AnimationController::builder()
        .target(node_ref)
        .transition(Transition::spring())
        .install();

    let _ = FluidTimeline::builder(controller).install();
}
