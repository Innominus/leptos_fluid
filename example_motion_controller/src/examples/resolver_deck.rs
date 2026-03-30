use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_fluid_motion::{controller, when, Easing, FluidStyle, Transition};
use web_sys::Element;

const DECK_TITLES: [&str; 3] = ["Routing", "Resolver", "Delivery"];

#[component]
pub fn ResolverDeckExample() -> impl IntoView {
    let active_index = RwSignal::new(0usize);
    let energized = RwSignal::new(false);
    let first_ref = NodeRef::<leptos::html::Div>::new();
    let second_ref = NodeRef::<leptos::html::Div>::new();
    let third_ref = NodeRef::<leptos::html::Div>::new();

    let controller = controller! {
        resolver: {
            let first_ref = first_ref.clone();
            let second_ref = second_ref.clone();
            let third_ref = third_ref.clone();
            move || resolve_active_card(active_index.get_untracked(), &first_ref, &second_ref, &third_ref)
        },
        transition: Transition::new().duration_ms(240).easing(Easing::EaseInOut),
        initial: resolver_card_style(false),
    };
    let seeded = StoredValue::new(false);

    Effect::new(move || {
        if seeded.get_value() || first_ref.get().is_none() {
            return;
        }
        seeded.set_value(true);
        controller.set_immediate(resolver_card_style(energized.get_untracked()));
    });

    when! {
        controller: controller,
        on(energized.get()) {
            true => animate(resolver_card_style(true)),
            false => animate(resolver_card_style(false)),
        },
        on(active_index.get()) {
            _ => animate(resolver_card_style(energized.get())),
        },
    }

    view! {
        <article class="demo-panel panel-wide" data-testid="resolver-panel">
            <div class="panel-header">
                <p class="panel-eyebrow">"Dynamic target"</p>
                <h2>"resolver: move the controller between live nodes"</h2>
                <p>
                    "The controller reattaches to whichever card the resolver points at next."
                </p>
            </div>

            <div class="button-row">
                <button
                    data-testid="resolver-next"
                    on:click=move |_| active_index.update(|index| *index = (*index + 1) % DECK_TITLES.len())
                >
                    "Move focus"
                </button>
                <button
                    class="ghost"
                    data-testid="resolver-pulse"
                    on:click=move |_| energized.update(|value| *value = !*value)
                >
                    {move || if energized.get() { "Return to base" } else { "Pulse active card" }}
                </button>
            </div>

            <div class="stage">
                <div class="resolver-grid">
                    <ResolverCard index=0 active_index node_ref=first_ref title=DECK_TITLES[0] />
                    <ResolverCard index=1 active_index node_ref=second_ref title=DECK_TITLES[1] />
                    <ResolverCard index=2 active_index node_ref=third_ref title=DECK_TITLES[2] />
                </div>
            </div>
        </article>
    }
}

#[component]
fn ResolverCard(
    index: usize,
    active_index: RwSignal<usize>,
    node_ref: NodeRef<leptos::html::Div>,
    title: &'static str,
) -> impl IntoView {
    view! {
        <div
            node_ref=node_ref
            class="resolver-card"
            class:active=move || active_index.get() == index
            data-testid=resolver_test_id(index)
        >
            <p class="chip">{resolver_chip(index)}</p>
            <h3>{title}</h3>
            <p>{resolver_body(index)}</p>
        </div>
    }
}

fn resolver_body(index: usize) -> &'static str {
    match index {
        0 => "A resolver lets state choose which real element receives the next command.",
        1 => "The newly active node inherits the current intent.",
        _ => "The previous target freezes until it becomes active again.",
    }
}

fn resolver_test_id(index: usize) -> &'static str {
    match index {
        0 => "resolver-card-0",
        1 => "resolver-card-1",
        _ => "resolver-card-2",
    }
}

fn resolver_chip(index: usize) -> &'static str {
    match index {
        0 => "01",
        1 => "02",
        _ => "03",
    }
}

fn resolve_active_card(
    active_index: usize,
    first_ref: &NodeRef<leptos::html::Div>,
    second_ref: &NodeRef<leptos::html::Div>,
    third_ref: &NodeRef<leptos::html::Div>,
) -> Option<Element> {
    match active_index {
        0 => first_ref.get_untracked().map(|node| node.unchecked_into()),
        1 => second_ref.get_untracked().map(|node| node.unchecked_into()),
        _ => third_ref.get_untracked().map(|node| node.unchecked_into()),
    }
}

fn resolver_card_style(energized: bool) -> FluidStyle {
    if energized {
        FluidStyle::new()
            .opacity(1.0)
            .y(-12.0)
            .scale(1.03)
            .rotate(-1.2)
            .with("background", "#0f766e")
            .with("color", "#ecfeff")
            .with("border-color", "rgba(153,246,228,.5)")
            .with("box-shadow", "0 26px 48px rgba(15,118,110,.24)")
    } else {
        FluidStyle::new()
            .opacity(0.76)
            .y(0.0)
            .scale(0.96)
            .rotate(0.0)
            .with("background", "#e2e8f0")
            .with("color", "#0f172a")
            .with("border-color", "rgba(15,23,42,.12)")
            .with("box-shadow", "0 14px 28px rgba(15,23,42,.12)")
    }
}
