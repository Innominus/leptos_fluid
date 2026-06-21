use leptos::prelude::*;

use leptos_fluid_motion::{AnimationController, Easing, FluidStyle, Transition};
use leptos_fluid_scroll::prelude::*;

#[component]
pub fn ImageRevealSection() -> impl IntoView {
    let image_ref = NodeRef::<leptos::html::Div>::new();

    let controller = AnimationController::builder()
        .target(image_ref)
        .transition(Transition::new().duration_ms(800).easing(Easing::EaseOut))
        .initial(FluidStyle::new().with("clip-path", "inset(50% 0 50% 0)").scale(1.2))
        .install();

    let controller_for_cb = controller;
    let trigger = ScrollTrigger::builder()
        .target(image_ref)
        .start("top 80%")
        .end("top 20%")
        .scrub(Scrub::Bool(false))
        .on_enter(move |_| {
            controller_for_cb.animate(
                FluidStyle::new().with("clip-path", "inset(0% 0 0% 0)").scale(1.0),
            );
        })
        .install();

    let progress = trigger.progress();
    let is_active = trigger.is_active();

    view! {
        <section class="section section-image" id="image">
            <div class="section-inner">
                <p class="kicker">"06 — Image Reveal"</p>
                <h2>"Cascade Range, 2024"</h2>
                <p class="lead">
                    "A clip-path inset opens from a thin horizontal slit to full \
                     visibility as the frame scrolls into view. A slight scale \
                     settle complements the reveal."
                </p>

                <div class="image-frame" node_ref=image_ref>
                    <div class="image-frame-inner"></div>
                    <span class="image-label" style:opacity=move || if is_active.get() { "1.0" } else { "0.0" }>
                        "Cascade Range, 2024"
                    </span>
                </div>

                <div class="indicator">
                    <span class="badge" class:active=move || is_active.get()>
                        {move || format!("progress {:.2}", progress.get())}
                    </span>
                    <span class="badge" class:active=move || is_active.get()>
                        {move || if is_active.get() { "active" } else { "idle" }}
                    </span>
                </div>
            </div>
        </section>
    }
}