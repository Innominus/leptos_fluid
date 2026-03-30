use leptos::prelude::*;
use leptos_fluid_motion::{
    use_spring, FluidButton, FluidDiv, FluidSpan, FluidStyle, Spring, Transition,
};

use super::spring_utils::lerp;

const WRAPPER_TAGS: [&str; 4] = ["FluidDiv", "FluidButton", "FluidSpan", "while_hover"];

#[component]
pub fn WrapperGallerySection() -> impl IntoView {
    let spotlight = RwSignal::new(false);
    let surface_progress = use_spring(0.0, Spring::new(580, 0.3));

    Effect::new({
        let surface_progress = surface_progress.clone();
        move || surface_progress.set(if spotlight.get() { 1.0 } else { 0.0 })
    });

    let chips = WRAPPER_TAGS
        .into_iter()
        .enumerate()
        .map(|(index, label)| {
            view! {
                <FluidSpan
                    class="chip"
                    initial=FluidStyle::new().opacity(0.0).x(-14.0)
                    animate=FluidStyle::new().opacity(1.0).x(0.0)
                    transition=Transition::new().duration_ms(180).delay_ms(32 * index as u32)
                >
                    {label}
                </FluidSpan>
            }
        })
        .collect_view();

    view! {
        <section class="section-grid">
            <div class="panel">
                <p class="section-kicker">"Wrapper gallery"</p>
                <h2>"Readable component-level motion"</h2>
                <p>
                    "These wrappers stay close to normal Leptos markup while still giving you hover, tap, and reactive animate styles."
                </p>
                <div class="button-row">
                    <button on:click=move |_| spotlight.update(|value| *value = !*value)>
                        {move || if spotlight.get() { "Cool the gallery" } else { "Spotlight the gallery" }}
                    </button>
                </div>
            </div>

            <div class="wrapper-grid">
                <FluidDiv
                    class="wrapper-card"
                    initial=wrapper_surface_style(0.0)
                    animate=move || wrapper_surface_style(surface_progress.get())
                    transition=Transition::new().duration_ms(0)
                    while_hover=FluidStyle::new().scale(1.02).y(-6.0)
                >
                    <p class="chip">"FluidDiv"</p>
                    <h3>"Reactive surfaces"</h3>
                    <p>
                        "Drive cards from signals and keep hover behavior declarative."
                    </p>
                </FluidDiv>

                <FluidButton
                    class="wrapper-button"
                    initial=FluidStyle::new().opacity(0.0).y(12.0)
                    animate=FluidStyle::new().opacity(1.0).y(0.0)
                    transition=Transition::snappy()
                    while_hover=FluidStyle::new().scale(1.05).y(-3.0)
                    while_tap=FluidStyle::new().scale(0.96)
                >
                    "Launch a FluidButton interaction"
                </FluidButton>

                <div class="chip-row">{chips}</div>
            </div>
        </section>
    }
}

fn wrapper_surface_style(progress: f64) -> FluidStyle {
    let background = if progress >= 0.5 {
        "linear-gradient(140deg, rgba(19, 78, 74, 0.9), rgba(37, 99, 235, 0.76))"
    } else {
        "linear-gradient(140deg, rgba(255, 255, 255, 0.06), rgba(255, 255, 255, 0.02))"
    };
    let border_alpha = lerp(0.08, 0.34, progress);

    FluidStyle::new()
        .opacity(lerp(0.92, 1.0, progress))
        .scale(lerp(1.0, 1.01, progress))
        .rotate(lerp(0.0, -0.8, progress))
        .with("background", background)
        .with(
            "border-color",
            format!("rgba(116, 241, 255, {border_alpha:.3})"),
        )
}
