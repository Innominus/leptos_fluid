use leptos::prelude::*;
use leptos_fluid_motion::{Easing, FluidButton, FluidDiv, FluidStyle, FluidValue, Transition};

const HERO_PILLARS: [(&str, &str); 4] = [
    (
        "Wrapper components",
        "FluidDiv, FluidButton, and FluidSpan for expressive markup.",
    ),
    (
        "Raw styling",
        "style! plus FluidStyle builders for intentional surfaces.",
    ),
    (
        "Sequencing",
        "Timeline and spring-driven motion for continuous interaction.",
    ),
    (
        "Layout transitions",
        "FLIP primitives for move, resize, and reorder cases.",
    ),
];

#[component]
pub fn HeroSection() -> impl IntoView {
    let energized = RwSignal::new(false);

    let pillars = HERO_PILLARS
        .into_iter()
        .map(|(label, body)| {
            view! {
                <div class="summary-item">
                    <p class="summary-kicker">{label}</p>
                    <p class="summary-body">{body}</p>
                </div>
            }
        })
        .collect_view();

    view! {
        <section class="hero-grid">
            <div class="panel hero-copy">
                <p class="kicker">"Leptos Fluid"</p>
                <h1>"A rebuilt motion showroom for the whole stack."</h1>
                <p class="lead-copy">
                    "This example focuses on the full motion surface: wrapper components, style composition, timelines, springs, and FLIP layout animation."
                </p>

                <div class="button-row">
                    <button on:click=move |_| energized.update(|value| *value = !*value)>
                        {move || if energized.get() { "Return to dock" } else { "Energize showcase" }}
                    </button>
                    <button class="alt" on:click=move |_| energized.set(false)>
                        "Reset"
                    </button>
                </div>

                <div class="summary-grid">{pillars}</div>
            </div>

            <FluidDiv
                class="hero-showcase"
                initial=hero_card_style(false)
                animate=move || hero_card_style(energized.get())
                transition=Transition::new().duration_ms(260).easing(Easing::EaseInOut)
                while_hover=FluidStyle::new().scale(1.01).y(-6.0)
                while_tap=FluidStyle::new().scale(0.99)
            >
                <div class="hero-orb hero-orb-left"></div>
                <div class="hero-orb hero-orb-right"></div>
                <p class="chip">"Motion stack"</p>
                <h2>"One surface, multiple layers"</h2>
                <p>
                    "Wrapper components read beautifully, builders keep plain elements ergonomic, and FLIP handles layout changes without losing continuity."
                </p>
                <FluidButton
                    class="hero-cta"
                    initial=FluidStyle::new().opacity(0.0).y(10.0)
                    animate=FluidStyle::new().opacity(1.0).y(0.0)
                    transition=Transition::snappy()
                    while_hover=FluidStyle::new().scale(1.04)
                    while_tap=FluidStyle::new().scale(0.96)
                >
                    "FluidButton in the hero"
                </FluidButton>
            </FluidDiv>
        </section>
    }
}

fn hero_card_style(energized: bool) -> FluidStyle {
    if energized {
        FluidStyle::new()
            .opacity(1.0)
            .x(8.0)
            .y(-10.0)
            .scale(1.02)
            .rotate(-1.2)
            .with_prop(
                "box-shadow",
                FluidValue::from("0 34px 84px rgba(6, 7, 18, 0.56)"),
            )
    } else {
        FluidStyle::new()
            .opacity(0.86)
            .x(-12.0)
            .y(14.0)
            .scale(0.96)
            .rotate(1.4)
            .with_prop(
                "box-shadow",
                FluidValue::from("0 22px 54px rgba(6, 7, 18, 0.4)"),
            )
    }
}
