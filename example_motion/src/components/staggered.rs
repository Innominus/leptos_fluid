use leptos::prelude::*;
use leptos_fluid::motion::{MotionDiv, MotionSpan, MotionStyle, Transition};

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
            MotionStyle::new().opacity(0.9).scale(1.0)
        } else {
            MotionStyle::new().opacity(0.4).scale(0.86)
        }
    };

    view! {
        <section class="hero">
            <div class="panel">
                <h2>"Staggered chips"</h2>
                <p>
                    "Using MotionSpan with different delay values to get a simple stagger without extra runtime."
                    "Combine translate + opacity for a clean reveal."
                </p>
                <div class="list">
                    <MotionSpan
                        class="chip"
                        initial=MotionStyle::new().opacity(0.0).x(-16.0)
                        animate=MotionStyle::new().opacity(1.0).x(0.0)
                        transition=chip_delay(1)
                    >
                        "Initial → animate"
                    </MotionSpan>
                    <MotionSpan
                        class="chip"
                        initial=MotionStyle::new().opacity(0.0).x(-16.0)
                        animate=MotionStyle::new().opacity(1.0).x(0.0)
                        transition=chip_delay(2)
                    >
                        "Custom delay"
                    </MotionSpan>
                    <MotionSpan
                        class="chip"
                        initial=MotionStyle::new().opacity(0.0).x(-16.0)
                        animate=MotionStyle::new().opacity(1.0).x(0.0)
                        transition=chip_delay(3)
                    >
                        "MotionSpan"
                    </MotionSpan>
                    <MotionSpan
                        class="chip"
                        initial=MotionStyle::new().opacity(0.0).x(-16.0)
                        animate=MotionStyle::new().opacity(1.0).x(0.0)
                        transition=chip_delay(4)
                    >
                        "Lightweight"
                    </MotionSpan>
                </div>
            </div>

            <MotionDiv
                class="glass"
                initial=MotionStyle::new().opacity(0.0).y(26.0)
                animate=move || {
                    MotionStyle::new()
                        .opacity(1.0)
                        .y(0.0)
                        .scale(1.0)
                        .with(
                            "background",
                            "linear-gradient(130deg, rgba(20,24,44,0.9), rgba(255,255,255,0.04))",
                        )
                }
                transition=Transition::spring_with(560, 0.35)
            >
                <h2>"Pulse orb"</h2>
                <p>"Tiny helper layer with MotionDiv + style composition."</p>
                <MotionDiv
                    class="orb one"
                    initial=MotionStyle::new().opacity(0.0).scale(0.8)
                    animate=pulse_style
                    transition=Transition::spring_with(780, 0.8)
                ></MotionDiv>
            </MotionDiv>
        </section>
    }
}
