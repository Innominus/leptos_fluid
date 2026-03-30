use leptos::prelude::*;
use leptos_fluid::motion::{use_spring, FluidDiv, FluidStyle, Spring, Transition};

use crate::components::common::PageShell;
use crate::components::spring_utils::lerp;

#[component]
pub fn Home() -> impl IntoView {
    let expanded = RwSignal::new(false);
    let card_progress = use_spring(0.0, Spring::new(560, 0.32));

    Effect::new({
        let card_progress = card_progress.clone();
        move || card_progress.set(if expanded.get() { 1.0 } else { 0.0 })
    });

    let animate_style = move || {
        let progress = card_progress.get();
        FluidStyle::new()
            .opacity(lerp(0.6, 1.0, progress))
            .y(lerp(24.0, 0.0, progress))
            .scale(lerp(0.96, 1.0, progress))
    };

    view! {
        <PageShell>
            <div class="flex flex-col gap-4 items-center py-8 w-full">
                <div class="text-xl font-semibold text-slate-800">"Fluid"</div>
                <button class="btn btn-sm" on:click=move |_| expanded.update(|val| *val = !*val)>
                    {move || if expanded.get() { "Collapse" } else { "Expand" }}
                </button>
                <FluidDiv
                    class="w-full max-w-lg bg-white rounded-xl border shadow-lg border-slate-200"
                    initial=FluidStyle::new().opacity(0.6).y(24.0).scale(0.96)
                    animate=animate_style
                    transition=Transition::new().duration_ms(0)
                    while_hover=FluidStyle::new().scale(1.02)
                    while_tap=FluidStyle::new().scale(0.98)
                >
                    <div class="p-6 text-left">
                        <div class="text-lg font-semibold text-slate-900">
                            "Leptos Fluid Motion"
                        </div>
                        <p class="mt-2 text-sm text-slate-600">
                            "A tiny, WASM-friendly motion layer with hover/tap states and an rAF spring driving the main card pose."
                        </p>
                    </div>
                </FluidDiv>
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
