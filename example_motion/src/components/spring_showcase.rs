use leptos::prelude::*;
use leptos_fluid_motion::{FluidDiv, FluidStyle, Transition};

#[component]
pub fn SpringShowcaseSection() -> impl IntoView {
    let expanded = RwSignal::new(false);
    let roomy = RwSignal::new(false);
    let lane = RwSignal::new(1usize);

    view! {
        <section class="section-grid">
            <div class="panel">
                <p class="section-kicker">"Spring primitives"</p>
                <h2>"Dedicated examples for the new spring APIs"</h2>
                <p>
                    "These demos keep spring motion scoped to supported properties only: translate, scale, rotate, opacity, width, and height. The surface styling stays static so the motion reads cleanly."
                </p>
            </div>

            <div class="spring-showcase-grid">
                <div class="spring-demo-card">
                    <div class="spring-demo-copy">
                        <p class="chip">"Transition::spring()"</p>
                        <h3>"Visible overshoot with the default API"</h3>
                        <p>
                            "This one intentionally exaggerates the travel so you can see the new spring runtime settle instead of reading it as a plain tween."
                        </p>
                    </div>
                    <div class="button-row">
                        <button on:click=move |_| expanded.update(|value| *value = !*value)>
                            {move || if expanded.get() { "Dock card" } else { "Lift card" }}
                        </button>
                    </div>
                    <div class="spring-demo-stage spring-demo-stage-center">
                        <FluidDiv
                            class="spring-surface spring-surface-calm"
                            initial=calm_surface_style(false)
                            animate=move || calm_surface_style(expanded.get())
                            transition=Transition::spring().duration_ms(520).bounce(0.42)
                        >
                            <p class="chip">"calm"</p>
                            <h4>"Default API, tuned to bounce"</h4>
                            <p>"Large travel plus a stronger bounce makes the spring shape obvious."</p>
                        </FluidDiv>
                    </div>
                </div>

                <div class="spring-demo-card">
                    <div class="spring-demo-copy">
                        <p class="chip">"spring_with"</p>
                        <h3>"Width + height with a lively settle"</h3>
                        <p>
                            "This pushes width and height harder than a production UI would, purely to make the spring character easy to read."
                        </p>
                    </div>
                    <div class="button-row">
                        <button class="alt" on:click=move |_| roomy.update(|value| *value = !*value)>
                            {move || if roomy.get() { "Compact shell" } else { "Expand shell" }}
                        </button>
                    </div>
                    <div class="spring-demo-stage spring-demo-stage-center">
                        <FluidDiv
                            class="spring-surface spring-surface-size"
                            initial=size_surface_style(false)
                            animate=move || size_surface_style(roomy.get())
                            transition=Transition::spring_with(560, 0.44)
                        >
                            <p class="chip">"size"</p>
                            <h4>"Springing explicit size props"</h4>
                            <p>{move || if roomy.get() { "Expanded to show a larger state." } else { "Compact and ready." }}</p>
                        </FluidDiv>
                    </div>
                </div>

                <div class="spring-demo-card spring-demo-card-wide">
                    <div class="spring-demo-copy">
                        <p class="chip">"retarget"</p>
                        <h3>"Live retargeting with visible momentum"</h3>
                        <p>
                            "Rapidly switching lanes should keep the puck moving with momentum instead of flattening into a sequence of restarts."
                        </p>
                    </div>
                    <div class="button-row segmented-row">
                        <button class:alt=move || lane.get() != 0 on:click=move |_| lane.set(0)>
                            "Left"
                        </button>
                        <button class:alt=move || lane.get() != 1 on:click=move |_| lane.set(1)>
                            "Center"
                        </button>
                        <button class:alt=move || lane.get() != 2 on:click=move |_| lane.set(2)>
                            "Right"
                        </button>
                    </div>
                    <div class="spring-demo-stage spring-demo-stage-lanes">
                        <div class="spring-lane-markers">
                            <span></span>
                            <span></span>
                            <span></span>
                        </div>
                        <FluidDiv
                            class="spring-puck"
                            initial=lane_puck_style(1)
                            animate=move || lane_puck_style(lane.get())
                            transition=Transition::spring_with(540, 0.52)
                        >
                            <span>{move || match lane.get() { 0 => "L", 1 => "C", _ => "R" }}</span>
                        </FluidDiv>
                    </div>
                </div>
            </div>
        </section>
    }
}

fn calm_surface_style(expanded: bool) -> FluidStyle {
    if expanded {
        FluidStyle::new()
            .opacity(1.0)
            .y(-28.0)
            .scale(1.04)
            .rotate(-4.0)
    } else {
        FluidStyle::new()
            .opacity(0.74)
            .y(22.0)
            .scale(0.93)
            .rotate(3.2)
    }
}

fn size_surface_style(roomy: bool) -> FluidStyle {
    if roomy {
        FluidStyle::new()
            .opacity(1.0)
            .width(360.0)
            .height(176.0)
            .scale(1.04)
            .y(-12.0)
    } else {
        FluidStyle::new()
            .opacity(0.82)
            .width(184.0)
            .height(88.0)
            .scale(0.92)
            .y(10.0)
    }
}

fn lane_puck_style(lane: usize) -> FluidStyle {
    match lane {
        0 => FluidStyle::new()
            .x(-118.0)
            .y(12.0)
            .scale(0.9)
            .rotate(-10.0)
            .opacity(0.68),
        1 => FluidStyle::new()
            .x(0.0)
            .y(-14.0)
            .scale(1.06)
            .rotate(0.0)
            .opacity(1.0),
        _ => FluidStyle::new()
            .x(118.0)
            .y(12.0)
            .scale(0.9)
            .rotate(10.0)
            .opacity(0.68),
    }
}
