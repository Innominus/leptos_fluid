use leptos::prelude::*;

use leptos_fluid_motion::{AnimationController, Easing, FluidStyle, Transition};
use leptos_fluid_scroll::prelude::*;

const TARGET: f64 = 1247.0;
const COUNT_DURATION_MS: f64 = 1500.0;

#[component]
pub fn CounterSection() -> impl IntoView {
    let section_ref = NodeRef::<leptos::html::Section>::new();
    let display_ref = NodeRef::<leptos::html::Div>::new();
    let count = RwSignal::new(0i64);

    let display_controller = AnimationController::builder()
        .target(display_ref)
        .transition(Transition::new().duration_ms(600).easing(Easing::EaseOut))
        .initial(FluidStyle::new().scale(0.8).opacity(0.0))
        .install();

    let display_controller_for_cb = display_controller;
    let trigger = ScrollTrigger::builder()
        .target(section_ref)
        .start("top 80%")
        .end("bottom 50%")
        .scrub(Scrub::Bool(false))
        .once(true)
        .on_enter(move |_| {
            display_controller_for_cb.animate(FluidStyle::new().scale(1.1).opacity(1.0));
            run_count_animation(count);
        })
        .install();

    let progress = trigger.progress();
    // The number counting is a one-shot rAF-driven counter that animates from 0
    // to TARGET over ~1.5s starting on enter; the container scale/fade is
    // motion-crate-driven via display_controller.animate() in on_enter.

    view! {
        <section class="section section-counter" id="counter" node_ref=section_ref>
            <div class="section-inner counter-inner">
                <p class="kicker">"05 — Number Counter"</p>
                <div class="counter-display" node_ref=display_ref>
                    <span class="counter-number">{move || format!("{}", count.get())}</span>
                    <span class="counter-suffix">"+"</span>
                </div>
                <p class="counter-label">"Active users"</p>
                <p class="counter-sublabel">"Updated in real time as you scroll."</p>
                <div class="indicator">
                    <span class="badge">
                        {move || format!("progress {:.2}", progress.get())}
                    </span>
                </div>
            </div>
        </section>
    }
}

/// One-shot rAF-driven counter: animates `count` from 0 to TARGET over ~1.5s.
/// Cancels any pending frame on unmount via `on_cleanup`.
fn run_count_animation(count: RwSignal<i64>) {
    let cancelled: StoredValue<bool, LocalStorage> = StoredValue::new_local(false);
    let handle: StoredValue<Option<AnimationFrameRequestHandle>, LocalStorage> =
        StoredValue::new_local(None);
    let start: StoredValue<Option<f64>, LocalStorage> = StoredValue::new_local(None);

    on_cleanup(move || {
        cancelled.set_value(true);
        if let Some(h) = handle.get_value() {
            h.cancel();
        }
    });

    fn step(
        count: RwSignal<i64>,
        cancelled: StoredValue<bool, LocalStorage>,
        handle: StoredValue<Option<AnimationFrameRequestHandle>, LocalStorage>,
        start: StoredValue<Option<f64>, LocalStorage>,
    ) {
        let new_handle = request_animation_frame_with_handle(move || {
            if cancelled.get_value() {
                return;
            }
            let now = js_sys::Date::now();
            // Capture the start timestamp on the first frame so subsequent
            // frames compute `elapsed = now - t0` correctly. Without this,
            // `start` stays `None` forever and `t0 = now` every frame → the
            // counter never advances past 0.
            let t0 = start.get_value().unwrap_or_else(|| {
                start.set_value(Some(now));
                now
            });
            let elapsed = now - t0;
            let p = (elapsed / COUNT_DURATION_MS).clamp(0.0, 1.0);
            // ease-out
            let eased: f64 = 1.0 - (1.0 - p).powi(2);
            count.set((eased * TARGET) as i64);
            if p < 1.0 {
                step(count, cancelled, handle, start);
            }
        });
        if let Ok(h) = new_handle {
            handle.set_value(Some(h));
        }
    }

    step(count, cancelled, handle, start);
}