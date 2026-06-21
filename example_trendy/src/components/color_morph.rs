use leptos::prelude::*;

use leptos_fluid_motion::{AnimationController, Easing, FluidStyle, Transition};
use leptos_fluid_scroll::prelude::*;

#[component]
pub fn ColorMorphSection() -> impl IntoView {
    let block_ref = NodeRef::<leptos::html::Div>::new();

    let controller = AnimationController::builder()
        .target(block_ref)
        .transition(Transition::new().duration_ms(200).easing(Easing::EaseOut))
        .initial(FluidStyle::new().with("background-color", "rgba(255, 45, 117, 1.0)"))
        .install();

    let trigger = ScrollTrigger::builder()
        .target(block_ref)
        .start("top 80%")
        .end("bottom 20%")
        .scrub(Scrub::Number(0.15))
        .bind_controller(controller, |p| {
            let r = 255.0 + (0.0 - 255.0) * p;
            let g = 45.0 + (71.0 - 45.0) * p;
            let b = 117.0 + (255.0 - 117.0) * p;
            FluidStyle::new().with(
                "background-color",
                format!("rgba({:.0}, {:.0}, {:.0}, 1.0)", r, g, b),
            )
        })
        .install();

    let progress = trigger.progress();
    let progress_pct = Memo::new(move |_| (progress.get() * 100.0).round() as i32);

    view! {
        <section class="section section-color" id="color">
            <div class="section-inner">
                <p class="kicker">"09 — Section Color Morph"</p>
                <h2>"Ambient transitions"</h2>
                <p class="lead">
                    "A full-width block morphs from electric magenta to electric \
                     blue as you scroll through the section. The controller tweens \
                     the background-color string in lockstep with scrubbed progress."
                </p>

                <div class="color-block" node_ref=block_ref>
                    <div class="color-block-content">
                        <h3 class="color-block-title">"Ambient transitions"</h3>
                        <p class="color-block-desc">
                            "Color is bound to scroll progress and smoothed by the \
                             controller transition."
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