use leptos::prelude::*;
use leptos_fluid::motion::{
    bind_interaction_node_ref, AnimationController, FluidStyle, Transition,
};

use crate::components::common::PageShell;

#[component]
pub fn Home() -> impl IntoView {
    let expanded = RwSignal::new(false);

    let card_ref = NodeRef::<leptos::html::Div>::new();

    let animate_style = move || {
        if expanded.get() {
            FluidStyle::new().opacity(1.0).y(0.0).scale(1.0)
        } else {
            FluidStyle::new().opacity(0.72).y(18.0).scale(0.98)
        }
    };

    let controller = AnimationController::builder()
        .target(card_ref)
        .transition(Transition::spring_with(420, 0.12))
        .initial(FluidStyle::new().opacity(0.72).y(18.0).scale(0.98))
        .animate(animate_style)
        .install();

    bind_interaction_node_ref(
        controller,
        card_ref,
        animate_style,
        Some(FluidStyle::new().scale(1.02)),
        Some(FluidStyle::new().scale(0.98)),
    );

    view! {
        <PageShell>
            <div class="flex flex-col gap-4 items-center py-8 w-full">
                <div class="text-xl font-semibold text-slate-800">"Fluid"</div>
                <button class="btn btn-sm" on:click=move |_| expanded.update(|val| *val = !*val)>
                    {move || if expanded.get() { "Collapse" } else { "Expand" }}
                </button>
                <div
                    class="w-full max-w-lg bg-white rounded-xl border shadow-lg border-slate-200"
                    node_ref=card_ref
                >
                    <div class="p-6 text-left">
                        <div class="text-lg font-semibold text-slate-900">
                            "Leptos Fluid Motion"
                        </div>
                        <p class="mt-2 text-sm text-slate-600">
                            "A tiny, WASM-friendly motion layer with hover/tap states and a live rAF spring transition."
                        </p>
                    </div>
                </div>
            </div>
            <div
                style="min-height:600px;"
                class="inline-block mt-8 w-full text-center text-white bg-teal-500"
            >
                "Home"
            </div>
        </PageShell>
    }
}
