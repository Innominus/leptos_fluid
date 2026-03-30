use leptos::prelude::*;
use leptos_fluid_motion::{AnimationController, FluidStep, FluidStyle, FluidTimeline, Transition};

#[component]
pub fn SpringTimelineExample() -> impl IntoView {
    let glyph_ref = NodeRef::<leptos::html::Div>::new();
    let transition = Transition::spring_with(520, 0.38);

    let controller = AnimationController::builder()
        .target(glyph_ref)
        .transition(transition.clone())
        .initial(spring_timeline_rest())
        .install();
    let seeded = StoredValue::new(false);

    Effect::new(move || {
        if seeded.get_value() || glyph_ref.get().is_none() {
            return;
        }
        seeded.set_value(true);
        controller.set_immediate(spring_timeline_rest());
    });

    let timeline = FluidTimeline::builder(controller)
        .initial(spring_timeline_rest())
        .step(FluidStep::to(spring_timeline_lift()).inherit_wait_from(&transition))
        .step(FluidStep::to(spring_timeline_arc()).inherit_wait_from(&transition))
        .step(FluidStep::to(spring_timeline_settle()).wait_ms(160))
        .install();

    let step_index = timeline.step_index();
    let is_paused = timeline.is_paused();

    let restart = move |_| timeline.restart();
    let toggle_pause = move |_| {
        if is_paused.get_untracked() {
            timeline.resume();
        } else {
            timeline.pause();
        }
    };
    let reset = move |_| timeline.set_immediate(spring_timeline_rest());

    view! {
        <article class="demo-panel" data-testid="spring-timeline-panel">
            <div class="panel-header">
                <p class="panel-eyebrow">"Spring timeline"</p>
                <h2>"Timeline steps with spring transitions"</h2>
                <p>
                    "Timeline waits still use the configured duration, while each step uses a deliberately springy segment so you can see the bounce between waypoints."
                </p>
            </div>

            <div class="button-row">
                <button data-testid="spring-timeline-restart" on:click=restart>
                    "Restart sequence"
                </button>
                <button class="ghost" data-testid="spring-timeline-pause" on:click=toggle_pause>
                    {move || if is_paused.get() { "Resume" } else { "Pause" }}
                </button>
                <button class="ghost" data-testid="spring-timeline-reset" on:click=reset>
                    "Reset"
                </button>
            </div>

            <div class="stage">
                <div node_ref=glyph_ref class="timeline-glyph spring-timeline-glyph" data-testid="spring-timeline-glyph">
                    <p class="chip">"timeline"</p>
                    <h3>"Duration-driven spring steps"</h3>
                    <p data-testid="spring-timeline-status">
                        {move || spring_timeline_status(step_index.get(), is_paused.get())}
                    </p>
                </div>
            </div>
        </article>
    }
}

fn spring_timeline_status(step_index: usize, is_paused: bool) -> &'static str {
    if is_paused {
        return "Paused inside the active spring segment.";
    }

    match step_index {
        0 => "Lifting upward with a visible overshoot.",
        1 => "Arcing across the lane while preserving momentum.",
        2 => "Settling back into the dock with a short hold.",
        _ => "Idle and ready for another pass.",
    }
}

fn spring_timeline_rest() -> FluidStyle {
    FluidStyle::new()
        .opacity(0.72)
        .x(0.0)
        .y(18.0)
        .scale(0.88)
        .rotate(0.0)
}

fn spring_timeline_lift() -> FluidStyle {
    FluidStyle::new()
        .opacity(1.0)
        .x(-26.0)
        .y(-26.0)
        .scale(1.02)
        .rotate(-8.0)
}

fn spring_timeline_arc() -> FluidStyle {
    FluidStyle::new()
        .opacity(1.0)
        .x(60.0)
        .y(2.0)
        .scale(1.08)
        .rotate(10.0)
}

fn spring_timeline_settle() -> FluidStyle {
    FluidStyle::new()
        .opacity(0.88)
        .x(-14.0)
        .y(14.0)
        .scale(0.94)
        .rotate(-5.0)
}
