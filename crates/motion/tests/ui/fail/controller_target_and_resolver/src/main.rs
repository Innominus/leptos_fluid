use leptos::prelude::*;
use leptos_fluid_motion::{controller, Transition};

fn main() {
    let node_ref = NodeRef::<leptos::html::Div>::new();
    let _ = controller! {
        target: node_ref,
        resolver: move || None,
        transition: Transition::spring(),
    };
}
