use leptos::prelude::*;
use leptos_fluid_motion::{use_spring, AnimationController, FluidStyle, Spring, Transition};

use super::spring_utils::lerp;

#[derive(Clone, Copy, PartialEq, Eq)]
enum BuilderCardState {
    Docked,
    Lifted,
}

#[component]
pub fn BuilderCardExample() -> impl IntoView {
    let state = RwSignal::new(BuilderCardState::Docked);
    let preview_ref = NodeRef::<leptos::html::Div>::new();
    let card_progress = use_spring(0.0, Spring::new(580, 0.32));

    Effect::new({
        let card_progress = card_progress.clone();
        move || {
            card_progress.set(match state.get() {
                BuilderCardState::Docked => 0.0,
                BuilderCardState::Lifted => 1.0,
            })
        }
    });

    let animate_progress = card_progress.clone();
    let controller = AnimationController::builder()
        .target(preview_ref)
        .transition(Transition::new().duration_ms(0))
        .initial(builder_card_style(0.0))
        .animate(move || builder_card_style(animate_progress.get()))
        .install();
    let seeded = StoredValue::new(false);

    let seeded_progress = card_progress.clone();
    Effect::new(move || {
        if seeded.get_value() || preview_ref.get().is_none() {
            return;
        }
        seeded.set_value(true);
        controller.set_immediate(builder_card_style(seeded_progress.get()));
    });

    let toggle = move |_| {
        state.update(|current| {
            *current = match *current {
                BuilderCardState::Docked => BuilderCardState::Lifted,
                BuilderCardState::Lifted => BuilderCardState::Docked,
            }
        });
    };
    let reset_progress = card_progress.clone();
    let reset = move |_| {
        state.set(BuilderCardState::Docked);
        reset_progress.set_immediate(0.0);
        controller.set_immediate(builder_card_style(0.0));
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

fn builder_card_style(progress: f64) -> FluidStyle {
    let shadow_y = lerp(14.0, 28.0, progress);
    let shadow_blur = lerp(28.0, 56.0, progress);
    let shadow_alpha = lerp(0.12, 0.28, progress);
    let background = if progress >= 0.5 {
        "#0f766e"
    } else {
        "#eef2f7"
    };
    let color = if progress >= 0.5 {
        "#ecfeff"
    } else {
        "#0f172a"
    };
    let border_alpha = lerp(0.12, 0.4, progress);

    FluidStyle::new()
        .opacity(lerp(0.78, 1.0, progress))
        .x(lerp(-18.0, 0.0, progress))
        .y(lerp(16.0, 0.0, progress))
        .scale(lerp(0.94, 1.02, progress))
        .rotate(lerp(1.6, -1.0, progress))
        .with("background", background)
        .with("color", color)
        .with(
            "border-color",
            format!("rgba(103,232,249,{border_alpha:.3})"),
        )
        .with(
            "box-shadow",
            format!("0 {shadow_y:.1}px {shadow_blur:.1}px rgba(8,47,73,{shadow_alpha:.3})"),
        )
}
