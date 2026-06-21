use leptos::prelude::*;

use leptos_fluid_motion::{AnimationController, Easing, FluidStyle, Transition};
use leptos_fluid_scroll::prelude::*;

const LINES: &[&str] = &["We build", "interfaces", "that move", "with you."];

#[component]
pub fn TextMaskRevealSection() -> impl IntoView {
    let section_ref = NodeRef::<leptos::html::Section>::new();

    let line_refs: Vec<NodeRef<leptos::html::Span>> = (0..LINES.len())
        .map(|_| NodeRef::<leptos::html::Span>::new())
        .collect();

    let line_controllers: Vec<AnimationController> = line_refs
        .iter()
        .enumerate()
        .map(|(i, line_ref)| {
            AnimationController::builder()
                .target(*line_ref)
                .transition(
                    Transition::new()
                        .duration_ms(600)
                        .delay_ms(i as u32 * 80)
                        .easing(Easing::EaseOut),
                )
                .initial(FluidStyle::new().y(120.0))
                .install()
        })
        .collect();

    let _trigger = ScrollTrigger::builder()
        .target(section_ref)
        .start("top 80%")
        .end("top 20%")
        // One-shot tween-on-enter: each controller animates from y(120) to y(0) with staggered delay_ms.
        .once(true)
        .on_enter(move |_| {
            for controller in &line_controllers {
                controller.animate(FluidStyle::new().y(0.0));
            }
        })
        .install();

    view! {
        <section class="section section-text" id="text" node_ref=section_ref>
            <div class="section-inner">
                <p class="kicker">"03 — Text Mask Reveal"</p>
                <h2 class="text-mask-heading">
                    {
                        LINES.iter().enumerate().map(|(i, line)| {
                            let line_ref = line_refs[i];
                            view! {
                                <span class="line-mask">
                                    <span class="line" node_ref=line_ref>{*line}</span>
                                </span>
                            }
                        }).collect::<Vec<_>>()
                    }
                </h2>
                <p class="lead">
                    "Each line rises from behind a clip mask, staggered by 80ms. \
                     A single ScrollTrigger with once: true drives the cascade by \
                     animating per-line controllers on enter."
                </p>
            </div>
        </section>
    }
}