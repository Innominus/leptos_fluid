use leptos::prelude::*;
use leptos_fluid::motion::{
    style, use_spring, Easing, MotionButton, MotionDiv, MotionSpan, MotionStyle, Spring, Transition,
};

#[component]
pub fn App() -> impl IntoView {
    let hero_toggle = RwSignal::new(false);
    let pulse = RwSignal::new(true);
    let card_focus = RwSignal::new(false);
    let island_open = RwSignal::new(false);
    let arena_ref = NodeRef::<leptos::html::Div>::new();
    let follow_x = use_spring(0.0, Spring::new(600, 0.55));
    let follow_y = use_spring(0.0, Spring::new(600, 0.55));
    let active_tab = RwSignal::new(0usize);
    let tabs_ref = NodeRef::<leptos::html::Div>::new();
    let tab_refs = {
        let mut refs = Vec::new();
        refs.push(NodeRef::<leptos::html::Button>::new());
        refs.push(NodeRef::<leptos::html::Button>::new());
        refs.push(NodeRef::<leptos::html::Button>::new());
        refs.push(NodeRef::<leptos::html::Button>::new());
        refs
    };
    let tab_refs_effect = tab_refs.clone();
    let underline_x = RwSignal::new(0.0);
    let underline_w = RwSignal::new(0.0);

    Effect::new({
        let tabs_ref = tabs_ref.clone();
        let tab_refs_effect = tab_refs_effect.clone();
        move || {
            let index = active_tab.get();
            let tabs_ref = tabs_ref.clone();
            let tab_refs_effect = tab_refs_effect.clone();
            request_animation_frame(move || {
                let Some(container) = tabs_ref.get_untracked() else {
                    return;
                };
                let Some(tab_ref) = tab_refs_effect.get(index) else {
                    return;
                };
                let Some(tab) = tab_ref.get_untracked() else {
                    return;
                };
                let container_rect = container.get_bounding_client_rect();
                let tab_rect = tab.get_bounding_client_rect();
                underline_x.set(tab_rect.left() - container_rect.left());
                underline_w.set(tab_rect.width());
            });
        }
    });

    let hero_style = move || {
        if hero_toggle.get() {
            MotionStyle::new()
                .opacity(1.0)
                .x(0.0)
                .y(0.0)
                .scale(1.0)
                .rotate(-1.0)
                .with("box-shadow", "0 30px 80px rgba(6, 7, 18, 0.6)")
        } else {
            MotionStyle::new()
                .opacity(0.7)
                .x(-12.0)
                .y(12.0)
                .scale(0.96)
                .rotate(1.5)
                .with("box-shadow", "0 18px 50px rgba(6, 7, 18, 0.4)")
        }
    };

    let pulse_style = move || {
        if pulse.get() {
            MotionStyle::new().opacity(0.9).scale(1.0)
        } else {
            MotionStyle::new().opacity(0.4).scale(0.86)
        }
    };

    let focus_style = move || {
        if card_focus.get() {
            MotionStyle::new()
                .opacity(1.0)
                .scale(1.02)
                .with("border-color", "rgba(116, 241, 255, 0.8)")
        } else {
            MotionStyle::new()
                .opacity(0.9)
                .scale(1.0)
                .with("border-color", "rgba(255, 255, 255, 0.08)")
        }
    };

    let chip_delay = |index: usize| {
        Transition::new()
            .duration_ms(420)
            .bounce(0.2)
            .delay_ms(90 * index as u32)
    };

    let island_style = move || {
        if island_open.get() {
            MotionStyle::new()
                .width(360.0)
                .height(140.0)
                .opacity(1.0)
                .scale(1.0)
                .with("border-radius", "32px")
                .with("background", "linear-gradient(150deg, #0b0d18, #111827)")
        } else {
            MotionStyle::new()
                .width(170.0)
                .height(46.0)
                .opacity(0.95)
                .scale(1.0)
                .with("border-radius", "999px")
                .with("background", "linear-gradient(150deg, #0b0d18, #0f172a)")
        }
    };

    let island_glow = move || {
        if island_open.get() {
            MotionStyle::new()
                .opacity(0.8)
                .scale(1.0)
                .with("filter", "blur(18px)")
        } else {
            MotionStyle::new()
                .opacity(0.0)
                .scale(0.9)
                .with("filter", "blur(28px)")
        }
    };

    let island_content = move || {
        if island_open.get() {
            MotionStyle::new().opacity(1.0).y(0.0)
        } else {
            MotionStyle::new().opacity(0.0).y(12.0)
        }
    };

    let ball_style = {
        let follow_x = follow_x.clone();
        let follow_y = follow_y.clone();
        move || {
            MotionStyle::new()
                .x(follow_x.get())
                .y(follow_y.get())
                .scale(1.0)
                .opacity(1.0)
                .with("box-shadow", "0 20px 50px rgba(15, 23, 42, 0.35)")
        }
    };

    let underline_style = move || {
        MotionStyle::new()
            .x(underline_x.get())
            .width(underline_w.get())
            .opacity(1.0)
    };

    view! {
        <main class="page">
            <section class="hero">
                <div class="panel">
                    <span class="tag">"Leptos Fluid"</span>
                    <h1>"Motion playground that actually moves."</h1>
                    <p>
                        "Each panel showcases a different combination of MotionStyle, transitions, and hover/tap variants. "
                        "Toggle the controls to see everything react in real time."
                    </p>
                    <div class="button-row">
                        <button on:click=move |_| {
                            hero_toggle.update(|val| *val = !*val)
                        }>
                            {move || if hero_toggle.get() { "Reset hero" } else { "Throw it off" }}
                        </button>
                        <button class="alt" on:click=move |_| pulse.update(|val| *val = !*val)>
                            {move || if pulse.get() { "Dim pulse" } else { "Wake pulse" }}
                        </button>
                        <button class="alt" on:click=move |_| card_focus.update(|val| *val = !*val)>
                            {move || if card_focus.get() { "Unfocus cards" } else { "Focus cards" }}
                        </button>
                    </div>
                </div>

                <MotionDiv
                    class="glass"
                    initial=MotionStyle::new().opacity(0.0).y(30.0)
                    animate=hero_style
                    transition=Transition::spring_with(620, 0.45)
                    while_hover=MotionStyle::new().scale(1.02)
                    while_tap=MotionStyle::new().scale(0.98)
                >
                    <div class="orb one"></div>
                    <div class="orb two"></div>
                    <h2>"Hero card"</h2>
                    <p>
                        "Animated with a spring, rotated transforms, and dynamic shadows. Uses while_hover and while_tap for micro interactions."
                    </p>
                    <MotionButton
                        class="alt"
                        initial=MotionStyle::new().opacity(0.0).y(10.0)
                        animate=move || MotionStyle::new().opacity(1.0).y(0.0)
                        transition=Transition::new().duration_ms(360).easing(Easing::EaseOut)
                        while_hover=MotionStyle::new().scale(1.04)
                        while_tap=MotionStyle::new().scale(0.96)
                    >
                        "MotionButton"
                    </MotionButton>
                </MotionDiv>
            </section>

            <section class="grid">
                <MotionDiv
                    class="card"
                    initial=MotionStyle::new().opacity(0.0).y(20.0)
                    animate=focus_style
                    transition=Transition::new().duration_ms(420).bounce(0.35)
                >
                    <h3>"Reactive focus"</h3>
                    <p>"Drive border/opacity from any signal or closure."</p>
                </MotionDiv>

                <MotionDiv
                    class="card"
                    initial=MotionStyle::new().opacity(0.0).y(24.0)
                    animate=MotionStyle::new().opacity(1.0).y(0.0)
                    transition=Transition::spring_with(520, 0.6)
                    while_hover=MotionStyle::new().scale(1.03)
                >
                    <h3>"Hover lift"</h3>
                    <p>"Easing + hover scale for quick emphasis."</p>
                </MotionDiv>

                <MotionDiv
                    class="card"
                    initial=MotionStyle::new().opacity(0.0).x(-24.0)
                    animate=MotionStyle::new().opacity(1.0).x(0.0)
                    transition=Transition::new().duration_ms(520).bounce(0.2)
                    while_tap=MotionStyle::new().scale(0.97)
                >
                    <h3>"Slide & tap"</h3>
                    <p>"Different combos of initial, animate, and tap variants."</p>
                </MotionDiv>

                <MotionDiv
                    class="card"
                    initial=MotionStyle::new().opacity(0.0).y(20.0)
                    animate=move || {
                        style!(
                            "opacity" => 1.0,
                            "background" => "linear-gradient(140deg, rgba(255, 255, 255, 0.08), rgba(255, 255, 255, 0.02))",
                            "border-color" => "rgba(255, 139, 214, 0.4)"
                        )
                            .y(0.0)
                            .scale(1.0)
                    }
                    transition=Transition::snappy().bounce(0.15)
                    while_hover=MotionStyle::new().scale(1.03).rotate(-0.6)
                >
                    <h3>"style! macro"</h3>
                    <p>"Use the macro + builders together for rich styles."</p>
                </MotionDiv>
            </section>

            <section class="tabs-demo">
                <div class="panel">
                    <h2>"Interruptible underline"</h2>
                    <p>
                        "Click while the underline is moving. The animation reads its current progress and"
                        "retargets smoothly to the next tab instead of snapping."
                    </p>
                    <div class="list">
                        <span class="chip">"progress aware"</span>
                        <span class="chip">"spring retarget"</span>
                        <span class="chip">"mid-flight updates"</span>
                    </div>
                </div>

                <div class="tabs-card">
                    <div class="tab-list" node_ref=tabs_ref>
                        <button
                            node_ref=tab_refs[0]
                            class="tab-button"
                            class:active=move || active_tab.get() == 0
                            on:click=move |_| active_tab.set(0)
                        >
                            "Overview"
                        </button>
                        <button
                            node_ref=tab_refs[1]
                            class="tab-button"
                            class:active=move || active_tab.get() == 1
                            on:click=move |_| active_tab.set(1)
                        >
                            "Motion"
                        </button>
                        <button
                            node_ref=tab_refs[2]
                            class="tab-button"
                            class:active=move || active_tab.get() == 2
                            on:click=move |_| active_tab.set(2)
                        >
                            "Tokens"
                        </button>
                        <button
                            node_ref=tab_refs[3]
                            class="tab-button"
                            class:active=move || active_tab.get() == 3
                            on:click=move |_| active_tab.set(3)
                        >
                            "Notes"
                        </button>
                        <MotionDiv
                            class="tab-underline"
                            initial=MotionStyle::new().x(0.0).width(0.0)
                            animate=underline_style
                            transition=Transition::spring_with(1000, 10.0)
                        ></MotionDiv>
                    </div>
                    <div class="tab-panel">
                        {move || match active_tab.get() {
                            0 => view! { <p>"Overview content slides in smoothly."</p> }.into_any(),
                            1 => view! { <p>"Motion settings react mid-flight."</p> }.into_any(),
                            2 => view! { <p>"Tokens shift without snapping."</p> }.into_any(),
                            _ => view! { <p>"Notes keep the underline moving."</p> }.into_any(),
                        }}
                    </div>
                </div>
            </section>

            <section class="island-shell">
                <div class="panel">
                    <h2>"Dynamic island"</h2>
                    <p>
                        "A nod to Emil Kowalski's iOS island recreations. This uses width/height, border-radius, opacity, "
                        "and spring bounce to morph the shape."
                    </p>
                    <div class="button-row">
                        <button
                            class="alt"
                            on:click=move |_| island_open.update(|val| *val = !*val)
                        >
                            {move || {
                                if island_open.get() { "Collapse island" } else { "Expand island" }
                            }}
                        </button>
                    </div>
                </div>

                <div class="island-wrap">
                    <MotionDiv
                        class="island-glow"
                        initial=MotionStyle::new().opacity(0.0).scale(0.9)
                        animate=island_glow
                        transition=Transition::spring_with(680, 0.8)
                    ></MotionDiv>

                    <MotionDiv
                        class="island"
                        initial=MotionStyle::new().width(170.0).height(46.0).opacity(0.0)
                        animate=island_style
                        transition=Transition::spring_with(700, 0.7)
                        while_hover=MotionStyle::new().scale(1.02)
                    >
                        <div class="island-inner">
                            <div class="island-pill">
                                <div class="island-dot"></div>
                                <div class="island-wave"></div>
                            </div>
                            <MotionDiv
                                class="island-content"
                                initial=MotionStyle::new().opacity(0.0).y(8.0)
                                animate=island_content
                                transition=Transition::new().duration_ms(360).bounce(0.25)
                            >
                                <div>
                                    <p class="island-title">"Now Playing"</p>
                                    <p class="island-sub">"Midnight City — M83"</p>
                                </div>
                                <div class="island-meter">
                                    <span></span>
                                    <span></span>
                                    <span></span>
                                </div>
                            </MotionDiv>
                        </div>
                    </MotionDiv>
                </div>
            </section>

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
                    <MotionDiv
                        class="follow-ball"
                        initial=MotionStyle::new().opacity(1.0).scale(1.0)
                        animate=ball_style
                        transition=Transition::new().duration_ms(0)
                    ></MotionDiv>
                    <div class="follow-hint">"Move your cursor"</div>
                </div>
            </section>

            <section class="hero">
                <div class="panel">
                    <h2>"Staggered chips"</h2>
                    <p>
                        "Using MotionSpan with different delay values to get a simple stagger without extra runtime."
                        "Combine translate + opacity for a clean reveal."
                    </p>
                    <div class="list">
                        <MotionSpan
                            class="chip"
                            initial=MotionStyle::new().opacity(0.0).x(-16.0)
                            animate=MotionStyle::new().opacity(1.0).x(0.0)
                            transition=chip_delay(1)
                        >
                            "Initial → animate"
                        </MotionSpan>
                        <MotionSpan
                            class="chip"
                            initial=MotionStyle::new().opacity(0.0).x(-16.0)
                            animate=MotionStyle::new().opacity(1.0).x(0.0)
                            transition=chip_delay(2)
                        >
                            "Custom delay"
                        </MotionSpan>
                        <MotionSpan
                            class="chip"
                            initial=MotionStyle::new().opacity(0.0).x(-16.0)
                            animate=MotionStyle::new().opacity(1.0).x(0.0)
                            transition=chip_delay(3)
                        >
                            "MotionSpan"
                        </MotionSpan>
                        <MotionSpan
                            class="chip"
                            initial=MotionStyle::new().opacity(0.0).x(-16.0)
                            animate=MotionStyle::new().opacity(1.0).x(0.0)
                            transition=chip_delay(4)
                        >
                            "Lightweight"
                        </MotionSpan>
                    </div>
                </div>

                <MotionDiv
                    class="glass"
                    initial=MotionStyle::new().opacity(0.0).y(26.0)
                    animate=move || {
                        MotionStyle::new()
                            .opacity(1.0)
                            .y(0.0)
                            .scale(1.0)
                            .with(
                                "background",
                                "linear-gradient(130deg, rgba(20,24,44,0.9), rgba(255,255,255,0.04))",
                            )
                    }
                    transition=Transition::spring_with(560, 0.35)
                >
                    <h2>"Pulse orb"</h2>
                    <p>"Tiny helper layer with MotionDiv + style composition."</p>
                    <MotionDiv
                        class="orb one"
                        initial=MotionStyle::new().opacity(0.0).scale(0.8)
                        animate=pulse_style
                        transition=Transition::spring_with(780, 0.8)
                    ></MotionDiv>
                </MotionDiv>
            </section>

            <footer class="footer">
                <span>"Motion example crate · leptos_fluid_motion_example"</span>
                <span>"Tweak transitions and styles to feel the API."</span>
            </footer>
        </main>
    }
}
