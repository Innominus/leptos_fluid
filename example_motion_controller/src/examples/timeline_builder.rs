use leptos::prelude::*;
use leptos_fluid_motion::{AnimationController, FluidStep, FluidStyle, FluidTimeline, Transition};

#[component]
pub fn TimelineBuilderExample() -> impl IntoView {
    let glyph_ref = NodeRef::<leptos::html::Div>::new();
    let transition = Transition::spring_with(520, 0.32);

    let controller = AnimationController::builder()
        .target(glyph_ref)
        .transition(transition.clone())
        .initial(timeline_builder_rest())
        .install();
    let seeded = StoredValue::new(false);

    Effect::new(move || {
        if seeded.get_value() || glyph_ref.get().is_none() {
            return;
        }
        seeded.set_value(true);
        controller.set_immediate(timeline_builder_rest());
    });
    let timeline = FluidTimeline::builder(controller)
        .initial(timeline_builder_rest())
        .step(FluidStep::to(timeline_builder_lift()).inherit_wait_from(&transition))
        .step(FluidStep::to(timeline_builder_orbit()).inherit_wait_from(&transition))
        .step(FluidStep::to(timeline_builder_settle()).wait_ms(160))
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
    let reset = move |_| timeline.set_immediate(timeline_builder_rest());

    view! {
        <article class="demo-panel" data-testid="timeline-builder-panel">
            <div class="panel-header">
                <p class="panel-eyebrow">"Builder timeline"</p>
                <h2>"FluidTimeline::builder(controller)"</h2>
                <p>
                    "Typed steps with explicit pause and restart controls."
                </p>
            </div>

            <div class="button-row">
                <button data-testid="timeline-builder-restart" on:click=restart>
                    "Restart sequence"
                </button>
                <button class="ghost" data-testid="timeline-builder-pause" on:click=toggle_pause>
                    {move || if is_paused.get() { "Resume" } else { "Pause" }}
                </button>
                <button class="ghost" data-testid="timeline-builder-reset" on:click=reset>
                    "Reset"
                </button>
            </div>

            <div class="stage">
                <div node_ref=glyph_ref class="timeline-glyph builder-glyph" data-testid="timeline-builder-glyph">
                    <p class="chip">"timeline"</p>
                    <h3>"Builder-driven sequence"</h3>
                    <p data-testid="timeline-builder-status">
                        {move || builder_timeline_status(step_index.get(), is_paused.get())}
                    </p>
                </div>
            </div>
        </article>
    }
}

fn builder_timeline_status(step_index: usize, is_paused: bool) -> &'static str {
    if is_paused {
        return "Paused at the current keyframe.";
    }

    match step_index {
        0 => "Lift: leaving the dock.",
        1 => "Orbit: crossing the stage.",
        2 => "Settle: returning with a short hold.",
        _ => "Idle and waiting for restart.",
    }
}

fn timeline_builder_rest() -> FluidStyle {
    FluidStyle::new()
        .opacity(0.78)
        .x(0.0)
        .y(16.0)
        .scale(0.92)
        .rotate(0.0)
        .with("background", "#e2e8f0")
        .with("color", "#0f172a")
        .with("box-shadow", "0 14px 28px rgba(15,23,42,.12)")
}

fn timeline_builder_lift() -> FluidStyle {
    FluidStyle::new()
        .opacity(1.0)
        .x(0.0)
        .y(-10.0)
        .scale(1.0)
        .rotate(-1.2)
        .with("background", "#2563eb")
        .with("color", "#eff6ff")
        .with("box-shadow", "0 24px 42px rgba(37,99,235,.24)")
}

fn timeline_builder_orbit() -> FluidStyle {
    FluidStyle::new()
        .opacity(1.0)
        .x(34.0)
        .y(-2.0)
        .scale(1.05)
        .rotate(4.0)
        .with("background", "#7c3aed")
        .with("color", "#f5f3ff")
        .with("box-shadow", "0 28px 52px rgba(109,40,217,.26)")
}

fn timeline_builder_settle() -> FluidStyle {
    FluidStyle::new()
        .opacity(0.92)
        .x(-10.0)
        .y(8.0)
        .scale(0.98)
        .rotate(-2.0)
        .with("background", "#f59e0b")
        .with("color", "#451a03")
        .with("box-shadow", "0 18px 36px rgba(180,83,9,.22)")
}
