use leptos::prelude::*;
use leptos_fluid_motion::{style, Easing, FluidElement, FluidStyle, Transition};

struct Mood {
    label: &'static str,
    note: &'static str,
    background: &'static str,
    border: &'static str,
    shadow: &'static str,
}

const MOODS: [Mood; 3] = [
    Mood {
        label: "Signal room",
        note: "Soft cyan glow with steady contrast.",
        background: "linear-gradient(145deg, rgba(14, 116, 144, 0.95), rgba(8, 47, 73, 0.92))",
        border: "rgba(103, 232, 249, 0.36)",
        shadow: "0 28px 68px rgba(8, 47, 73, 0.46)",
    },
    Mood {
        label: "Editorial bloom",
        note: "Warm editorial palette driven through style!.",
        background: "linear-gradient(145deg, rgba(217, 119, 6, 0.94), rgba(124, 45, 18, 0.92))",
        border: "rgba(253, 186, 116, 0.4)",
        shadow: "0 28px 68px rgba(124, 45, 18, 0.42)",
    },
    Mood {
        label: "Broadcast mode",
        note: "High-contrast magenta and cobalt mix.",
        background: "linear-gradient(145deg, rgba(190, 24, 93, 0.92), rgba(30, 64, 175, 0.92))",
        border: "rgba(244, 114, 182, 0.42)",
        shadow: "0 30px 72px rgba(30, 64, 175, 0.36)",
    },
];

#[component]
pub fn StyleLabSection() -> impl IntoView {
    let active_mood = RwSignal::new(0usize);

    view! {
        <section class="section-grid">
            <div class="panel">
                <p class="section-kicker">"Style lab"</p>
                <h2>"FluidStyle + style! on the raw primitive"</h2>
                <p>
                    "FluidElement stays close to the bare primitive. Switch moods to see typed transform helpers and raw CSS properties compose together."
                </p>
                <div class="button-row">
                    <button class="alt" on:click=move |_| active_mood.set(0)>
                        "Signal room"
                    </button>
                    <button class="alt" on:click=move |_| active_mood.set(1)>
                        "Editorial bloom"
                    </button>
                    <button class="alt" on:click=move |_| active_mood.set(2)>
                        "Broadcast mode"
                    </button>
                </div>
            </div>

            <FluidElement
                tag="article"
                class="style-preview"
                initial=mood_style(0)
                animate=move || mood_style(active_mood.get())
                transition=Transition::new().duration_ms(240).easing(Easing::EaseInOut)
                while_hover=FluidStyle::new().scale(1.01).y(-6.0)
            >
                <p class="chip">"style!"</p>
                <h3>{move || MOODS[active_mood.get()].label}</h3>
                <p>{move || MOODS[active_mood.get()].note}</p>
                <div class="swatch-row">
                    <span class="swatch"></span>
                    <span class="swatch"></span>
                    <span class="swatch"></span>
                </div>
            </FluidElement>
        </section>
    }
}

fn mood_style(index: usize) -> FluidStyle {
    let mood = &MOODS[index % MOODS.len()];
    style!(
        "background" => mood.background,
        "border-color" => mood.border,
        "box-shadow" => mood.shadow,
    )
    .opacity(1.0)
    .scale(1.0)
    .y(0.0)
}
