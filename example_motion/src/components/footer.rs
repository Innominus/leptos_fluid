use leptos::prelude::*;

#[component]
pub fn FooterSection() -> impl IntoView {
    view! {
        <footer class="footer">
            <span>"Fluid motion example · leptos_fluid_motion_example"</span>
            <span>"Tweak transitions and styles to feel the API."</span>
        </footer>
    }
}
