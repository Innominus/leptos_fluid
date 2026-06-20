use leptos::prelude::*;

#[component]
pub fn HeroSection() -> impl IntoView {
    view! {
        <section class="hero">
            <p class="kicker">"Leptos Fluid Scroll"</p>
            <h1>"Scroll-driven motion, mapped to controllers and timelines."</h1>
            <p class="lead">
                "A CSR playground for leptos_fluid_scroll: scrubbed controllers, \
                 toggle-bound timelines, discrete step scrubbing, one-shot reveals, \
                 and pure-callback triggers. Scroll to see each integration mode."
            </p>
        </section>
    }
}