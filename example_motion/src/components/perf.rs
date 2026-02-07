use std::collections::VecDeque;

use js_sys::Date;
use leptos::prelude::*;
use leptos_fluid::motion::{FluidDiv, FluidStyle, Transition};

const SAMPLE_WINDOW: usize = 120;
const UPDATE_EVERY: u32 = 10;

#[component]
pub fn PerfSection() -> impl IntoView {
    let running = RwSignal::new(false);
    let dot_count = RwSignal::new(80usize);
    let tick = RwSignal::new(0.0);

    let avg_ms = RwSignal::new(0.0);
    let p95_ms = RwSignal::new(0.0);
    let fps = RwSignal::new(0.0);
    let last_ms = RwSignal::new(0.0);

    let samples = StoredValue::new(VecDeque::with_capacity(SAMPLE_WINDOW));
    let last_time = StoredValue::new(None::<f64>);
    let frame_count = StoredValue::new(0u32);

    Effect::new({
        let running = running.clone();
        let samples = samples.clone();
        let last_time = last_time.clone();
        let frame_count = frame_count.clone();
        let avg_ms = avg_ms.clone();
        let p95_ms = p95_ms.clone();
        let fps = fps.clone();
        let last_ms = last_ms.clone();
        let tick = tick.clone();
        move || {
            if !running.get() {
                return;
            }
            samples.set_value(VecDeque::with_capacity(SAMPLE_WINDOW));
            last_time.set_value(None);
            frame_count.set_value(0);
            schedule_perf_loop(
                running,
                samples,
                last_time,
                frame_count,
                avg_ms,
                p95_ms,
                fps,
                last_ms,
                tick,
            );
        }
    });

    let dots = move || (0..dot_count.get()).collect::<Vec<_>>();

    view! {
        <section class="perf-section">
            <div class="panel">
                <h2>"Performance"</h2>
                <p>
                    "A lightweight benchmark loop to sample frame times while driving a small FluidDiv swarm. "
                    "Use it to compare changes and spot regressions."
                </p>
                <div class="button-row">
                    <button on:click=move |_| {
                        running.update(|value| *value = !*value)
                    }>
                        {move || if running.get() { "Stop benchmark" } else { "Start benchmark" }}
                    </button>
                    <label class="perf-control">
                        <span>"Dots"</span>
                        <input
                            type="range"
                            min="20"
                            max="10000"
                            step="10"
                            prop:value=move || dot_count.get() as i32
                            on:input=move |ev| {
                                let value = event_target_value(&ev).parse::<usize>().unwrap_or(80);
                                dot_count.set(value);
                            }
                        />
                        <span class="perf-value">{move || dot_count.get()}</span>
                    </label>
                </div>
                <div class="perf-metrics">
                    <div>
                        <span class="label">"FPS"</span>
                        <strong>{move || format!("{:.1}", fps.get())}</strong>
                    </div>
                    <div>
                        <span class="label">"Avg"</span>
                        <strong>{move || format!("{:.2} ms", avg_ms.get())}</strong>
                    </div>
                    <div>
                        <span class="label">"P95"</span>
                        <strong>{move || format!("{:.2} ms", p95_ms.get())}</strong>
                    </div>
                    <div>
                        <span class="label">"Last"</span>
                        <strong>{move || format!("{:.2} ms", last_ms.get())}</strong>
                    </div>
                </div>
            </div>

            <div class="perf-stage">
                <For
                    each=dots
                    key=|index| *index
                    children=move |index| {
                        let tick = tick.clone();
                        let index_f = index as f64;
                        view! {
                            <FluidDiv
                                class="perf-dot"
                                initial=FluidStyle::new().opacity(0.9)
                                animate=move || {
                                    let t = tick.get();
                                    let angle = t * 1.2 + index_f * 0.42;
                                    let radius = 28.0 + (index_f % 10.0) * 6.0;
                                    let wobble = (t + index_f * 0.2).sin() * 6.0;
                                    let x = angle.cos() * (radius + wobble);
                                    let y = angle.sin() * (radius + wobble);
                                    let scale = 0.7 + ((t + index_f * 0.15).sin() * 0.5).abs();
                                    FluidStyle::new()
                                        .x(x)
                                        .y(y)
                                        .scale(scale)
                                        .opacity(0.6 + scale * 0.4)
                                }
                                transition=Transition::new().duration_ms(0)
                            ></FluidDiv>
                        }
                    }
                />
                <div class="perf-hint">
                    {move || if running.get() { "Sampling" } else { "Idle" }}
                </div>
            </div>
        </section>
    }
}

fn schedule_perf_loop(
    running: RwSignal<bool>,
    samples: StoredValue<VecDeque<f64>>,
    last_time: StoredValue<Option<f64>>,
    frame_count: StoredValue<u32>,
    avg_ms: RwSignal<f64>,
    p95_ms: RwSignal<f64>,
    fps: RwSignal<f64>,
    last_ms: RwSignal<f64>,
    tick: RwSignal<f64>,
) {
    request_animation_frame(move || {
        if !running.get_untracked() {
            return;
        }

        let now = Date::now();
        let last = last_time.get_value().unwrap_or(now);
        let mut dt = now - last;
        if dt <= 0.0 {
            dt = 16.0;
        }
        last_time.set_value(Some(now));
        last_ms.set(dt);

        let mut buffer = samples.get_value();
        buffer.push_back(dt);
        if buffer.len() > SAMPLE_WINDOW {
            buffer.pop_front();
        }
        samples.set_value(buffer);

        let mut frames = frame_count.get_value();
        frames += 1;
        frame_count.set_value(frames);

        let delta_seconds = dt / 1000.0;
        tick.set(tick.get_untracked() + delta_seconds);

        if frames % UPDATE_EVERY == 0 {
            let buffer = samples.get_value();
            let (avg, p95, fps_value) = compute_metrics(&buffer);
            avg_ms.set(avg);
            p95_ms.set(p95);
            fps.set(fps_value);
        }

        schedule_perf_loop(
            running,
            samples,
            last_time,
            frame_count,
            avg_ms,
            p95_ms,
            fps,
            last_ms,
            tick,
        );
    });
}

fn compute_metrics(samples: &VecDeque<f64>) -> (f64, f64, f64) {
    if samples.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let count = samples.len() as f64;
    let sum: f64 = samples.iter().sum();
    let avg = sum / count;

    let mut sorted: Vec<f64> = samples.iter().copied().collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((sorted.len() as f64) * 0.95).ceil() as usize;
    let idx = idx.saturating_sub(1).min(sorted.len() - 1);
    let p95 = sorted[idx];

    let fps = if avg > 0.0 { 1000.0 / avg } else { 0.0 };

    (avg, p95, fps)
}
