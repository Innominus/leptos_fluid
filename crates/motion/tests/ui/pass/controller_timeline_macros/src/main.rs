use leptos::prelude::*;
use leptos_fluid_motion::{controller, timeline, when, FluidStyle, Transition};

fn card_style(active: bool) -> FluidStyle {
    if active {
        FluidStyle::new().opacity(1.0).scale(1.0)
    } else {
        FluidStyle::new().opacity(0.5).scale(0.94)
    }
}

fn main() {
    let node_ref = NodeRef::<leptos::html::Div>::new();
    let active = RwSignal::new(false);
    let paused = RwSignal::new(false);

    let controller = controller! {
        target: node_ref,
        transition: Transition::new(),
        initial: card_style(false),
    };

    let timeline = timeline! {
        controller: controller,
        initial: card_style(false),
        autoplay: true,
        steps: [
            { to: card_style(true) },
            { to: card_style(false), wait_ms: 180 },
        ],
        triggers: [
            on(paused.get()) {
                true => pause(),
                false => resume(),
            },
        ],
    };

    when! {
        controller: controller,
        on(active.get()) {
            true => animate(card_style(true)),
            false => animate(card_style(false)),
        },
    }

    let _ = timeline;
}
