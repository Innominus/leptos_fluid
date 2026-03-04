use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_fluid::motion::{AnimationController, FluidStyle, Transition};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <main class="page">
            <header class="hero" data-testid="controller-hero">
                <p class="eyebrow">"Leptos Fluid Motion"</p>
                <h1>"AnimationController-only playground"</h1>
                <p class="lead">
                    "This app uses plain elements + NodeRef and drives every motion path through AnimationController."
                </p>
            </header>

            <section class="grid">
                <ToggleCardExample />
                <TabsUnderlineExample />
                <PointerStateExample />
                <QueueLatestExample />
            </section>
        </main>
    }
}

#[component]
fn TabsUnderlineExample() -> impl IntoView {
    let active_tab = RwSignal::new(0usize);
    let hovered_tab = RwSignal::new(None::<usize>);
    let tabs_ref = NodeRef::<leptos::html::Div>::new();
    let underline_ref = NodeRef::<leptos::html::Div>::new();
    let tab_refs = (0..4)
        .map(|_| NodeRef::<leptos::html::Button>::new())
        .collect::<Vec<_>>();
    let tab_refs_effect = tab_refs.clone();
    let initialized = StoredValue::new(false);

    let controller = AnimationController::with_transition(Transition::spring_with(520, 0.35));
    controller.attach_resolver({
        let underline_ref = underline_ref.clone();
        move || {
            underline_ref
                .get_untracked()
                .map(|node| node.unchecked_into())
        }
    });

    Effect::new({
        let tabs_ref = tabs_ref.clone();
        let tab_refs_effect = tab_refs_effect.clone();
        move || {
            let index = hovered_tab.get().unwrap_or_else(|| active_tab.get());
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
                let target = FluidStyle::new()
                    .x(tab_rect.left() - container_rect.left())
                    .width(tab_rect.width())
                    .opacity(1.0);

                if initialized.get_value() {
                    controller.animate(target);
                } else {
                    controller.set_immediate(target);
                    initialized.set_value(true);
                }
            });
        }
    });

    view! {
        <article class="panel panel-wide" data-testid="controller-tabs-panel">
            <div class="panel-header">
                <h2>"Fluid tab underline"</h2>
                <p>
                    "The underline is a plain element driven by one controller. Rapid clicks retarget mid-flight without snapping."
                </p>
            </div>

            <div class="tabs-shell">
                <div
                    class="tabs-list"
                    node_ref=tabs_ref
                    data-testid="controller-tabs-list"
                    on:pointerleave=move |_| hovered_tab.set(None)
                >
                    <button
                        class="tabs-button"
                        class:active=move || active_tab.get() == 0
                        node_ref=tab_refs[0]
                        data-testid="controller-tab-button-0"
                        on:pointerenter=move |_| hovered_tab.set(Some(0))
                        on:click=move |_| active_tab.set(0)
                    >
                        "API"
                    </button>
                    <button
                        class="tabs-button"
                        class:active=move || active_tab.get() == 1
                        node_ref=tab_refs[1]
                        data-testid="controller-tab-button-1"
                        on:pointerenter=move |_| hovered_tab.set(Some(1))
                        on:click=move |_| active_tab.set(1)
                    >
                        "Workflow Studio"
                    </button>
                    <button
                        class="tabs-button"
                        class:active=move || active_tab.get() == 2
                        node_ref=tab_refs[2]
                        data-testid="controller-tab-button-2"
                        on:pointerenter=move |_| hovered_tab.set(Some(2))
                        on:click=move |_| active_tab.set(2)
                    >
                        "Retargeting Engine"
                    </button>
                    <button
                        class="tabs-button"
                        class:active=move || active_tab.get() == 3
                        node_ref=tab_refs[3]
                        data-testid="controller-tab-button-3"
                        on:pointerenter=move |_| hovered_tab.set(Some(3))
                        on:click=move |_| active_tab.set(3)
                    >
                        "Queue"
                    </button>

                    <div
                        node_ref=underline_ref
                        class="tabs-underline"
                        data-testid="controller-tab-underline"
                    ></div>
                </div>

                <div class="tabs-content" data-testid="controller-tab-content">
                    <h3>{move || tabs_title(active_tab.get())}</h3>
                    <p>{move || tabs_body(active_tab.get())}</p>
                </div>
            </div>
        </article>
    }
}

