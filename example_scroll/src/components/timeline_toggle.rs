use leptos::prelude::*;
use leptos_fluid_motion::{Easing, FluidStep, FluidStyle, FluidTimeline, Transition};
use leptos_fluid_scroll::prelude::*;

#[component]
pub fn TimelineToggleSection() -> impl IntoView {
    let card_ref = NodeRef::<leptos::html::Div>::new();

    let transition = Transition::new().duration_ms(300).easing(Easing::EaseInOut);
    let controller = leptos_fluid_motion::AnimationController::builder()
        .target(card_ref)
        .transition(transition.clone())
        .initial(toggle_rest_style())
        .install();

    let seeded = StoredValue::new(false);
    Effect::new(move || {
        if seeded.get_value() || card_ref.get().is_none() {
            return;
        }
        seeded.set_value(true);
        controller.set_immediate(toggle_rest_style());
    });

    let timeline = FluidTimeline::builder(controller)
        .initial(toggle_rest_style())
        .autoplay(false)
        .auto_loop(true)
        .step(FluidStep::to(toggle_lift_style()).inherit_wait_from(&transition))
        .step(FluidStep::to(toggle_glide_style()).inherit_wait_from(&transition))
        .step(FluidStep::to(toggle_anchor_style()).inherit_wait_from(&transition))
        .install();

    let _ = ScrollTrigger::builder()
        .target(card_ref)
        .start("top center")
        .end("bottom top")
        .bind_timeline(timeline, "play pause resume none")
        .install();

    let step_index = timeline.step_index();
    let is_paused = timeline.is_paused();
    let is_running = timeline.is_running();

    view! {
        <section class="section">
            <div class="panel">
                <p class="kicker">"Toggle-bound timeline"</p>
                <h2>"Timeline plays when scrolled in, pauses when scrolled out"</h2>
                <p>
                    "bind_timeline(timeline, \"play pause resume none\") maps the four \
                     toggleActions phases to FluidTimeline methods."
                </p>
                <div class="indicator">
                    <span class="badge" class:active=move || is_running.get()>
                        {move || if is_running.get() { "running" } else { "stopped" }}
                    </span>
                    <span class="badge" class:active=move || is_paused.get()>
                        {move || if is_paused.get() { "paused" } else { "playing" }}
                    </span>
                    <span class="badge">
                        {move || format!("step {}", step_index.get())}
                    </span>
                </div>
                <div class="step-row">
                    <span class="chip" class:active=move || step_index.get() == 0>"Rest"</span>
                    <span class="chip" class:active=move || step_index.get() == 1>"Lift"</span>
                    <span class="chip" class:active=move || step_index.get() == 2>"Glide"</span>
                    <span class="chip" class:active=move || step_index.get() == 3>"Anchor"</span>
                </div>
            </div>

            <div class="card card-toggle" node_ref=card_ref>
                <p class="chip">"toggle"</p>
                <h3>"Three-step timeline"</h3>
                <p>"Plays on enter, pauses on leave, resumes on enter-back."</p>
            </div>
        </section>
    }
}

fn toggle_rest_style() -> FluidStyle {
    FluidStyle::new()
        .opacity(0.78)
        .x(0.0)
        .y(20.0)
        .scale(0.92)
        .rotate(0.0)
        .with("border-color", "rgba(255,255,255,0.1)")
}

fn toggle_lift_style() -> FluidStyle {
    FluidStyle::new()
        .opacity(1.0)
        .x(0.0)
        .y(-10.0)
        .scale(1.02)
        .rotate(-1.4)
        .with("border-color", "rgba(116, 241, 255, 0.3)")
}

fn toggle_glide_style() -> FluidStyle {
    FluidStyle::new()
        .opacity(1.0)
        .x(30.0)
        .y(0.0)
        .scale(1.04)
        .rotate(2.5)
        .with("border-color", "rgba(255, 204, 112, 0.34)")
}

fn toggle_anchor_style() -> FluidStyle {
    FluidStyle::new()
        .opacity(0.92)
        .x(-8.0)
        .y(8.0)
        .scale(0.98)
        .rotate(-1.0)
        .with("border-color", "rgba(196,181,253,0.28)")
}