use leptos::prelude::*;
use leptos_fluid_motion::{AnimationController, FluidStyle, Transition};

#[derive(Clone, Copy, PartialEq, Eq)]
enum SpringLane {
    Left,
    Center,
    Right,
}

#[component]
pub fn SpringRetargetExample() -> impl IntoView {
    let lane = RwSignal::new(SpringLane::Center);
    let preview_ref = NodeRef::<leptos::html::Div>::new();
    let controller = AnimationController::builder()
        .target(preview_ref)
        .transition(Transition::spring_with(540, 0.48))
        .initial(spring_retarget_style(SpringLane::Center))
        .install();
    let seeded = StoredValue::new(false);

    Effect::new(move || {
        if seeded.get_value() || preview_ref.get().is_none() {
            return;
        }
        seeded.set_value(true);
        controller.set_immediate(spring_retarget_style(lane.get_untracked()));
    });

    controller.on_change(
        move || lane.get(),
        move |next, controller| {
            controller.animate(spring_retarget_style(next));
        },
    );

    view! {
        <article class="demo-panel" data-testid="spring-retarget-panel">
            <div class="panel-header">
                <p class="panel-eyebrow">"Spring retarget"</p>
                <h2>"AnimationController with Transition::spring_with(...)"</h2>
                <p>
                    "Redirect the same controller while it is moving. The stronger bounce here makes the preserved momentum easier to see."
                </p>
            </div>

            <div class="button-row segmented-row">
                <button class:ghost=move || lane.get() != SpringLane::Left on:click=move |_| lane.set(SpringLane::Left)>
                    "Left"
                </button>
                <button class:ghost=move || lane.get() != SpringLane::Center on:click=move |_| lane.set(SpringLane::Center)>
                    "Center"
                </button>
                <button class:ghost=move || lane.get() != SpringLane::Right on:click=move |_| lane.set(SpringLane::Right)>
                    "Right"
                </button>
            </div>

            <div class="stage spring-retarget-stage">
                <div class="spring-retarget-lanes">
                    <span></span>
                    <span></span>
                    <span></span>
                </div>
                <div node_ref=preview_ref class="preview-card spring-retarget-card" data-testid="spring-retarget-preview">
                    <p class="chip">"controller"</p>
                    <h3>"Retarget me mid-flight"</h3>
                    <p data-testid="spring-retarget-status">
                        {move || match lane.get() {
                            SpringLane::Left => "Springing toward the left lane.",
                            SpringLane::Center => "Holding the center lane.",
                            SpringLane::Right => "Springing toward the right lane.",
                        }}
                    </p>
                </div>
            </div>
        </article>
    }
}

fn spring_retarget_style(lane: SpringLane) -> FluidStyle {
    match lane {
        SpringLane::Left => FluidStyle::new()
            .x(-136.0)
            .y(16.0)
            .scale(0.9)
            .rotate(-11.0)
            .opacity(0.7),
        SpringLane::Center => FluidStyle::new()
            .x(0.0)
            .y(-16.0)
            .scale(1.06)
            .rotate(0.0)
            .opacity(1.0),
        SpringLane::Right => FluidStyle::new()
            .x(136.0)
            .y(16.0)
            .scale(0.9)
            .rotate(11.0)
            .opacity(0.7),
    }
}
