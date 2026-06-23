use leptos::prelude::*;
use leptos_fluid_motion::{AnimationController, Easing, FluidStyle, Transition};
use leptos_fluid_scroll::prelude::*;

#[component]
pub fn ScrubCardSection() -> impl IntoView {
    let card_ref = NodeRef::<leptos::html::Div>::new();

    let controller = AnimationController::builder()
        .target(card_ref)
        .transition(Transition::new().duration_ms(120).easing(Easing::Linear))
        .initial(FluidStyle::new().opacity(0.0).scale(0.8).y(100.0))
        .install();

    let trigger = ScrollTrigger::builder()
        .target(card_ref)
        .start("top center")
        .end("bottom center")
        .scrub(Scrub::Bool(true))
        .bind_controller(controller, Box::new(|p| {
            FluidStyle::new()
                .opacity(p)
                .scale(0.8 + p * 0.2)
                .y(100.0 - p * 100.0)
        }))
        .install();

    let progress = trigger.progress();
    let is_active = trigger.is_active();

    view! {
        <section class="section section-scrub">
            <div class="panel">
                <p class="kicker">"Scrub-bound controller"</p>
                <h2>"A card driven by scroll progress"</h2>
                <p>
                    "ScrollTrigger::builder().scrub(true).bind_controller(...) maps \
                     the scroll progress signal to a FluidStyle via an AnimationController. \
                     The card fades, scales, and translates as it crosses the viewport."
                </p>
                <div class="indicator">
                    <span class="badge" class:active=move || is_active.get()>
                        {move || if is_active.get() { "active" } else { "idle" }}
                    </span>
                    <span class="badge">
                        {move || format!("progress {:.2}", progress.get())}
                    </span>
                </div>
                <div class="progress-track">
                    <div
                        class="progress-fill"
                        style:width=move || format!("{}%", (progress.get() * 100.0).round())
                    ></div>
                </div>
            </div>

            <div class="card card-scrub" node_ref=card_ref>
                <p class="chip">"scrub"</p>
                <h3>"Opacity, scale, and y translate"</h3>
                <p>"This element is bound to the scroll progress of the section."</p>
            </div>
        </section>
    }
}