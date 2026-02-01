use leptos::prelude::*;
use leptos_fluid::motion::{Easing, FluidButton, FluidDiv, FluidStyle, Transition};

#[component]
pub fn HeroSection(pulse: RwSignal<bool>, card_focus: RwSignal<bool>) -> impl IntoView {
    let hero_toggle = RwSignal::new(false);

    let hero_style = move || {
        if hero_toggle.get() {
            FluidStyle::new()
                .opacity(1.0)
                .x(0.0)
                .y(0.0)
                .scale(1.0)
                .rotate(-1.0)
                .with("box-shadow", "0 30px 80px rgba(6, 7, 18, 0.6)")
        } else {
            FluidStyle::new()
                .opacity(0.7)
                .x(-12.0)
                .y(12.0)
                .scale(0.96)
                .rotate(1.5)
                .with("box-shadow", "0 18px 50px rgba(6, 7, 18, 0.4)")
        }
    };

    view! {
        <section class="hero">
            <div class="panel">
                <span class="tag">"Leptos Fluid"</span>
                <h1>"Fluid motion playground that actually moves."</h1>
                <p>
                    "Each panel showcases a different combination of FluidStyle, transitions, and hover/tap variants. "
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
                        {move || { if card_focus.get() { "Unfocus cards" } else { "Focus cards" } }}
                    </button>
                </div>
            </div>

            <FluidDiv
                class="glass"
                initial=FluidStyle::new().opacity(0.0).y(30.0)
                animate=hero_style
                transition=Transition::spring_with(620, 0.45)
                while_hover=FluidStyle::new().scale(1.02)
                while_tap=FluidStyle::new().scale(0.98)
            >
                <div class="orb one"></div>
                <div class="orb two"></div>
                <h2>"Hero card"</h2>
                <p>
                    "Animated with a spring, rotated transforms, and dynamic shadows. Uses while_hover and while_tap for micro interactions."
                </p>
                <FluidButton
                    class="alt"
                    initial=FluidStyle::new().opacity(0.0).y(10.0)
                    animate=move || FluidStyle::new().opacity(1.0).y(0.0)
                    transition=Transition::new().duration_ms(360).easing(Easing::EaseOut)
                    while_hover=FluidStyle::new().scale(1.04)
                    while_tap=FluidStyle::new().scale(0.96)
                >
                    "FluidButton"
                </FluidButton>
            </FluidDiv>
        </section>
    }
}
