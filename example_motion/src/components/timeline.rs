use leptos::prelude::*;
use leptos_fluid::motion::{
    FluidDiv, FluidNodeRef, FluidStep, FluidStyle, FluidTimeline, Transition,
};

#[component]
pub fn TimelineSection() -> impl IntoView {
    let base_style = FluidStyle::new().opacity(0.45).y(18.0).scale(0.94);
    let transition = Transition::spring_with(520, 0.35);
    let node_ref = FluidNodeRef::new();
    let timeline = FluidTimeline::new(base_style.clone());

    timeline.attach_node_ref(node_ref);
    timeline.set_steps([
        FluidStep::new(FluidStyle::new().opacity(1.0).y(0.0).scale(1.0)).wait_for(&transition),
        FluidStep::new(FluidStyle::new().opacity(1.0).y(-14.0).scale(1.04)).wait_for(&transition),
        FluidStep::new(FluidStyle::new().opacity(0.95).x(22.0).rotate(3.0)).wait_for(&transition),
        FluidStep::new(FluidStyle::new().opacity(0.5).y(12.0).scale(0.92)).wait_for(&transition),
    ]);
    timeline.set_auto_loop(true);

    let animate = timeline.signal();
    let step_index = timeline.step_index();
    let paused = timeline.is_paused();
    let auto_loop = timeline.auto_loop();

    let play = move |_| timeline.play();
    let pause = move |_| timeline.pause();
    let resume = move |_| timeline.resume();
    let reset = {
        let base_style = base_style.clone();
        move |_| timeline.set_immediate(base_style.clone())
    };
    let toggle_loop = move |_| timeline.toggle_auto_loop();

    let started = StoredValue::new(false);
    Effect::new(move || {
        if started.get_value() {
            return;
        }
        started.set_value(true);
        timeline.play();
    });

    let status = move || {
        if paused.get() {
            return "Paused";
        }
        match step_index.get() {
            0 => "Entering",
            1 => "Floating",
            2 => "Shifting",
            3 => "Fading",
            _ => "Idle",
        }
    };

    view! {
        <section class="hero">
            <div class="panel">
                <h2>"FluidTimeline"</h2>
                <p>
                    "Compose multi-step sequences outside the element. "
                    "The timeline only drives a FluidStyle signal; the element stays simple."
                </p>
                <div class="mode-row">
                    <span class="chip chip-button" class:active=move || step_index.get() == 0>
                        "Enter"
                    </span>
                    <span class="chip chip-button" class:active=move || step_index.get() == 1>
                        "Float"
                    </span>
                    <span class="chip chip-button" class:active=move || step_index.get() == 2>
                        "Shift"
                    </span>
                    <span class="chip chip-button" class:active=move || step_index.get() == 3>
                        "Fade"
                    </span>
                    <span
                        class="chip chip-button"
                        class:active=move || auto_loop.get()
                        on:click=toggle_loop
                    >
                        "Auto-loop"
                    </span>
                </div>
                <div class="button-row">
                    <button on:click=play>"Play timeline"</button>
                    <button class="alt" on:click=pause>
                        "Pause"
                    </button>
                    <button class="alt" on:click=resume>
                        "Resume"
                    </button>
                    <button class="alt" on:click=reset>
                        "Reset"
                    </button>
                </div>
            </div>

            <div class="presence-card timeline-card">
                <div class="presence-stage timeline-stage">
                    <div class="timeline-orb"></div>
                    <FluidDiv
                        class="timeline-widget"
                        initial=base_style
                        animate=animate
                        transition=transition
                        node_ref=node_ref
                    >
                        <span class="chip">"Timeline"</span>
                        <h3>"Motion state"</h3>
                        <p>{status}</p>
                    </FluidDiv>
                </div>
            </div>
        </section>
    }
}