#[component]
fn ToggleCardExample() -> impl IntoView {
    let expanded = RwSignal::new(false);
    let node_ref = NodeRef::<leptos::html::Div>::new();
    let controller = AnimationController::with_transition(Transition::spring_with(560, 0.22));

    controller.attach_resolver({
        let node_ref = node_ref.clone();
        move || node_ref.get_untracked().map(|node| node.unchecked_into())
    });

    controller.bind(move || toggle_card_style(expanded.get()));

    view! {
        <article class="panel" data-testid="controller-bind-panel">
            <div class="panel-header">
                <h2>"Declarative bind"</h2>
                <p>
                    "Bind controller.animate() to app state and keep markup plain."
                </p>
            </div>

            <div class="button-row">
                <button data-testid="controller-bind-toggle" on:click=move |_| expanded.update(|value| *value = !*value)>
                    {move || if expanded.get() { "Collapse" } else { "Expand" }}
                </button>
                <button
                    class="ghost"
                    data-testid="controller-bind-reset"
                    on:click=move |_| controller.set_immediate(toggle_card_style(false))
                >
                    "Snap reset"
                </button>
            </div>

            <div class="stage">
                <div node_ref=node_ref class="motion-card" data-testid="controller-bind-card">
                    <p class="chip">"bind()"</p>
                    <h3>"Controller as the animation primitive"</h3>
                    <p>
                        "No FluidDiv or FluidElement wrappers; state chooses styles, controller executes them."
                    </p>
                </div>
            </div>
        </article>
    }
}

#[component]
fn PointerStateExample() -> impl IntoView {
    let armed = RwSignal::new(false);
    let hovered = RwSignal::new(false);
    let pressed = RwSignal::new(false);

    let node_ref = NodeRef::<leptos::html::Button>::new();
    let controller = AnimationController::with_transition(Transition::new().duration_ms(180));

    controller.attach_resolver({
        let node_ref = node_ref.clone();
        move || node_ref.get_untracked().map(|node| node.unchecked_into())
    });

    controller.set_immediate(pointer_base_style(false));

    Effect::new(move || {
        let armed_now = armed.get();
        if hovered.get_untracked() || pressed.get_untracked() {
            return;
        }
        controller.animate(pointer_base_style(armed_now));
    });

    let on_pointer_enter = move |_| {
        hovered.set(true);
        controller.animate_with(
            pointer_hover_style(armed.get_untracked()),
            Transition::new().duration_ms(140),
        );
    };

    let on_pointer_leave = move |_| {
        hovered.set(false);
        pressed.set(false);
        controller.animate(pointer_base_style(armed.get_untracked()));
    };

    let on_pointer_down = move |_| {
        pressed.set(true);
        controller.animate_with(
            pointer_press_style(armed.get_untracked()),
            Transition::new().duration_ms(90),
        );
    };

    let release = move || {
        move |_| {
            if !pressed.get_untracked() {
                return;
            }
            pressed.set(false);
            if hovered.get_untracked() {
                controller.animate_with(
                    pointer_hover_style(armed.get_untracked()),
                    Transition::new().duration_ms(140),
                );
            } else {
                controller.animate(pointer_base_style(armed.get_untracked()));
            }
        }
    };

    let on_pointer_up = release();
    let on_pointer_cancel = release();

    view! {
        <article class="panel" data-testid="controller-pointer-panel">
            <div class="panel-header">
                <h2>"Manual interaction states"</h2>
                <p>
                    "Your app logic handles hover/press/active. The controller only receives target styles."
                </p>
            </div>

            <div class="button-row">
                <button
                    class="ghost"
                    data-testid="controller-pointer-arm-toggle"
                    on:click=move |_| armed.update(|value| *value = !*value)
                >
                    {move || if armed.get() { "Disable active mode" } else { "Enable active mode" }}
                </button>
            </div>

            <div class="stage center">
                <button
                    node_ref=node_ref
                    class="control-pill"
                    data-testid="controller-pointer-pill"
                    on:pointerenter=on_pointer_enter
                    on:pointerleave=on_pointer_leave
                    on:pointerdown=on_pointer_down
                    on:pointerup=on_pointer_up
                    on:pointercancel=on_pointer_cancel
                >
                    {move || if armed.get() { "Armed control" } else { "Idle control" }}
                </button>
            </div>
        </article>
    }
}

#[component]
fn QueueLatestExample() -> impl IntoView {
    let mounted = RwSignal::new(false);
    let queued_step = RwSignal::new(0usize);
    let node_ref = NodeRef::<leptos::html::Div>::new();

    let controller = AnimationController::with_transition(Transition::spring_with(480, 0.28));

    controller.attach_resolver({
        let node_ref = node_ref.clone();
        move || node_ref.get_untracked().map(|node| node.unchecked_into())
    });

    Effect::new(move || {
        let step = queued_step.get();
        controller.animate(queue_style(step));
    });

    Effect::new(move || {
        if !mounted.get() {
            return;
        }
        let controller = controller;
        let step = queued_step.get_untracked();
        request_animation_frame(move || {
            controller.animate(queue_style(step));
        });
    });

    view! {
        <article class="panel" data-testid="controller-queue-panel">
            <div class="panel-header">
                <h2>"Queue latest while detached"</h2>
                <p>
                    "Issue commands before mount, then attach the element and watch the latest queued state apply."
                </p>
            </div>

            <div class="button-row">
                <button
                    data-testid="controller-queue-next"
                    on:click=move |_| queued_step.update(|step| *step = (*step + 1) % 4)
                >
                    "Queue next style"
                </button>
                <button
                    class="ghost"
                    data-testid="controller-queue-mount"
                    on:click=move |_| mounted.update(|value| *value = !*value)
                >
                    {move || if mounted.get() { "Unmount target" } else { "Mount target" }}
                </button>
            </div>

            <div class="stage">
                <Show
                    when=move || mounted.get()
                    fallback=move || {
                        view! {
                            <p class="detached-note" data-testid="controller-queue-detached">
                                "Detached: queue styles, then mount to replay latest."
                            </p>
                        }
                    }
                >
                    <div node_ref=node_ref class="queue-chip" data-testid="controller-queue-chip">
                        <p class="chip">"queued"</p>
                        <h3 data-testid="controller-queue-label">{move || queue_label(queued_step.get())}</h3>
                    </div>
                </Show>
            </div>
        </article>
    }
}

