use leptos::prelude::*;
use leptos_fluid_motion::{controller, timeline, FluidStyle, Transition};

fn main() {
    let node_ref = NodeRef::<leptos::html::Div>::new();
    let controller = controller! {
        target: node_ref,
        transition: Transition::spring(),
    };
    let _ = timeline! {
        controller: controller,
        steps: [
            { to: FluidStyle::new(), delay_ms: 120 },
        ],
    };
}
