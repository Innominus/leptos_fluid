use leptos::prelude::*;
use leptos_fluid_motion::{
    AnimationController, FluidStep, FluidStyle, FluidTimeline, Transition,
};

fn style(active: bool) -> FluidStyle {
    if active {
        FluidStyle::new().opacity(1.0).scale(1.0)
    } else {
        FluidStyle::new().opacity(0.6).scale(0.94)
    }
}

fn main() {
    let node_ref = NodeRef::<leptos::html::Div>::new();
    let active = RwSignal::new(false);
    let paused = RwSignal::new(false);

    let controller = AnimationController::builder()
        .target(node_ref)
        .transition(Transition::new())
        .initial(style(false))
        .animate(move || style(active.get()))
        .install();

    controller.on_change(
        move || active.get(),
        move |value, controller| {
            controller.animate(style(value));
        },
    );

    let _timeline = FluidTimeline::builder(controller)
        .initial(style(false))
        .autoplay(true)
        .step(FluidStep::to(style(true)))
        .step(FluidStep::to(style(false)).wait_ms(180))
        .on_change(
            move || paused.get(),
            move |value, timeline| {
                if value {
                    timeline.pause();
                } else {
                    timeline.resume();
                }
            },
        )
        .install();
}