fn toggle_card_style(expanded: bool) -> FluidStyle {
    if expanded {
        FluidStyle::new()
            .opacity(1.0)
            .x(0.0)
            .y(0.0)
            .scale(1.0)
            .with("background", "linear-gradient(150deg, #0f766e, #155e75)")
            .with("border-color", "rgba(8, 145, 178, 0.65)")
            .with("box-shadow", "0 22px 50px rgba(8, 47, 73, 0.28)")
    } else {
        FluidStyle::new()
            .opacity(0.82)
            .x(-20.0)
            .y(12.0)
            .scale(0.94)
            .with("background", "linear-gradient(160deg, #e7f4f1, #d8ebf8)")
            .with("border-color", "rgba(14, 116, 144, 0.24)")
            .with("box-shadow", "0 10px 24px rgba(15, 23, 42, 0.14)")
    }
}

fn pointer_base_style(armed: bool) -> FluidStyle {
    if armed {
        FluidStyle::new()
            .scale(1.0)
            .with("background", "linear-gradient(135deg, #0f766e, #115e59)")
            .with("color", "#ecfeff")
            .with("box-shadow", "0 10px 20px rgba(8, 47, 73, 0.2)")
    } else {
        FluidStyle::new()
            .scale(1.0)
            .with("background", "linear-gradient(135deg, #f8fafc, #f1f5f9)")
            .with("color", "#0f172a")
            .with("box-shadow", "0 10px 20px rgba(15, 23, 42, 0.12)")
    }
}

fn pointer_hover_style(armed: bool) -> FluidStyle {
    let mut style = pointer_base_style(armed).scale(1.04);
    style = style.with("box-shadow", "0 16px 28px rgba(15, 23, 42, 0.22)");
    style
}

fn pointer_press_style(armed: bool) -> FluidStyle {
    let mut style = pointer_base_style(armed).scale(0.96).y(2.0);
    style = style.with("box-shadow", "0 8px 14px rgba(15, 23, 42, 0.18)");
    style
}

fn queue_style(step: usize) -> FluidStyle {
    match step % 4 {
        0 => FluidStyle::new()
            .opacity(0.76)
            .x(-36.0)
            .y(10.0)
            .rotate(-8.0)
            .with("background", "linear-gradient(145deg, #dbeafe, #c7d2fe)"),
        1 => FluidStyle::new()
            .opacity(1.0)
            .x(0.0)
            .y(0.0)
            .rotate(0.0)
            .scale(1.02)
            .with("background", "linear-gradient(145deg, #ccfbf1, #99f6e4)"),
        2 => FluidStyle::new()
            .opacity(0.96)
            .x(42.0)
            .y(-14.0)
            .rotate(9.0)
            .scale(0.98)
            .with("background", "linear-gradient(145deg, #fef3c7, #fde68a)"),
        _ => FluidStyle::new()
            .opacity(0.92)
            .x(14.0)
            .y(18.0)
            .rotate(-4.0)
            .scale(1.06)
            .with("background", "linear-gradient(145deg, #ffedd5, #fed7aa)"),
    }
}

fn queue_label(step: usize) -> &'static str {
    match step % 4 {
        0 => "Queued: cool start",
        1 => "Queued: anchor",
        2 => "Queued: drift",
        _ => "Queued: flare",
    }
}

fn tabs_title(index: usize) -> &'static str {
    match index {
        0 => "Controller as execution layer",
        1 => "App state chooses targets",
        2 => "Mid-flight retargeting",
        _ => "Queue latest semantics",
    }
}

fn tabs_body(index: usize) -> &'static str {
    match index {
        0 => "The underline is just a node_ref + animate() calls. The controller does not need component wrappers.",
        1 => "Your app owns the tab state and measurements, then forwards x/width targets as declarative commands.",
        2 => "Rapid tab clicks keep motion fluid because each new target starts from current visual progress.",
        _ => "If a target is detached, the latest command is replayed when the element is available again.",
    }
}
