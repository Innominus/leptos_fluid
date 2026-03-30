use leptos::prelude::*;
use leptos_fluid_motion::{
    AnimationController, Easing, FluidStep, FluidStyle, FluidTimeline, Transition,
};

#[component]
pub fn TimelineStudioSection() -> impl IntoView {
    let glyph_ref = NodeRef::<leptos::html::Div>::new();
    let transition = Transition::new().duration_ms(240).easing(Easing::EaseInOut);
    let controller = AnimationController::builder()
        .target(glyph_ref)
        .transition(transition.clone())
        .initial(timeline_rest_style())
        .install();
    let seeded = StoredValue::new(false);

    Effect::new(move || {
        if seeded.get_value() || glyph_ref.get().is_none() {
            return;
        }
        seeded.set_value(true);
        controller.set_immediate(timeline_rest_style());
    });

    let timeline = FluidTimeline::builder(controller)
        .initial(timeline_rest_style())
        .autoplay(false)
        .auto_loop(true)
        .step(FluidStep::to(timeline_lift_style()).inherit_wait_from(&transition))
        .step(FluidStep::to(timeline_glide_style()).inherit_wait_from(&transition))
        .step(FluidStep::to(timeline_anchor_style()).wait_ms(180))
        .install();

    let step_index = timeline.step_index();
    let is_paused = timeline.is_paused();
    let auto_loop = timeline.auto_loop();

    let restart = move |_| timeline.restart();
    let toggle_pause = move |_| {
        if is_paused.get_untracked() {
            timeline.resume();
        } else {
            timeline.pause();
        }
    };
    let reset = move |_| timeline.set_immediate(timeline_rest_style());
    let toggle_loop = move |_| timeline.toggle_auto_loop();

    view! {
        <section class="section-grid">
            <div class="panel">
                <p class="section-kicker">"Timeline studio"</p>
                <h2>"Builder timeline on a plain node"</h2>
                <p>
                    "A controller and timeline stay explicit here: typed steps, plain node refs, and direct playback controls."
                </p>

                <div class="button-row">
                    <button on:click=restart>
                        "Restart"
                    </button>
                    <button class="alt" on:click=toggle_pause>
                        {move || if is_paused.get() { "Resume" } else { "Pause" }}
                    </button>
                    <button class="alt" on:click=reset>
                        "Reset"
                    </button>
                    <button class="alt" on:click=toggle_loop>
                        {move || if auto_loop.get() { "Disable loop" } else { "Enable loop" }}
                    </button>
                </div>

                <div class="mode-row">
                    <span class="chip chip-button" class:active=move || step_index.get() == 0>
                        "Lift"
                    </span>
                    <span class="chip chip-button" class:active=move || step_index.get() == 1>
                        "Glide"
                    </span>
                    <span class="chip chip-button" class:active=move || step_index.get() == 2>
                        "Anchor"
                    </span>
                </div>
            </div>

            <div class="timeline-stage-shell">
                <div class="timeline-stage">
                    <div class="timeline-glow"></div>
                    <div node_ref=glyph_ref class="timeline-node">
                        <p class="chip">"timeline"</p>
                        <h3>"Sequenced plain element"</h3>
                        <p>{move || timeline_status(step_index.get(), is_paused.get())}</p>
                    </div>
                </div>
            </div>
        </section>
    }
}

fn timeline_status(step_index: usize, is_paused: bool) -> &'static str {
    if is_paused {
        return "Paused at the current stage.";
    }

    match step_index {
        0 => "Lift: easing out of the dock.",
        1 => "Glide: crossing the stage.",
        2 => "Anchor: holding the finished state.",
        _ => "Idle and ready to restart.",
    }
}

fn timeline_rest_style() -> FluidStyle {
    FluidStyle::new()
        .opacity(0.78)
        .x(0.0)
        .y(18.0)
        .scale(0.92)
        .rotate(0.0)
        .with(
            "background",
            "linear-gradient(145deg, rgba(255,255,255,0.08), rgba(255,255,255,0.02))",
        )
        .with("border-color", "rgba(255,255,255,0.1)")
}

fn timeline_lift_style() -> FluidStyle {
    FluidStyle::new()
        .opacity(1.0)
        .x(0.0)
        .y(-12.0)
        .scale(1.0)
        .rotate(-1.6)
        .with(
            "background",
            "linear-gradient(145deg, rgba(15,118,110,0.96), rgba(37,99,235,0.84))",
        )
        .with("border-color", "rgba(116, 241, 255, 0.3)")
}

fn timeline_glide_style() -> FluidStyle {
    FluidStyle::new()
        .opacity(1.0)
        .x(30.0)
        .y(-2.0)
        .scale(1.04)
        .rotate(3.0)
        .with(
            "background",
            "linear-gradient(145deg, rgba(217,119,6,0.96), rgba(190,24,93,0.84))",
        )
        .with("border-color", "rgba(255, 204, 112, 0.34)")
}

fn timeline_anchor_style() -> FluidStyle {
    FluidStyle::new()
        .opacity(0.92)
        .x(-8.0)
        .y(10.0)
        .scale(0.98)
        .rotate(-1.2)
        .with(
            "background",
            "linear-gradient(145deg, rgba(124,58,237,0.88), rgba(37,99,235,0.78))",
        )
        .with("border-color", "rgba(196,181,253,0.28)")
}
