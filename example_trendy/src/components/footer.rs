use leptos::prelude::*;

#[component]
pub fn FooterSection() -> impl IntoView {
    view! {
        <footer class="footer">
            <div class="footer-inner">
                <p class="kicker">"Leptos Fluid"</p>
                <h3>"Built with leptos_fluid_scroll + leptos_fluid_motion"</h3>
                <p>"10 scroll-driven animation patterns — bold editorial style."</p>
            </div>
        </footer>
    }
}