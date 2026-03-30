use leptos::prelude::*;
use leptos_fluid::motion::{FluidDiv, FluidStyle, Transition};

#[component]
pub fn Motion() -> impl IntoView {
    let expanded = RwSignal::new(false);
    let highlight = RwSignal::new(false);

    let card_style = move || {
        if expanded.get() {
            FluidStyle::new().opacity(1.0).y(0.0).scale(1.0)
        } else {
            FluidStyle::new().opacity(0.72).y(18.0).scale(0.98)
        }
    };

    let glow_style = move || {
        if highlight.get() {
            FluidStyle::new().opacity(1.0).scale(1.0)
        } else {
            FluidStyle::new().opacity(0.0).scale(0.96)
        }
    };

    view! {
        <section class="w-full min-h-screen bg-gradient-to-b via-white from-slate-50 to-slate-100">
            <div class="flex flex-col gap-10 py-12 px-6 mx-auto w-full max-w-5xl">
                <header class="flex flex-col gap-4">
                    <p class="text-xs font-semibold uppercase tracking-[0.3em] text-slate-500">
                        "Leptos Fluid Motion"
                    </p>
                    <h1 class="text-4xl font-semibold text-slate-900">
                        "A focused motion playground"
                    </h1>
                    <p class="max-w-2xl text-sm text-slate-600">
                        "This page isolates the motion components so you can see how the API feels and how the animations behave in a real layout."
                    </p>
                </header>

                <div class="grid gap-8 md:grid-cols-[1.2fr_0.8fr]">
                    <div class="p-8 rounded-3xl border shadow-xl border-slate-200 bg-white/90">
                        <div class="flex gap-4 justify-between items-center">
                            <div>
                                <h2 class="text-xl font-semibold text-slate-900">
                                    "State-driven motion"
                                </h2>
                                <p class="mt-2 text-sm text-slate-500">
                                    "Toggle the card or accent glow to watch FluidStyle updates."
                                </p>
                            </div>
                            <div class="flex gap-2 items-center">
                                <button
                                    class="btn btn-sm"
                                    on:click=move |_| expanded.update(|val| *val = !*val)
                                >
                                    {move || if expanded.get() { "Collapse" } else { "Expand" }}
                                </button>
                                <button
                                    class="btn btn-outline btn-sm"
                                    on:click=move |_| highlight.update(|val| *val = !*val)
                                >
                                    {move || if highlight.get() { "Dim" } else { "Glow" }}
                                </button>
                            </div>
                        </div>

                        <div class="relative mt-8">
                            <FluidDiv
                                class="absolute inset-0 rounded-2xl bg-emerald-200/60 blur-2xl"
                                initial=FluidStyle::new().opacity(0.0).scale(0.96)
                                animate=glow_style
                                transition=Transition::spring_with(420, 0.1)
                            ></FluidDiv>

                            <FluidDiv
                                class="relative p-6 bg-white rounded-2xl border border-slate-200 shadow-xl"
                                initial=FluidStyle::new().opacity(0.72).y(18.0).scale(0.98)
                                animate=card_style
                                transition=Transition::spring_with(440, 0.12)
                                while_hover=FluidStyle::new().scale(1.02)
                                while_tap=FluidStyle::new().scale(0.98)
                            >
                                <div class="flex gap-4 items-center">
                                    <div class="w-12 h-12 text-white rounded-full bg-slate-900"></div>
                                    <div>
                                        <p class="text-lg font-semibold text-slate-900">
                                            "Fluid Card"
                                        </p>
                                        <p class="text-sm text-slate-500">
                                            "Hover or tap to feel the micro-interactions."
                                        </p>
                                    </div>
                                </div>
                                <div class="grid gap-3 mt-6 text-sm text-slate-600">
                                    <p>"• initial → animate transitions"</p>
                                    <p>"• hover/tap variants"</p>
                                    <p>"• live spring transitions via rAF"</p>
                                </div>
                            </FluidDiv>
                        </div>
                    </div>

                    <div class="p-8 rounded-3xl border shadow-xl border-slate-200 bg-white/90">
                        <h3 class="text-lg font-semibold text-slate-900">"Quick recipes"</h3>
                        <ul class="mt-4 space-y-4 text-sm text-slate-600">
                            <li>
                                <span class="font-semibold text-slate-900">"Slide in"</span>
                                " — FluidStyle::new().x(32.0).opacity(0.0)"
                            </li>
                            <li>
                                <span class="font-semibold text-slate-900">"Scale pop"</span>
                                " — FluidStyle::new().scale(0.94).opacity(0.0)"
                            </li>
                            <li>
                                <span class="font-semibold text-slate-900">"Standard UI"</span>
                                " — Transition::new()"
                            </li>
                            <li>
                                <span class="font-semibold text-slate-900">"Live spring"</span>
                                " — Transition::spring_with(...)"
                            </li>
                        </ul>
                    </div>
                </div>
            </div>
        </section>
    }
}
