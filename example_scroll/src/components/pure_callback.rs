use leptos::prelude::*;
use leptos_fluid_scroll::prelude::*;

#[component]
pub fn PureCallbackSection() -> impl IntoView {
    let card_ref = NodeRef::<leptos::html::Div>::new();

    let progress_signal = RwSignal::new(0.0f64);
    let direction_signal = RwSignal::new(0i8);
    let is_active_signal = RwSignal::new(false);
    let velocity_signal = RwSignal::new(0.0f64);
    let enter_count = RwSignal::new(0u32);
    let leave_count = RwSignal::new(0u32);

    let enter_handle = enter_count;
    let leave_handle = leave_count;
    let progress_handle = progress_signal;
    let direction_handle = direction_signal;
    let active_handle = is_active_signal;
    let velocity_handle = velocity_signal;

    let trigger = ScrollTrigger::builder()
        .target(card_ref)
        .start("top center")
        .end("bottom center")
        .on_enter(move |_| {
            enter_handle.update(|v| *v += 1);
        })
        .on_leave(move |_| {
            leave_handle.update(|v| *v += 1);
        })
        .on_update(move |event| {
            progress_handle.set(event.progress);
            direction_handle.set(event.direction);
            active_handle.set(event.is_active);
            velocity_handle.set(event.velocity);
        })
        .install();

    let progress = trigger.progress();
    let is_active = trigger.is_active();
    let direction = trigger.direction();
    let velocity = trigger.velocity();

    view! {
        <section class="section">
            <div class="panel">
                <p class="kicker">"Pure-callback mode"</p>
                <h2>"Reactive readouts from callbacks"</h2>
                <p>
                    "ScrollTrigger with no motion binding: on_enter / on_leave / on_update \
                     callbacks update reactive signals. The trigger also exposes progress(), \
                     direction(), is_active(), and velocity() signals directly."
                </p>
                <div class="indicator">
                    <span class="badge" class:active=move || is_active.get()>
                        {move || if is_active.get() { "active" } else { "idle" }}
                    </span>
                    <span class="badge">
                        {move || format!("progress {:.2}", progress.get())}
                    </span>
                    <span class="badge">
                        {move || format!("dir {}", direction.get())}
                    </span>
                    <span class="badge">
                        {move || format!("vel {:.0}", velocity.get())}
                    </span>
                </div>
                <div class="indicator">
                    <span class="badge">{move || format!("enters {}", enter_count.get())}</span>
                    <span class="badge">{move || format!("leaves {}", leave_count.get())}</span>
                    <span class="badge">
                        {move || format!("cb-progress {:.2}", progress_signal.get())}
                    </span>
                    <span class="badge">
                        {move || format!("cb-dir {}", direction_signal.get())}
                    </span>
                    <span class="badge">
                        {move || format!("cb-active {}", is_active_signal.get())}
                    </span>
                    <span class="badge">
                        {move || format!("cb-vel {:.0}", velocity_signal.get())}
                    </span>
                </div>
                <div class="progress-track">
                    <div
                        class="progress-fill"
                        style:width=move || format!("{}%", (progress.get() * 100.0).round())
                    ></div>
                </div>
            </div>

            <div class="card card-callback" node_ref=card_ref>
                <p class="chip">"callback"</p>
                <h3>"No motion dep required"</h3>
                <p>"The trigger drives signals directly; callbacks mirror the same data."</p>
            </div>
        </section>
    }
}