use leptos::prelude::*;

use leptos_fluid_motion::{AnimationController, Easing, FluidStyle, Transition};
use leptos_fluid_scroll::prelude::*;

const FEATURES: &[(&str, &str, &str)] = &[
    ("01", "Scroll Scrub", "Bind controller output 1:1 to scroll progress."),
    ("02", "Timeline Binding", "Drive step sequences via toggleActions."),
    ("03", "Velocity Tracking", "Reactive velocity signal from a 32-slot ring buffer."),
    ("04", "One-shot Reveal", "Fire-once on enter for entrance animations."),
    ("05", "Spring Physics", "Live rAF springs with tunable bounce and duration."),
    ("06", "Auto Resize", "Measure-once size binding that reflows on content change."),
];

#[component]
fn StaggerCard(
    index: usize,
    number: &'static str,
    title: &'static str,
    desc: &'static str,
) -> impl IntoView {
    let card_ref = NodeRef::<leptos::html::Article>::new();

    let controller = AnimationController::builder()
        .target(card_ref)
        .transition(
            Transition::new()
                .duration_ms(500)
                .delay_ms(index as u32 * 100)
                .easing(Easing::EaseOut),
        )
        .initial(FluidStyle::new().opacity(0.0).y(60.0).scale(0.95))
        .install();

    let controller_for_cb = controller;
    let _trigger = ScrollTrigger::builder()
        .target(card_ref)
        .start("top 85%")
        // One-shot tween-on-enter: controller animates from hidden to revealed with per-card delay.
        .once(true)
        .on_enter(move |_| {
            controller_for_cb.animate(FluidStyle::new().opacity(1.0).y(0.0).scale(1.0));
        })
        .install();

    view! {
        <article class="stagger-card" node_ref=card_ref>
            <span class="card-numeral">{number}</span>
            <h3 class="stagger-card-title">{title}</h3>
            <p class="stagger-card-desc">{desc}</p>
        </article>
    }
}

#[component]
pub fn StaggerGridSection() -> impl IntoView {
    view! {
        <section class="section section-grid" id="grid">
            <div class="section-inner">
                <p class="kicker">"04 — Staggered Card Grid"</p>
                <h2>"Features in formation"</h2>
                <p class="lead">
                    "Six cards fade, lift, and scale into place with a 100ms stagger. \
                     Each card owns its own ScrollTrigger so the cascade registers \
                     independently as it crosses the viewport."
                </p>
                <div class="card-grid stagger-grid">
                    {
                        FEATURES.iter().enumerate().map(|(i, feat)| {
                            view! {
                                <StaggerCard index=i number={feat.0} title={feat.1} desc={feat.2} />
                            }
                        }).collect::<Vec<_>>()
                    }
                </div>
            </div>
        </section>
    }
}