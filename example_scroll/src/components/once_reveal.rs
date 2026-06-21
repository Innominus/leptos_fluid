use leptos::prelude::*;
use leptos_fluid_motion::{AnimationController, Easing, FluidStyle, Transition};
use leptos_fluid_scroll::prelude::*;

#[component]
pub fn OnceRevealSection() -> impl IntoView {
    let card_ref = NodeRef::<leptos::html::Div>::new();

    let controller = AnimationController::builder()
        .target(card_ref)
        .transition(Transition::new().duration_ms(500).easing(Easing::EaseOut))
        .initial(FluidStyle::new().opacity(0.0).y(40.0))
        .install();

    let revealed = RwSignal::new(false);

    let trigger = ScrollTrigger::builder()
        .target(card_ref)
        .start("top 85%")
        .once(true)
        .on_enter(move |_| {
            if revealed.get() {
                return;
            }
            revealed.set(true);
            controller.animate(FluidStyle::new().opacity(1.0).y(0.0));
        })
        .install();

    let progress = trigger.progress();

    view! {
        <section class="section">
            <div class="panel">
                <p class="kicker">"One-shot reveal"</p>
                <h2>"Fire-once on enter"</h2>
                <p>
                    "ScrollTrigger with once: true fires on_enter a single time \
                     when the element enters the viewport; the callback calls \
                     controller.animate() to tween from hidden (opacity 0, \
                     translateY 40px) to visible (opacity 1, translateY 0) over \
                     500ms."
                </p>
                <div class="indicator">
                    <span class="badge" class:active=move || revealed.get()>
                        {move || if revealed.get() { "revealed" } else { "hidden" }}
                    </span>
                    <span class="badge">
                        {move || format!("progress {:.2}", progress.get())}
                    </span>
                </div>
            </div>

            <div class="card card-once" node_ref=card_ref>
                <p class="chip">"once"</p>
                <h3>"Reveal on first enter"</h3>
                <p>"Starts hidden; animates in once when scrolled into view."</p>
            </div>
        </section>
    }
}
