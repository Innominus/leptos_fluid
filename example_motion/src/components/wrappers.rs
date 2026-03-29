use leptos::prelude::*;
use leptos_fluid_motion::{FluidButton, FluidDiv, FluidSpan, FluidStyle, Transition};

const WRAPPER_TAGS: [&str; 4] = ["FluidDiv", "FluidButton", "FluidSpan", "while_hover"];

#[component]
pub fn WrapperGallerySection() -> impl IntoView {
    let spotlight = RwSignal::new(false);

    let chips = WRAPPER_TAGS
        .into_iter()
        .enumerate()
        .map(|(index, label)| {
            view! {
                <FluidSpan
                    class="chip"
                    initial=FluidStyle::new().opacity(0.0).x(-14.0)
                    animate=FluidStyle::new().opacity(1.0).x(0.0)
                    transition=Transition::new().duration_ms(360).delay_ms(70 * index as u32)
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
                    initial=FluidStyle::new().opacity(0.0).y(18.0)
                    animate=move || wrapper_surface_style(spotlight.get())
                    transition=Transition::spring_with(520, 0.28)
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
                    transition=Transition::spring_with(420, 0.2)
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

fn wrapper_surface_style(spotlight: bool) -> FluidStyle {
    if spotlight {
        FluidStyle::new()
            .opacity(1.0)
            .scale(1.01)
            .rotate(-0.8)
            .with(
                "background",
                "linear-gradient(140deg, rgba(19, 78, 74, 0.9), rgba(37, 99, 235, 0.76))",
            )
            .with("border-color", "rgba(116, 241, 255, 0.34)")
    } else {
        FluidStyle::new()
            .opacity(0.92)
            .scale(1.0)
            .rotate(0.0)
            .with(
                "background",
                "linear-gradient(140deg, rgba(255, 255, 255, 0.06), rgba(255, 255, 255, 0.02))",
            )
            .with("border-color", "rgba(255, 255, 255, 0.08)")
    }
}
