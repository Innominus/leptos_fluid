use leptos::prelude::*;
use leptos_fluid_motion::{controller, when, FluidStyle, Transition};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Standby,
    Review,
    Live,
}

#[component]
pub fn MacroStateExample() -> impl IntoView {
    let mode = RwSignal::new(Mode::Standby);
    let preview_ref = NodeRef::<leptos::html::Div>::new();
    let controller = controller! {
        target: preview_ref,
        transition: Transition::spring_with(540, 0.25),
        initial: mode_style(Mode::Standby),
    };
    let seeded = StoredValue::new(false);

    Effect::new(move || {
        if seeded.get_value() || preview_ref.get().is_none() {
            return;
        }
        seeded.set_value(true);
        controller.set_immediate(mode_style(mode.get_untracked()));
    });

    when! {
        controller: controller,
        on(mode.get()) {
            Mode::Standby => animate(mode_style(Mode::Standby)),
            Mode::Review => animate(mode_style(Mode::Review)),
            Mode::Live => animate(mode_style(Mode::Live)),
        },
    }

    view! {
        <article class="demo-panel" data-testid="macro-state-panel">
            <div class="panel-header">
                <p class="panel-eyebrow">"Macro DSL"</p>
                <h2>"controller! + when!"</h2>
                <p>
                    "A small match-style state machine."
                </p>
            </div>

            <div class="button-row segmented-row">
                <button
                    class:ghost=move || mode.get() != Mode::Standby
                    data-testid="macro-state-standby"
                    on:click=move |_| mode.set(Mode::Standby)
                >
                    "Standby"
                </button>
                <button
                    class:ghost=move || mode.get() != Mode::Review
                    data-testid="macro-state-review"
                    on:click=move |_| mode.set(Mode::Review)
                >
                    "Review"
                </button>
                <button
                    class:ghost=move || mode.get() != Mode::Live
                    data-testid="macro-state-live"
                    on:click=move |_| mode.set(Mode::Live)
                >
                    "Live"
                </button>
            </div>

            <div class="stage">
                <div node_ref=preview_ref class="preview-card macro-state-card" data-testid="macro-state-preview">
                    <p class="chip">"macro"</p>
                    <h3>"Explicit state matches"</h3>
                    <p data-testid="macro-state-status">{move || mode_label(mode.get())}</p>
                </div>
            </div>
        </article>
    }
}

fn mode_label(mode: Mode) -> &'static str {
    match mode {
        Mode::Standby => "Standby: waiting for a signal.",
        Mode::Review => "Review: centered for inspection.",
        Mode::Live => "Live: bright, elevated, and broadcasting.",
    }
}

fn mode_style(mode: Mode) -> FluidStyle {
    match mode {
        Mode::Standby => FluidStyle::new()
            .opacity(0.76)
            .x(-10.0)
            .y(12.0)
            .scale(0.94)
            .rotate(1.5)
            .with("background", "#e2e8f0")
            .with("color", "#0f172a")
            .with("box-shadow", "0 12px 26px rgba(15,23,42,.12)"),
        Mode::Review => FluidStyle::new()
            .opacity(0.94)
            .x(0.0)
            .y(0.0)
            .scale(1.0)
            .rotate(0.0)
            .with("background", "#ffedd5")
            .with("color", "#7c2d12")
            .with("box-shadow", "0 20px 40px rgba(154,52,18,.18)"),
        Mode::Live => FluidStyle::new()
            .opacity(1.0)
            .x(12.0)
            .y(-8.0)
            .scale(1.04)
            .rotate(-2.0)
            .with("background", "#2563eb")
            .with("color", "#eff6ff")
            .with("box-shadow", "0 30px 56px rgba(30,64,175,.28)"),
    }
}
