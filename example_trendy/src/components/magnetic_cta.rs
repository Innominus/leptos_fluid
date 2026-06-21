use leptos::prelude::*;

use leptos_fluid_motion::{AnimationController, FluidSignal, FluidStyle, Spring, use_spring};

#[component]
pub fn MagneticCtaSection() -> impl IntoView {
    let wrap_ref = NodeRef::<leptos::html::Div>::new();
    let btn_ref = NodeRef::<leptos::html::Button>::new();
    let spring_x = use_spring(0.0, Spring::new(400, 0.25));
    let spring_y = use_spring(0.0, Spring::new(400, 0.25));
    let is_hovered = RwSignal::new(false);

    let sx = spring_x.signal();
    let sy = spring_y.signal();

    let controller = AnimationController::builder()
        .target(btn_ref)
        .initial(FluidStyle::new().x(0.0).y(0.0))
        .install();

    controller.bind_set_immediate(FluidSignal::derive(move || {
        FluidStyle::new().x(sx.get()).y(sy.get()).with("transition", "transform 50ms linear")
    }));

    view! {
        <section class="section section-cta" id="cta">
            <div class="section-inner">
                <p class="kicker">"10 — Magnetic CTA"</p>
                <h2>"Let's talk."</h2>
                <p class="lead">
                    "A magenta pill drifts toward your cursor while hovering, then \
                     snaps back to center with a spring when you leave. Hover \
                     state adds a glow."
                </p>

                <div class="magnetic-wrap-area">
                    <div
                        class="magnetic-wrap"
                        node_ref=wrap_ref
                        on:mousemove={
                            let spring_x = spring_x.clone();
                            let spring_y = spring_y.clone();
                            move |ev| {
                                if let Some(el) = wrap_ref.get() {
                                    let rect = el.get_bounding_client_rect();
                                    let cx = rect.left() + rect.width() / 2.0;
                                    let cy = rect.top() + rect.height() / 2.0;
                                    spring_x.set((ev.client_x() as f64 - cx) * 0.3);
                                    spring_y.set((ev.client_y() as f64 - cy) * 0.3);
                                }
                            }
                        }
                        on:mouseenter=move |_| { is_hovered.set(true); }
                        on:mouseleave={
                            let spring_x = spring_x.clone();
                            let spring_y = spring_y.clone();
                            move |_| {
                                is_hovered.set(false);
                                spring_x.set(0.0);
                                spring_y.set(0.0);
                            }
                        }
                    >
                        <button
                            class="magnetic-btn"
                            node_ref=btn_ref
                            class:hovered=move || is_hovered.get()
                        >
                            "Get Started →"
                        </button>
                    </div>
                </div>

                <p class="cta-hint">"Hover and move your cursor."</p>
            </div>
        </section>
    }
}