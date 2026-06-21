use leptos::prelude::*;

use leptos_fluid_motion::{AnimationController, Easing, FluidStyle, Transition};
use leptos_fluid_scroll::prelude::*;

#[component]
pub fn PerspectiveTiltSection() -> impl IntoView {
    let card_ref = NodeRef::<leptos::html::Div>::new();

    let controller = AnimationController::builder()
        .target(card_ref)
        .transition(Transition::new().duration_ms(200).easing(Easing::EaseOut))
        .initial(
            FluidStyle::new().with(
                "transform",
                "perspective(1000px) rotateY(-45deg) rotateX(15deg) scale(0.9)",
            ),
        )
        .install();

    let trigger = ScrollTrigger::builder()
        .target(card_ref)
        .start("top 90%")
        .end("bottom 10%")
        .scrub(Scrub::Number(0.15))
        .bind_controller(controller, |p| {
            let ry = -45.0 + p * 90.0;
            let rx = 15.0 - p * 30.0;
            let s = 0.9 + p * 0.2;
            FluidStyle::new().with(
                "transform",
                format!(
                    "perspective(1000px) rotateY({:.2}deg) rotateX({:.2}deg) scale({:.3})",
                    ry, rx, s
                ),
            )
        })
        .install();

    let progress = trigger.progress();
    let progress_pct = Memo::new(move |_| (progress.get() * 100.0).round() as i32);

    view! {
        <section class="section section-tilt" id="tilt">
            <div class="section-inner">
                <p class="kicker">"07 — 3D Perspective Tilt"</p>
                <h2>"Dimensional"</h2>
                <p class="lead">
                    "A card rotates through 3D space as you scroll. rotateY sweeps \
                     -45deg to +45deg, rotateX drifts from +15 to -15, and scale \
                     eases from 0.9 to 1.1 — all driven by scrubbed progress bound \
                     to a controller via a raw transform string."
                </p>

                <div class="tilt-container">
                    <div class="tilt-card" node_ref=card_ref>
                        <span class="tilt-numeral">"07"</span>
                        <h3 class="tilt-title">"Dimensional"</h3>
                        <p class="tilt-desc">
                            "Depth, parallax, and rotation composed from a single \
                             scroll-bound transform."
                        </p>
                    </div>
                </div>

                <div class="indicator">
                    <span class="badge">
                        {move || format!("{}%", progress_pct.get())}
                    </span>
                </div>
            </div>
        </section>
    }
}