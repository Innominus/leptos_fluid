use leptos::prelude::*;
use leptos_fluid::motion::{FluidDiv, FluidSpan, FluidStyle, FluidValue, Transition};

fn chip_delay(index: usize) -> Transition {
    Transition::new()
        .duration_ms(420)
        .bounce(0.2)
        .delay_ms(90 * index as u32)
}

#[component]
pub fn StaggeredChipsSection(pulse: RwSignal<bool>) -> impl IntoView {
    let pulse_style = move || {
        if pulse.get() {
            FluidStyle::new().opacity(0.9).scale(1.0)
        } else {
            FluidStyle::new().opacity(0.4).scale(0.86)
        }
    };

    view! {
        <section class="hero">
            <div class="panel">
                <h2>"Staggered chips"</h2>
                <p>
                    "Using FluidSpan with different delay values to get a simple stagger without extra runtime."
                    "Combine translate + opacity for a clean reveal."
                </p>
                <div class="list">
                    <FluidSpan
                        class="chip"
                        initial=FluidStyle::new().opacity(0.0).x(-16.0)
                        animate=FluidStyle::new().opacity(1.0).x(0.0)
                        transition=chip_delay(1)
                    >
                        "Initial → animate"
                    </FluidSpan>
                    <FluidSpan
                        class="chip"
                        initial=FluidStyle::new().opacity(0.0).x(-16.0)
                        animate=FluidStyle::new().opacity(1.0).x(0.0)
                        transition=chip_delay(2)
                    >
                        "Custom delay"
                    </FluidSpan>
                    <FluidSpan
                        class="chip"
                        initial=FluidStyle::new().opacity(0.0).x(-16.0)
                        animate=FluidStyle::new().opacity(1.0).x(0.0)
                        transition=chip_delay(3)
                    >
                        "FluidSpan"
                    </FluidSpan>
                    <FluidSpan
                        class="chip"
                        initial=FluidStyle::new().opacity(0.0).x(-16.0)
                        animate=FluidStyle::new().opacity(1.0).x(0.0)
                        transition=chip_delay(4)
                    >
                        "Lightweight"
                    </FluidSpan>
                </div>
            </div>

            <FluidDiv
                class="glass"
                initial=FluidStyle::new().opacity(0.0).y(26.0)
                animate=move || {
                    FluidStyle::new()
                        .opacity(1.0)
                        .y(0.0)
                        .scale(1.0)
                        .with_prop(
                            "background",
                            FluidValue::from(
                                "linear-gradient(130deg, rgba(20,24,44,0.9), rgba(255,255,255,0.04))",
                            ),
                        )
                }
                transition=Transition::spring_with(560, 0.35)
            >
                <h2>"Pulse orb"</h2>
                <p>"Tiny helper layer with FluidDiv + style composition."</p>
                <FluidDiv
                    class="orb one"
                    initial=FluidStyle::new().opacity(0.0).scale(0.8)
                    animate=pulse_style
                    transition=Transition::spring_with(780, 0.8)
                ></FluidDiv>
            </FluidDiv>
        </section>
    }
}
