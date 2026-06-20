use leptos::prelude::*;
use leptos_fluid_motion::{Easing, FluidStyle, FluidTimeline, Transition};
use leptos_fluid_scroll::prelude::*;

const STEP_COUNT: usize = 4;

#[component]
pub fn TimelineScrubSection() -> impl IntoView {
    let card_ref = NodeRef::<leptos::html::Div>::new();

    let controller = leptos_fluid_motion::AnimationController::builder()
        .target(card_ref)
        .transition(Transition::new().duration_ms(120).easing(Easing::Linear))
        .initial(step_style(0))
        .install();

    let seeded = StoredValue::new(false);
    Effect::new(move || {
        if seeded.get_value() || card_ref.get().is_none() {
            return;
        }
        seeded.set_value(true);
        controller.set_immediate(step_style(0));
    });

    let timeline = FluidTimeline::new(step_style(0));
    timeline.bind(controller);

    let trigger = ScrollTrigger::builder()
        .target(card_ref)
        .start("top center")
        .end("bottom center")
        .scrub(Scrub::Bool(true))
        .bind_timeline_scrub(timeline, STEP_COUNT, |idx, _p| step_style(idx))
        .install();

    let progress = trigger.progress();
    let current_step = RwSignal::new(0usize);

    Effect::new(move || {
        let p = progress.get();
        let target = if STEP_COUNT == 0 {
            0
        } else {
            ((p * STEP_COUNT as f64).floor() as usize).min(STEP_COUNT - 1)
        };
        current_step.set(target);
    });

    view! {
        <section class="section section-scrub">
            <div class="panel">
                <p class="kicker">"Discrete-step scrubbing"</p>
                <h2>"Timeline jumps between steps as you scroll"</h2>
                <p>
                    "bind_timeline_scrub(timeline, step_count, |idx, p| style_fn(idx, p)) \
                     maps scroll progress to a step index and calls set_immediate on the \
                     timeline for each step boundary crossed."
                </p>
                <div class="indicator">
                    <span class="badge">
                        {move || format!("step {}", current_step.get())}
                    </span>
                    <span class="badge">
                        {move || format!("progress {:.2}", progress.get())}
                    </span>
                </div>
                <div class="step-row">
                    {(0..STEP_COUNT).map(|i| {
                        view! {
                            <span class="chip" class:active=move || current_step.get() == i>
                                {format!("Step {}", i + 1)}
                            </span>
                        }
                    }).collect_view()}
                </div>
            </div>

            <div class="card card-scrub-step" node_ref=card_ref>
                <p class="chip">"step scrub"</p>
                <h3>"Four discrete style states"</h3>
                <p>"The element jumps between four style snapshots as you scroll."</p>
            </div>
        </section>
    }
}

fn step_style(index: usize) -> FluidStyle {
    match index {
        0 => FluidStyle::new()
            .opacity(0.6)
            .x(-40.0)
            .y(40.0)
            .scale(0.9)
            .rotate(-2.0),
        1 => FluidStyle::new()
            .opacity(0.85)
            .x(0.0)
            .y(0.0)
            .scale(1.0)
            .rotate(0.0),
        2 => FluidStyle::new()
            .opacity(1.0)
            .x(40.0)
            .y(-10.0)
            .scale(1.05)
            .rotate(2.5),
        _ => FluidStyle::new()
            .opacity(0.92)
            .x(0.0)
            .y(-20.0)
            .scale(1.02)
            .rotate(-1.0),
    }
}