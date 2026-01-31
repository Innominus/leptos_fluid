use leptos::prelude::*;
use leptos_fluid::motion::{AnimatePresence, MotionStyle, Transition};

#[component]
pub fn PresenceSection() -> impl IntoView {
    let open = RwSignal::new(true);
    let variant = RwSignal::new(0usize);
    let messages = [
        ("Live session", "Studio mic engaged", "REC"),
        ("Focus mode", "Notifications muted", "FOCUS"),
        ("Sync complete", "Assets pushed to cloud", "SYNC"),
    ];

    let initial = MotionStyle::new()
        .opacity(0.0)
        .y(18.0)
        .scale(0.96)
        .with("filter", "blur(8px)");
    let animate = MotionStyle::new()
        .opacity(1.0)
        .y(0.0)
        .scale(1.0)
        .with("filter", "blur(0px)");
    let exit = MotionStyle::new()
        .opacity(0.0)
        .y(-14.0)
        .scale(0.98)
        .with("filter", "blur(6px)");
    let transition = Transition::spring_with(560, 0.32);

    view! {
        <section class="presence-demo">
            <div class="panel">
                <h2>"Animate presence"</h2>
                <p>
                    "Mount and unmount with intent. AnimatePresence keeps the element alive just long"
                    "enough to finish its exit transition."
                </p>
                <div class="list">
                    <span class="chip">"enter + exit"</span>
                    <span class="chip">"no flicker"</span>
                    <span class="chip">"lifecycle aware"</span>
                </div>
                <div class="button-row">
                    <button on:click=move |_| open.update(|value| *value = !*value)>
                        {move || if open.get() { "Dismiss" } else { "Bring back" }}
                    </button>
                    <button
                        class="alt"
                        on:click=move |_| variant.update(|value| *value = (*value + 1) % messages.len())
                    >
                        "Swap status"
                    </button>
                </div>
            </div>

            <div class="presence-card">
                <div class="presence-stage">
                    <div class="presence-orb"></div>
                    <AnimatePresence
                        show=open
                        initial=initial
                        animate=animate
                        exit=exit
                        transition=transition
                    >
                        <div class="presence-toast">
                            <div class="presence-dot"></div>
                            <div class="presence-copy">
                                <p class="presence-title">{move || messages[variant.get()].0}</p>
                                <p class="presence-sub">{move || messages[variant.get()].1}</p>
                            </div>
                            <span class="presence-pill">{move || messages[variant.get()].2}</span>
                        </div>
                    </AnimatePresence>
                </div>
                <p class="presence-caption">
                    "Exit styles apply first, then AnimatePresence waits the transition duration before unmounting."
                </p>
            </div>
        </section>
    }
}
