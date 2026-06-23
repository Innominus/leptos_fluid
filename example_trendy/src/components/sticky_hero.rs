use leptos::prelude::*;

use leptos_fluid_motion::{AnimationController, FluidStyle};
use leptos_fluid_scroll::prelude::*;

#[component]
pub fn StickyHeroSection() -> impl IntoView {
    let section_ref = NodeRef::<leptos::html::Section>::new();
    let title_ref = NodeRef::<leptos::html::H1>::new();
    let bg_one_ref = NodeRef::<leptos::html::Div>::new();
    let bg_two_ref = NodeRef::<leptos::html::Div>::new();

    let title_controller = AnimationController::builder()
        .target(title_ref)
        .initial(FluidStyle::new().opacity(1.0).scale(1.0).y(0.0))
        .install();

    let bg_one_controller = AnimationController::builder()
        .target(bg_one_ref)
        .initial(FluidStyle::new().y(0.0))
        .install();

    let bg_two_controller = AnimationController::builder()
        .target(bg_two_ref)
        .initial(FluidStyle::new().y(0.0))
        .install();

    let trigger = ScrollTrigger::builder()
        .target(section_ref)
        .start("top top")
        .end("bottom top")
        .scrub(Scrub::Number(0.15))
        .bind_controller(title_controller, Box::new(|p| {
            FluidStyle::new()
                .opacity(1.0 - p * 0.7)
                .scale(1.0 - p * 0.5)
                .y(p * 50.0)
        }))
        .install();

    trigger.bind_controller(bg_one_controller, Box::new(|p| FluidStyle::new().y(p * -100.0)));
    trigger.bind_controller(bg_two_controller, Box::new(|p| FluidStyle::new().y(p * -200.0)));

    view! {
        <section class="section section-hero" id="hero" node_ref=section_ref>
            <div class="hero-spacer">
                <div class="hero-sticky">
                    <div class="hero-layer" node_ref=bg_one_ref>
                        <div class="hero-bg-blob hero-bg-blob-1"></div>
                    </div>
                    <div class="hero-layer" node_ref=bg_two_ref>
                        <div class="hero-bg-blob hero-bg-blob-2"></div>
                    </div>
                    <div class="hero-content">
                        <p class="kicker">"01 — Sticky Hero Parallax"</p>
                        <h1 class="hero-title" node_ref=title_ref>
                            "We design"
                            <br />
                            "in motion."
                        </h1>
                        <p class="lead">
                            "A pinned hero composition driven by scroll progress. \
                             Two parallax layers drift at different rates while the \
                             title scales down and fades out as you scroll through."
                        </p>
                    </div>
                </div>
            </div>
        </section>
    }
}