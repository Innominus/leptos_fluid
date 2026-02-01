use leptos::prelude::*;
use leptos_fluid::motion::{FluidPresence, FluidStyle, FluidSwap, PresenceMode, Transition};

const PRESENCE_MESSAGES: [(&str, &str, &str); 2] = [
    ("Live session", "Studio mic engaged", "REC"),
    ("Focus mode", "Notifications muted", "FOCUS"),
];

fn presence_toast(
    title: &'static str,
    subtitle: &'static str,
    pill: &'static str,
) -> impl IntoView {
    view! {
        <div class="presence-toast">
            <div class="presence-dot"></div>
            <div class="presence-copy">
                <p class="presence-title">{title}</p>
                <p class="presence-sub">{subtitle}</p>
            </div>
            <span class="presence-pill">{pill}</span>
        </div>
    }
}

#[component]
pub fn PresenceSection() -> impl IntoView {
    let open = RwSignal::new(true);
    let initial = FluidStyle::new()
        .opacity(0.0)
        .y(18.0)
        .scale(0.96)
        .with("filter", "blur(8px)");
    let animate = FluidStyle::new()
        .opacity(1.0)
        .y(0.0)
        .scale(1.0)
        .with("filter", "blur(0px)");
    let exit = FluidStyle::new()
        .opacity(0.0)
        .y(-14.0)
        .scale(0.98)
        .with("filter", "blur(6px)");
    let transition = Transition::spring_with(560, 0.32);
    let (title, subtitle, pill) = PRESENCE_MESSAGES[0];

    view! {
        <section class="presence-demo">
            <div class="panel">
                <h2>"Animate presence"</h2>
                <p>
                    "Mount and unmount with intent. FluidPresence keeps the element alive just long"
                    "enough to finish its exit transition."
                </p>
                <div class="list">
                    <span class="chip">"enter + exit"</span>
                    <span class="chip">"no flicker"</span>
                    <span class="chip">"lifecycle aware"</span>
                </div>
                <div class="button-row">
                    <button on:click=move |_| {
                        open.update(|value| *value = !*value)
                    }>{move || if open.get() { "Dismiss" } else { "Bring back" }}</button>
                </div>
            </div>

            <div class="presence-card">
                <div class="presence-stage">
                    <div class="presence-orb"></div>
                    <FluidPresence
                        show=open
                        initial=initial
                        animate=animate
                        exit=exit
                        transition=transition
                    >
                        {presence_toast(title, subtitle, pill)}
                    </FluidPresence>
                </div>
                <p class="presence-caption">
                    "Presence keeps the element mounted through its exit transition."
                </p>
            </div>
        </section>
    }
}

#[component]
pub fn PresenceSwapSection() -> impl IntoView {
    let swap = RwSignal::new(true);
    let mode = RwSignal::new(PresenceMode::Wait);
    let swap_initial = FluidStyle::new().opacity(0.0).y(10.0).scale(0.98);
    let swap_animate = FluidStyle::new().opacity(1.0).y(0.0).scale(1.0);
    let swap_exit = FluidStyle::new().opacity(0.0).y(-10.0).scale(0.98);
    let swap_transition = Transition::spring_with(520, 0.36);
    let sync_initial = FluidStyle::new().opacity(0.0).scale(0.99);
    let sync_exit = FluidStyle::new().opacity(0.0).scale(0.99);
    let sync_transition = Transition::spring_with(440, 0.3);
    let swap_animate_wait = swap_animate.clone();
    let swap_animate_sync = swap_animate.clone();

    view! {
        <section class="presence-demo">
            <div class="panel">
                <h2>"Fluid swap"</h2>
                <p>
                    "Swap between two states with either sequential (Wait) or overlapping (Sync)"
                    "presence."
                </p>
                <div class="list">
                    <span class="chip">"swap states"</span>
                    <span class="chip">"sync / wait"</span>
                    <div class="mode-row">
                        <span class="chip">"mode"</span>
                        <button
                            class="chip chip-button"
                            class:active=move || matches!(mode.get(), PresenceMode::Wait)
                            on:click=move |_| mode.set(PresenceMode::Wait)
                        >
                            "Wait"
                        </button>
                        <button
                            class="chip chip-button"
                            class:active=move || matches!(mode.get(), PresenceMode::Sync)
                            on:click=move |_| mode.set(PresenceMode::Sync)
                        >
                            "Sync"
                        </button>
                    </div>
                </div>
                <div class="button-row">
                    <button class="alt" on:click=move |_| swap.update(|value| *value = !*value)>
                        "Swap status"
                    </button>
                </div>
            </div>

            <div class="presence-card">
                <div class="presence-stage">
                    <div class="presence-orb"></div>
                    <Show
                        when=move || matches!(mode.get(), PresenceMode::Wait)
                        fallback=move || {
                            view! {
                                <FluidSwap
                                    show=swap
                                    initial=sync_initial.clone()
                                    animate=swap_animate_sync.clone()
                                    exit=sync_exit.clone()
                                    transition=sync_transition.clone()
                                    mode=PresenceMode::Sync
                                    first=move || {
                                        let (title, subtitle, pill) = PRESENCE_MESSAGES[0];
                                        presence_toast(title, subtitle, pill)
                                    }
                                    second=move || {
                                        let (title, subtitle, pill) = PRESENCE_MESSAGES[1];
                                        presence_toast(title, subtitle, pill)
                                    }
                                />
                            }
                        }
                    >
                        <FluidSwap
                            show=swap
                            initial=swap_initial.clone()
                            animate=swap_animate_wait.clone()
                            exit=swap_exit.clone()
                            transition=swap_transition.clone()
                            mode=PresenceMode::Wait
                            first=move || {
                                let (title, subtitle, pill) = PRESENCE_MESSAGES[0];
                                presence_toast(title, subtitle, pill)
                            }
                            second=move || {
                                let (title, subtitle, pill) = PRESENCE_MESSAGES[1];
                                presence_toast(title, subtitle, pill)
                            }
                        />
                    </Show>
                </div>
                <p class="presence-caption">
                    "PresenceMode::Sync overlaps enter/exit; Wait queues the next mount."
                </p>
            </div>
        </section>
    }
}
