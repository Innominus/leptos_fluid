use leptos::prelude::*;
use leptos_fluid::motion::{use_spring, FluidDiv, FluidStyle, Spring, Transition};

#[component]
pub fn SpringFollowSection() -> impl IntoView {
    let arena_ref = NodeRef::<leptos::html::Div>::new();
    let follow_x = use_spring(0.0, Spring::new(600, 0.55));
    let follow_y = use_spring(0.0, Spring::new(600, 0.55));

    let ball_style = {
        let follow_x = follow_x.clone();
        let follow_y = follow_y.clone();
        move || {
            FluidStyle::new()
                .x(follow_x.get())
                .y(follow_y.get())
                .scale(1.0)
                .opacity(1.0)
                .with("box-shadow", "0 20px 50px rgba(15, 23, 42, 0.35)")
        }
    };

    view! {
        <section class="hero">
            <div class="panel">
                <h2>"Spring follow"</h2>
                <p>
                    "A Leptos take on motion.dev's cursor spring. Pointer movement sets the target, "
                    "the spring solves toward it with duration + bounce."
                </p>
                <div class="list">
                    <span class="chip">"use_spring"</span>
                    <span class="chip">"duration + bounce"</span>
                    <span class="chip">"pointer tracking"</span>
                </div>
            </div>

            <div
                class="follow-arena"
                node_ref=arena_ref
                on:pointermove={
                    let follow_x = follow_x.clone();
                    let follow_y = follow_y.clone();
                    move |ev| {
                        let Some(arena) = arena_ref.get_untracked() else {
                            return;
                        };
                        let rect = arena.get_bounding_client_rect();
                        let center_x = rect.left() + rect.width() / 2.0;
                        let center_y = rect.top() + rect.height() / 2.0;
                        let target_x = ev.client_x() as f64 - center_x;
                        let target_y = ev.client_y() as f64 - center_y;
                        follow_x.set(target_x);
                        follow_y.set(target_y);
                    }
                }
            >
                <FluidDiv
                    class="follow-ball"
                    initial=FluidStyle::new().opacity(1.0).scale(1.0)
                    animate=ball_style
                    transition=Transition::new().duration_ms(0)
                ></FluidDiv>
                <div class="follow-hint">"Move your cursor"</div>
            </div>
        </section>
    }
}
