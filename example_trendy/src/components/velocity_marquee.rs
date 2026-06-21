use leptos::prelude::*;

use leptos_fluid_motion::{AnimationController, FluidStyle};
use leptos_fluid_scroll::prelude::*;

const MARQUEE_TEXT: &str = "SCROLL • DRIVES • MOTION • ";

const FALLBACK_ITEM_WIDTH: f64 = 400.0;
const BASE_SPEED: f64 = 50.0;
const VELOCITY_SCALE: f64 = 200.0;

#[component]
pub fn VelocityMarqueeSection() -> impl IntoView {
    let section_ref = NodeRef::<leptos::html::Section>::new();
    let track_ref = NodeRef::<leptos::html::Div>::new();
    let first_item_ref = NodeRef::<leptos::html::Span>::new();

    let trigger = ScrollTrigger::builder()
        .target(section_ref)
        .start("top 90%")
        .end("bottom 10%")
        .scrub(Scrub::Bool(false))
        .install();

    let velocity = trigger.velocity();
    let direction = trigger.direction();
    let is_active = trigger.is_active();

    let track_controller = AnimationController::builder()
        .target(track_ref)
        .initial(FluidStyle::new().x(0.0))
        .install();

    let item_width = StoredValue::<f64, LocalStorage>::new_local(FALLBACK_ITEM_WIDTH);

    Effect::new(move || {
        if let Some(el) = first_item_ref.get() {
            let rect = el.get_bounding_client_rect();
            let w = rect.width();
            if w > 0.0 {
                item_width.set_value(w);
            }
        }
    });

    schedule_marquee_tick(track_controller, velocity, direction, item_width, is_active);

    let rest_items = (0..5)
        .map(|_| {
            view! {
                <span class="marquee-item">{MARQUEE_TEXT}</span>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <section class="section section-marquee" id="marquee" node_ref=section_ref>
            <div class="section-inner marquee-inner">
                <p class="kicker">"08 — Velocity Marquee"</p>
            </div>
            <div class="marquee-viewport">
                <div class="marquee-track" node_ref=track_ref>
                    <span class="marquee-item" node_ref=first_item_ref>{MARQUEE_TEXT}</span>
                    {rest_items}
                </div>
            </div>
            <div class="section-inner marquee-foot">
                <div class="indicator">
                    <span class="badge">
                        {move || format!("velocity {:.0} px/s", velocity.get().abs())}
                    </span>
                    <span class="badge">
                        {move || {
                            let d = direction.get();
                            if d > 0 { "forward" } else if d < 0 { "reverse" } else { "idle" }
                        }}
                    </span>
                </div>
            </div>
        </section>
    }
}

fn schedule_marquee_tick(
    track_controller: AnimationController,
    velocity: Signal<f64>,
    direction: Signal<i8>,
    item_width: StoredValue<f64, LocalStorage>,
    is_active: Signal<bool>,
) {
    let offset: StoredValue<f64, LocalStorage> = StoredValue::new_local(0.0);
    let handle: StoredValue<Option<AnimationFrameRequestHandle>, LocalStorage> =
        StoredValue::new_local(None);
    let cancelled: StoredValue<bool, LocalStorage> = StoredValue::new_local(false);

    on_cleanup(move || {
        cancelled.set_value(true);
        if let Some(h) = handle.get_value() {
            h.cancel();
        }
    });

    step_marquee(track_controller, offset, velocity, direction, item_width, handle, cancelled, is_active);
}

fn step_marquee(
    track_controller: AnimationController,
    offset: StoredValue<f64, LocalStorage>,
    velocity: Signal<f64>,
    direction: Signal<i8>,
    item_width: StoredValue<f64, LocalStorage>,
    handle: StoredValue<Option<AnimationFrameRequestHandle>, LocalStorage>,
    cancelled: StoredValue<bool, LocalStorage>,
    is_active: Signal<bool>,
) {
    let new_handle = request_animation_frame_with_handle(move || {
        if cancelled.get_value() {
            return;
        }

        // Skip style writes when the section is offscreen; keep the rAF
        // loop alive so the marquee resumes immediately when scrolled back.
        if !is_active.get_untracked() {
            if cancelled.get_value() {
                return;
            }
            step_marquee(track_controller, offset, velocity, direction, item_width, handle, cancelled, is_active);
            return;
        }

        let v = velocity.get_untracked();
        let d = direction.get_untracked();
        let step = BASE_SPEED + v.abs() / VELOCITY_SCALE;
        let wrap = item_width.get_value();

        let new_offset = {
            let o = offset.get_value();
            if d < 0 {
                let new = o + step;
                if new >= 0.0 { -wrap + new } else { new }
            } else {
                let new = o - step;
                if new <= -wrap { 0.0 } else { new }
            }
        };
        offset.set_value(new_offset);

        track_controller.set_immediate(
            FluidStyle::new().x(new_offset).with("transition", "transform 30ms linear"),
        );

        if cancelled.get_value() {
            return;
        }
        step_marquee(track_controller, offset, velocity, direction, item_width, handle, cancelled, is_active);
    });

    if let Ok(h) = new_handle {
        handle.set_value(Some(h));
    }
}