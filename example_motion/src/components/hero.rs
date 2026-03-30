use leptos::prelude::*;
use leptos_fluid_motion::{
    use_spring, FluidButton, FluidDiv, FluidStyle, FluidValue, Spring, Transition,
};

use super::spring_utils::lerp;

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
    let showcase_progress = use_spring(0.0, Spring::new(620, 0.34));

    Effect::new({
        let showcase_progress = showcase_progress.clone();
        move || showcase_progress.set(if energized.get() { 1.0 } else { 0.0 })
    });

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
                initial=hero_card_style(0.0)
                animate=move || hero_card_style(showcase_progress.get())
                transition=Transition::new().duration_ms(0)
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

fn hero_card_style(progress: f64) -> FluidStyle {
    let shadow_y = lerp(22.0, 34.0, progress);
    let shadow_blur = lerp(54.0, 84.0, progress);
    let shadow_alpha = lerp(0.4, 0.56, progress);

    FluidStyle::new()
        .opacity(lerp(0.86, 1.0, progress))
        .x(lerp(-12.0, 8.0, progress))
        .y(lerp(14.0, -10.0, progress))
        .scale(lerp(0.96, 1.02, progress))
        .rotate(lerp(1.4, -1.2, progress))
        .with_prop(
            "box-shadow",
            FluidValue::from(format!(
                "0 {shadow_y:.1}px {shadow_blur:.1}px rgba(6, 7, 18, {shadow_alpha:.3})"
            )),
        )
}
