use leptos::prelude::*;
use leptos_fluid_motion::{use_spring, FluidDiv, FluidStyle, FluidValue, Spring, Transition};

#[component]
pub fn SpringFollowSection() -> impl IntoView {
    let arena_ref = NodeRef::<leptos::html::Div>::new();
    let follow_x = use_spring(0.0, Spring::new(640, 0.5));
    let follow_y = use_spring(0.0, Spring::new(640, 0.5));

    let ball_style = {
        let follow_x = follow_x.clone();
        let follow_y = follow_y.clone();
        move || cursor_ball_style(follow_x.get(), follow_y.get())
    };

    view! {
        <section class="section-grid">
            <div class="panel">
                <p class="section-kicker">"Spring follow"</p>
                <h2>"Continuous retargeting with use_spring"</h2>
                <p>
                    "Pointer movement only updates the target values. The spring resolves the motion continuously, so the wrapper itself can stay on a zero-duration transition."
                </p>
            </div>

            <div
                class="follow-arena"
                node_ref=arena_ref
                on:pointermove={
                    let follow_x = follow_x.clone();
                    let follow_y = follow_y.clone();
                    move |event| {
                        let Some(arena) = arena_ref.get_untracked() else {
                            return;
                        };
                        let rect = arena.get_bounding_client_rect();
                        let center_x = rect.left() + rect.width() / 2.0;
                        let center_y = rect.top() + rect.height() / 2.0;
                        follow_x.set(event.client_x() as f64 - center_x);
                        follow_y.set(event.client_y() as f64 - center_y);
                    }
                }
            >
                <FluidDiv
                    class="follow-ball"
                    initial=FluidStyle::new().opacity(1.0).scale(1.0)
                    animate=ball_style
                    transition=Transition::new().duration_ms(0)
                ></FluidDiv>
                <p class="follow-hint">"Move your cursor through the arena."</p>
            </div>
        </section>
    }
}

fn cursor_ball_style(x: f64, y: f64) -> FluidStyle {
    FluidStyle::new()
        .x(x)
        .y(y)
        .scale(1.0)
        .opacity(1.0)
        .with_prop(
            "box-shadow",
            FluidValue::from("0 22px 56px rgba(6, 7, 18, 0.42)"),
        )
}
