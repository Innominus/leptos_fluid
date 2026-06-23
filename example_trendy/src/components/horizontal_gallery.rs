use leptos::prelude::*;

use leptos_fluid_motion::{AnimationController, Easing, FluidStyle, Transition};
use leptos_fluid_scroll::prelude::*;

const FALLBACK_TRACK_WIDTH: f64 = 2200.0;

const CARDS: &[(&str, &str, &str)] = &[
    ("01", "Aurora", "Ambient gradient field for editorial headers."),
    ("02", "Velocity", "Motion-tuned dashboards with live scrubbing."),
    ("03", "Cascade", "Layered reveal sequences for product pages."),
    ("04", "Mirage", "Glassmorphic surfaces with depth-aware tilt."),
    ("05", "Pulse", "Reactive number counters driven by scroll progress."),
];

#[component]
pub fn HorizontalGallerySection() -> impl IntoView {
    let section_ref = NodeRef::<leptos::html::Section>::new();
    let track_ref = NodeRef::<leptos::html::Div>::new();

    // One-shot runtime measurement of the track width so the scroll-bound
    // translate maps `p=0..1` onto the actual scrollable track content.
    let track_width = StoredValue::<f64, LocalStorage>::new_local(FALLBACK_TRACK_WIDTH);
    Effect::new(move || {
        if let Some(track) = track_ref.get() {
            let scroll_w = track.scroll_width() as f64;
            if scroll_w > 0.0 {
                if let Some(window) = web_sys::window() {
                    if let Ok(w) = window.inner_width() {
                        if let Some(w) = w.as_f64() {
                            if scroll_w > w {
                                track_width.set_value(scroll_w - w);
                                return;
                            }
                        }
                    }
                }
                track_width.set_value(scroll_w);
            }
        }
    });

    let track_controller = AnimationController::builder()
        .target(track_ref)
        .transition(Transition::new().duration_ms(200).easing(Easing::EaseOut))
        .initial(FluidStyle::new().x(0.0))
        .install();

    let trigger = ScrollTrigger::builder()
        .target(section_ref)
        .start("top top")
        .end("bottom bottom")
        .scrub(Scrub::Number(0.15))
        .bind_controller(track_controller, Box::new(move |p| {
            FluidStyle::new().x(-(p * track_width.get_value()))
        }))
        .install();

    let progress = trigger.progress();
    let track_pct = Memo::new(move |_| (progress.get() * 100.0).round() as i32);

    view! {
        <section class="section section-gallery" id="gallery" node_ref=section_ref>
            <div class="gallery-spacer">
                <div class="sticky-pin">
                    <div class="gallery-inner">
                        <p class="kicker gallery-kicker">"02 — Horizontal Gallery"</p>
                        <div class="gallery-track" node_ref=track_ref>
                            {
                                CARDS.iter().map(|card| {
                                    view! {
                                        <article class="gallery-card">
                                            <span class="card-numeral">{card.0}</span>
                                            <h3 class="card-name">{card.1}</h3>
                                            <p class="card-desc">{card.2}</p>
                                        </article>
                                    }
                                }).collect::<Vec<_>>()
                            }
                        </div>
                        <div class="gallery-progress">
                            <div class="progress-track">
                                <div
                                    class="progress-fill"
                                    style:width=move || format!("{}%", track_pct.get())
                                ></div>
                            </div>
                            <span class="badge">
                                {move || format!("track {}%", track_pct.get())}
                            </span>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}