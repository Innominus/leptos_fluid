use leptos::prelude::*;
use leptos_fluid_motion::{controller, timeline, Easing, FluidStyle, Transition};

#[component]
pub fn TimelineMacroExample() -> impl IntoView {
    let running = RwSignal::new(false);
    let glyph_ref = NodeRef::<leptos::html::Div>::new();
    let controller = controller! {
        target: glyph_ref,
        transition: Transition::new().duration_ms(240).easing(Easing::EaseInOut),
        initial: timeline_macro_rest(),
    };
    let seeded = StoredValue::new(false);

    Effect::new(move || {
        if seeded.get_value() || glyph_ref.get().is_none() {
            return;
        }
        seeded.set_value(true);
        controller.set_immediate(timeline_macro_rest());
    });
    let sequence = timeline! {
        controller: controller,
        initial: timeline_macro_rest(),
        autoplay: false,
        auto_loop: true,
        steps: [
            { to: timeline_macro_charge() },
            { to: timeline_macro_burst() },
            { to: timeline_macro_settle(), wait_ms: 180 },
        ],
        triggers: [
            on(running.get()) {
                true => restart(),
                false => set_immediate(timeline_macro_rest()),
            },
        ],
    };
    let step_index = sequence.step_index();
    let is_paused = sequence.is_paused();

    let toggle_running = move |_| {
        running.update(|value| *value = !*value);
    };
    let toggle_pause = move |_| {
        if !running.get_untracked() {
            return;
        }

        if is_paused.get_untracked() {
            sequence.resume();
        } else {
            sequence.pause();
        }
    };

    view! {
        <article class="demo-panel" data-testid="timeline-macro-panel">
            <div class="panel-header">
                <p class="panel-eyebrow">"timeline!"</p>
                <h2>"Trigger-driven declarative playback"</h2>
                <p>
                    "Compact declarative playback rules."
                </p>
            </div>

            <div class="button-row">
                <button data-testid="timeline-macro-toggle" on:click=toggle_running>
                    {move || if running.get() { "Stop scene" } else { "Start scene" }}
                </button>
                <button class="ghost" data-testid="timeline-macro-pause" on:click=toggle_pause>
                    {move || {
                        if !running.get() {
                            "Pause"
                        } else if is_paused.get() {
                            "Resume"
                        } else {
                            "Pause"
                        }
                    }}
                </button>
            </div>

            <div class="stage">
                <div
                    node_ref=glyph_ref
                    class="timeline-glyph macro-glyph"
                    data-testid="timeline-macro-glyph"
                >
                    <p class="chip">"timeline!"</p>
                    <h3>"Macro-driven scene"</h3>
                    <p data-testid="timeline-macro-status">
                        {move || macro_timeline_status(
                            running.get(),
                            is_paused.get(),
                            step_index.get(),
                        )}
                    </p>
                </div>
            </div>
        </article>
    }
}

fn macro_timeline_status(running: bool, paused: bool, step_index: usize) -> &'static str {
    if !running {
        return "Stopped and reset to the resting style.";
    }
    if paused {
        return "Paused by trigger rule.";
    }

    match step_index {
        0 => "Charging the scene.",
        1 => "Bursting into the live state.",
        2 => "Holding the final settle state before the loop resets.",
        _ => "Running.",
    }
}

fn timeline_macro_rest() -> FluidStyle {
    FluidStyle::new()
        .opacity(0.72)
        .x(0.0)
        .y(14.0)
        .scale(0.9)
        .rotate(0.0)
        .with("background", "#ffedd5")
        .with("color", "#7c2d12")
        .with("box-shadow", "0 14px 28px rgba(154,52,18,.14)")
}

fn timeline_macro_charge() -> FluidStyle {
    FluidStyle::new()
        .opacity(0.96)
        .x(-18.0)
        .y(-4.0)
        .scale(1.0)
        .rotate(-2.0)
        .with("background", "#f59e0b")
        .with("color", "#fff7ed")
        .with("box-shadow", "0 22px 42px rgba(194,65,12,.22)")
}

fn timeline_macro_burst() -> FluidStyle {
    FluidStyle::new()
        .opacity(1.0)
        .x(22.0)
        .y(-16.0)
        .scale(1.06)
        .rotate(3.4)
        .with("background", "#db2777")
        .with("color", "#fdf2f8")
        .with("box-shadow", "0 30px 56px rgba(190,24,93,.26)")
}

fn timeline_macro_settle() -> FluidStyle {
    FluidStyle::new()
        .opacity(0.96)
        .x(10.0)
        .y(6.0)
        .scale(1.0)
        .rotate(-1.4)
        .with("background", "#7c3aed")
        .with("color", "#eef2ff")
        .with("box-shadow", "0 24px 48px rgba(37,99,235,.24)")
}
