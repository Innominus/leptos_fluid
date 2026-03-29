use leptos::prelude::*;
use leptos_fluid_motion::{FluidDiv, FluidStyle, Transition};

#[component]
pub fn PerfSection() -> impl IntoView {
    let running = RwSignal::new(false);
    let dot_count = RwSignal::new(120usize);
    let tick = RwSignal::new(0.0);

    Effect::new(move || {
        if !running.get() {
            return;
        }
        schedule_tick(running, tick);
    });

    let dots = move || (0..dot_count.get()).collect::<Vec<_>>();

    view! {
        <section class="section-grid">
            <div class="panel">
                <p class="section-kicker">"Perf field"</p>
                <h2>"A simple stress scene"</h2>
                <p>
                    "This keeps a dense field of tiny FluidDiv nodes moving so you can sanity-check visual smoothness while iterating on the runtime."
                </p>
                <div class="button-row">
                    <button on:click=move |_| running.update(|value| *value = !*value)>
                        {move || if running.get() { "Pause field" } else { "Start field" }}
                    </button>
                    <label class="perf-control">
                        <span>"Dots"</span>
                        <input
                            type="range"
                            min="24"
                            max="720"
                            step="12"
                            prop:value=move || dot_count.get() as i32
                            on:input=move |event| {
                                let value = event_target_value(&event).parse::<usize>().unwrap_or(120);
                                dot_count.set(value);
                            }
                        />
                        <span class="perf-value">{move || dot_count.get()}</span>
                    </label>
                </div>
                <p class="panel-note">
                    {move || if running.get() { "Running the field" } else { "Field paused" }}
                </p>
            </div>

            <div class="perf-stage">
                {move || {
                    dots()
                        .into_iter()
                        .map(|index| {
                            let tick = tick;
                            let index_f = index as f64;

                            view! {
                                <FluidDiv
                                    class="perf-dot"
                                    initial=FluidStyle::new().opacity(0.85)
                                    animate=move || perf_dot_style(tick.get(), index_f)
                                    transition=Transition::new().duration_ms(0)
                                ></FluidDiv>
                            }
                        })
                        .collect_view()
                }}
                <p class="perf-hint">{move || if running.get() { "Sampling the field" } else { "Idle" }}</p>
            </div>
        </section>
    }
}

fn schedule_tick(running: RwSignal<bool>, tick: RwSignal<f64>) {
    request_animation_frame(move || {
        if !running.get_untracked() {
            return;
        }

        tick.update(|value| *value += 0.016);
        schedule_tick(running, tick);
    });
}

fn perf_dot_style(tick: f64, index: f64) -> FluidStyle {
    let angle = tick * 1.1 + index * 0.28;
    let radius = 34.0 + (index % 14.0) * 5.2;
    let wobble = (tick * 1.35 + index * 0.21).sin() * 7.0;
    let scale = 0.66 + ((tick + index * 0.18).sin() * 0.4).abs();

    FluidStyle::new()
        .x(angle.cos() * (radius + wobble))
        .y(angle.sin() * (radius + wobble))
        .scale(scale)
        .opacity(0.5 + scale * 0.45)
}
