use leptos::prelude::*;
use leptos_fluid::motion::{MotionDiv, MotionStyle, Transition};

#[component]
pub fn Motion() -> impl IntoView {
    let expanded = RwSignal::new(false);
    let highlight = RwSignal::new(false);

    let card_style = move || {
        if expanded.get() {
            MotionStyle::new()
                .opacity(1.0)
                .y(0.0)
                .scale(1.0)
                .with("box-shadow", "0 24px 60px rgba(15, 23, 42, 0.18)")
        } else {
            MotionStyle::new()
                .opacity(0.6)
                .y(26.0)
                .scale(0.96)
                .with("box-shadow", "0 10px 30px rgba(15, 23, 42, 0.12)")
        }
    };

    let glow_style = move || {
        if highlight.get() {
            MotionStyle::new()
                .opacity(1.0)
                .scale(1.0)
                .with("filter", "blur(0px)")
        } else {
            MotionStyle::new()
                .opacity(0.0)
                .scale(0.94)
                .with("filter", "blur(16px)")
        }
    };

    view! {
        <section class="w-full min-h-screen bg-gradient-to-b via-white from-slate-50 to-slate-100">
            <div class="flex flex-col gap-10 py-12 px-6 mx-auto w-full max-w-5xl">
                <header class="flex flex-col gap-4">
                    <p class="text-xs font-semibold uppercase tracking-[0.3em] text-slate-500">
                        "Leptos Motion"
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
                                    "Toggle the card or accent glow to watch MotionStyle updates."
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
                            <MotionDiv
                                class="absolute inset-0 rounded-2xl bg-emerald-200/60"
                                initial=MotionStyle::new().opacity(0.0).scale(0.95)
                                animate=glow_style
                                transition=Transition::spring()
                            ></MotionDiv>

                            <MotionDiv
                                class="relative p-6 bg-white rounded-2xl border border-slate-200"
                                initial=MotionStyle::new().opacity(0.0).y(18.0).scale(0.97)
                                animate=card_style
                                transition=Transition::spring()
                                while_hover=MotionStyle::new().scale(1.02)
                                while_tap=MotionStyle::new().scale(0.98)
                            >
                                <div class="flex gap-4 items-center">
                                    <div class="w-12 h-12 text-white rounded-full bg-slate-900"></div>
                                    <div>
                                        <p class="text-lg font-semibold text-slate-900">
                                            "Motion Card"
                                        </p>
                                        <p class="text-sm text-slate-500">
                                            "Hover or tap to feel the micro-interactions."
                                        </p>
                                    </div>
                                </div>
                                <div class="grid gap-3 mt-6 text-sm text-slate-600">
                                    <p>"• initial → animate transitions"</p>
                                    <p>"• hover/tap variants"</p>
                                    <p>"• spring easing default"</p>
                                </div>
                            </MotionDiv>
                        </div>
                    </div>

                    <div class="p-8 rounded-3xl border shadow-xl border-slate-200 bg-white/90">
                        <h3 class="text-lg font-semibold text-slate-900">"Quick recipes"</h3>
                        <ul class="mt-4 space-y-4 text-sm text-slate-600">
                            <li>
                                <span class="font-semibold text-slate-900">"Slide in"</span>
                                " — MotionStyle::new().x(32.0).opacity(0.0)"
                            </li>
                            <li>
                                <span class="font-semibold text-slate-900">"Scale pop"</span>
                                " — MotionStyle::new().scale(0.94).opacity(0.0)"
                            </li>
                            <li>
                                <span class="font-semibold text-slate-900">"Spring"</span>
                                " — Transition::spring()"
                            </li>
                        </ul>
                    </div>
                </div>
            </div>
        </section>
    }
}
