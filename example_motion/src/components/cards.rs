use leptos::prelude::*;
use leptos_fluid::motion::{style, FluidDiv, FluidStyle, Transition};

#[component]
pub fn CardsSection(card_focus: RwSignal<bool>) -> impl IntoView {
    let focus_style = move || {
        if card_focus.get() {
            FluidStyle::new()
                .opacity(1.0)
                .scale(1.02)
                .with("border-color", "rgba(116, 241, 255, 0.8)")
        } else {
            FluidStyle::new()
                .opacity(0.9)
                .scale(1.0)
                .with("border-color", "rgba(255, 255, 255, 0.08)")
        }
    };

    view! {
        <section class="grid">
            <FluidDiv
                class="card"
                initial=FluidStyle::new().opacity(0.0).y(20.0)
                animate=focus_style
                transition=Transition::new().duration_ms(420).bounce(0.35)
            >
                <h3>"Reactive focus"</h3>
                <p>"Drive border/opacity from any signal or closure."</p>
            </FluidDiv>

            <FluidDiv
                class="card"
                initial=FluidStyle::new().opacity(0.0).y(24.0)
                animate=FluidStyle::new().opacity(1.0).y(0.0)
                transition=Transition::spring_with(520, 0.6)
                while_hover=FluidStyle::new().scale(1.03)
            >
                <h3>"Hover lift"</h3>
                <p>"Easing + hover scale for quick emphasis."</p>
            </FluidDiv>

            <FluidDiv
                class="card"
                initial=FluidStyle::new().opacity(0.0).x(-24.0)
                animate=FluidStyle::new().opacity(1.0).x(0.0)
                transition=Transition::new().duration_ms(520).bounce(0.2)
                while_tap=FluidStyle::new().scale(0.97)
            >
                <h3>"Slide & tap"</h3>
                <p>"Different combos of initial, animate, and tap variants."</p>
            </FluidDiv>

            <FluidDiv
                class="card"
                initial=FluidStyle::new().opacity(0.0).y(20.0)
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
                while_hover=FluidStyle::new().scale(1.03).rotate(-0.6)
            >
                <h3>"style! macro"</h3>
                <p>"Use the macro + builders together for rich styles."</p>
            </FluidDiv>
        </section>
    }
}
