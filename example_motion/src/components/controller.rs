use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_fluid::motion::{AnimationController, FluidStyle, Transition};

#[component]
pub fn ControllerSection() -> impl IntoView {
    let expanded = RwSignal::new(false);
    let node_ref = NodeRef::<leptos::html::Div>::new();
    let controller = AnimationController::with_transition(Transition::spring_with(520, 0.35));

    controller.attach_resolver({
        let node_ref = node_ref.clone();
        move || node_ref.get_untracked().map(|node| node.unchecked_into())
    });

    Effect::new(move || {
        let target = if expanded.get() {
            FluidStyle::new().opacity(1.0).x(0.0).y(0.0).scale(1.0)
        } else {
            FluidStyle::new().opacity(0.5).x(-24.0).y(10.0).scale(0.94)
        };
        controller.animate(target);
    });

    view! {
        <section class="hero">
            <div class="panel">
                <h2>"AnimationController"</h2>
                <p>
                    "Drive a plain node_ref with declarative state. No FluidElement wrapper required."
                </p>
                <div class="button-row">
                    <button class="alt" on:click=move |_| expanded.update(|value| *value = !*value)>
                        {move || {
                            if expanded.get() { "Send back" } else { "Animate forward" }
                        }}
                    </button>
                </div>
            </div>

            <div class="presence-card timeline-card">
                <div class="presence-stage timeline-stage">
                    <div class="timeline-orb"></div>
                    <div node_ref=node_ref class="timeline-widget">
                        <span class="chip">"Controller"</span>
                        <h3>"Element-agnostic"</h3>
                        <p>
                            "Attach a resolver and call animate()."
                        </p>
                    </div>
                </div>
            </div>
        </section>
    }
}
