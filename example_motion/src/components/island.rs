use leptos::prelude::*;
use leptos_fluid::motion::{use_spring, FluidDiv, FluidStyle, FluidValue, Spring, Transition};

#[component]
pub fn IslandSection() -> impl IntoView {
    let island_open = RwSignal::new(false);
    let island_width = use_spring(170.0, Spring::new(620, 0.5));
    let island_height = use_spring(46.0, Spring::new(620, 0.5));
    let island_radius = use_spring(999.0, Spring::new(620, 0.55));
    let island_glow_opacity = use_spring(0.0, Spring::new(520, 0.45));
    let island_glow_scale = use_spring(0.92, Spring::new(520, 0.45));
    let island_content_opacity = use_spring(0.0, Spring::new(420, 0.35));
    let island_content_y = use_spring(14.0, Spring::new(420, 0.35));
    let island_content_scale = use_spring(0.96, Spring::new(420, 0.35));

    Effect::new({
        let island_width = island_width.clone();
        let island_height = island_height.clone();
        let island_radius = island_radius.clone();
        let island_glow_opacity = island_glow_opacity.clone();
        let island_glow_scale = island_glow_scale.clone();
        let island_content_opacity = island_content_opacity.clone();
        let island_content_y = island_content_y.clone();
        let island_content_scale = island_content_scale.clone();
        move || {
            if island_open.get() {
                island_width.set(360.0);
                island_height.set(140.0);
                island_radius.set(32.0);
                island_glow_opacity.set(0.85);
                island_glow_scale.set(1.0);
                island_content_opacity.set(1.0);
                island_content_y.set(0.0);
                island_content_scale.set(1.0);
            } else {
                island_width.set(170.0);
                island_height.set(46.0);
                island_radius.set(999.0);
                island_glow_opacity.set(0.0);
                island_glow_scale.set(0.92);
                island_content_opacity.set(0.0);
                island_content_y.set(14.0);
                island_content_scale.set(0.96);
            }
        }
    });

    let island_style = move || {
        let radius = island_radius.get();
        let background = if island_open.get() {
            "linear-gradient(140deg, rgba(15, 23, 42, 0.95), rgba(8, 12, 24, 0.92))"
        } else {
            "linear-gradient(140deg, rgba(9, 12, 22, 0.95), rgba(10, 14, 28, 0.9))"
        };
        let border_color = if island_open.get() {
            "rgba(116, 241, 255, 0.25)"
        } else {
            "rgba(255, 255, 255, 0.12)"
        };
        let shadow = if island_open.get() {
            "0 30px 80px rgba(3, 6, 16, 0.7)"
        } else {
            "0 18px 50px rgba(3, 6, 16, 0.6)"
        };
        FluidStyle::new()
            .width(island_width.get())
            .height(island_height.get())
            .opacity(1.0)
            .with_prop("border-radius", FluidValue::from(format!("{radius}px")))
            .with_prop("background", FluidValue::from(background))
            .with_prop("border-color", FluidValue::from(border_color))
            .with_prop("box-shadow", FluidValue::from(shadow))
    };

    let island_glow = move || {
        FluidStyle::new()
            .opacity(island_glow_opacity.get())
            .scale(island_glow_scale.get())
            .with_prop("filter", FluidValue::from("blur(22px)"))
    };

    let island_content = move || {
        FluidStyle::new()
            .opacity(island_content_opacity.get())
            .y(island_content_y.get())
            .scale(island_content_scale.get())
    };

    view! {
        <section class="island-shell">
            <div class="panel">
                <h2>"Dynamic island"</h2>
                <p>
                    "A nod to Emil Kowalski's iOS island recreations. This uses width/height, border-radius, opacity, "
                    "and spring bounce to morph the shape."
                </p>
                <div class="button-row">
                    <button class="alt" on:click=move |_| island_open.update(|val| *val = !*val)>
                        {move || {
                            if island_open.get() { "Collapse island" } else { "Expand island" }
                        }}
                    </button>
                </div>
            </div>

            <div class="island-wrap">
                <FluidDiv
                    class="island-glow"
                    initial=FluidStyle::new().opacity(0.0).scale(0.92)
                    animate=island_glow
                    transition=Transition::new().duration_ms(0)
                ></FluidDiv>

                <FluidDiv
                    class="island"
                    initial=FluidStyle::new().width(170.0).height(46.0).opacity(1.0)
                    animate=island_style
                    transition=Transition::new().duration_ms(0)
                    while_hover=FluidStyle::new().scale(1.02)
                >
                    <div class="island-inner">
                        <div class="island-pill">
                            <div class="island-dot"></div>
                            <div class="island-wave"></div>
                        </div>
                        <FluidDiv
                            class="island-content"
                            initial=FluidStyle::new().opacity(0.0).y(14.0).scale(0.96)
                            animate=island_content
                            transition=Transition::new().duration_ms(0)
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
                        </FluidDiv>
                    </div>
                </FluidDiv>
            </div>
        </section>
    }
}
