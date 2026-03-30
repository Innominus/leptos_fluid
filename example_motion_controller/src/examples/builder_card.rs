use leptos::prelude::*;
use leptos_fluid_motion::{AnimationController, Easing, FluidStyle, Transition};

#[derive(Clone, Copy, PartialEq, Eq)]
enum BuilderCardState {
    Docked,
    Lifted,
}

#[component]
pub fn BuilderCardExample() -> impl IntoView {
    let state = RwSignal::new(BuilderCardState::Docked);
    let preview_ref = NodeRef::<leptos::html::Div>::new();
    let controller = AnimationController::builder()
        .target(preview_ref)
        .transition(Transition::new().duration_ms(240).easing(Easing::EaseInOut))
        .initial(builder_card_style(BuilderCardState::Docked))
        .install();
    let seeded = StoredValue::new(false);

    Effect::new(move || {
        if seeded.get_value() || preview_ref.get().is_none() {
            return;
        }
        seeded.set_value(true);
        controller.set_immediate(builder_card_style(state.get_untracked()));
    });

    controller.on_change(
        move || state.get(),
        move |next, controller| {
            controller.animate(builder_card_style(next));
        },
    );

    let toggle = move |_| {
        state.update(|current| {
            *current = match *current {
                BuilderCardState::Docked => BuilderCardState::Lifted,
                BuilderCardState::Lifted => BuilderCardState::Docked,
            }
        });
    };
    let reset = move |_| {
        state.set(BuilderCardState::Docked);
        controller.set_immediate(builder_card_style(BuilderCardState::Docked));
    };

    view! {
        <article class="demo-panel" data-testid="builder-card-panel">
            <div class="panel-header">
                <p class="panel-eyebrow">"Builder API"</p>
                <h2>"AnimationController::builder()"</h2>
                <p>
                    "Typed install flow on a plain div."
                </p>
            </div>

            <div class="button-row">
                <button data-testid="builder-card-toggle" on:click=toggle>
                    {move || match state.get() {
                        BuilderCardState::Docked => "Lift the card",
                        BuilderCardState::Lifted => "Dock the card",
                    }}
                </button>
                <button class="ghost" data-testid="builder-card-reset" on:click=reset>
                    "Snap reset"
                </button>
            </div>

            <div class="stage">
                <div node_ref=preview_ref class="preview-card builder-card" data-testid="builder-card-preview">
                    <p class="chip">"builder"</p>
                    <h3>"Readable install path"</h3>
                    <p data-testid="builder-card-status">
                        {move || match state.get() {
                            BuilderCardState::Docked => "Docked and ready for launch.",
                            BuilderCardState::Lifted => "Lifted into focus with a single state transition.",
                        }}
                    </p>
                </div>
            </div>
        </article>
    }
}

fn builder_card_style(state: BuilderCardState) -> FluidStyle {
    match state {
        BuilderCardState::Docked => FluidStyle::new()
            .opacity(0.78)
            .x(-18.0)
            .y(16.0)
            .scale(0.94)
            .rotate(1.6)
            .with("background", "#eef2f7")
            .with("color", "#0f172a")
            .with("border-color", "rgba(15,23,42,.12)")
            .with("box-shadow", "0 14px 28px rgba(15,23,42,.12)"),
        BuilderCardState::Lifted => FluidStyle::new()
            .opacity(1.0)
            .x(0.0)
            .y(0.0)
            .scale(1.02)
            .rotate(-1.0)
            .with("background", "#0f766e")
            .with("color", "#ecfeff")
            .with("border-color", "rgba(103,232,249,.4)")
            .with("box-shadow", "0 28px 56px rgba(8,47,73,.28)"),
    }
}
