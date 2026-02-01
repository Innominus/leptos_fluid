use leptos::prelude::*;
use leptos_fluid::motion::{FluidDiv, FluidStyle, Transition};

use crate::components::common::PageShell;

#[component]
pub fn Home() -> impl IntoView {
    let expanded = RwSignal::new(false);
    let animate_style = move || {
        if expanded.get() {
            FluidStyle::new().opacity(1.0).y(0.0).scale(1.0)
        } else {
            FluidStyle::new().opacity(0.6).y(24.0).scale(0.96)
        }
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
                    initial=FluidStyle::new().opacity(0.0).y(32.0).scale(0.94)
                    animate=animate_style
                    transition=Transition::spring()
                    while_hover=FluidStyle::new().scale(1.02)
                    while_tap=FluidStyle::new().scale(0.98)
                >
                    <div class="p-6 text-left">
                        <div class="text-lg font-semibold text-slate-900">
                            "Leptos Fluid Motion"
                        </div>
                        <p class="mt-2 text-sm text-slate-600">
                            "A tiny, WASM-friendly motion layer with hover/tap states and a spring transition."
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
