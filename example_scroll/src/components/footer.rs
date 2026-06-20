use leptos::prelude::*;

#[component]
pub fn FooterSection() -> impl IntoView {
    view! {
        <section class="footer-panel">
            <p class="kicker">"Leptos Fluid Scroll"</p>
            <h2>"Phase 7 demo app"</h2>
            <p>
                "Built with leptos_fluid_scroll and leptos_fluid_motion. \
                 Source and docs live on GitHub."
            </p>
            <p>
                <a href="https://github.com/Innominus/leptos_fluid">
                    "github.com/Innominus/leptos_fluid"
                </a>
            </p>
        </section>
    }
}