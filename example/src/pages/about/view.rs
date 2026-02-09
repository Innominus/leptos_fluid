use leptos::prelude::*;

use crate::components::common::PageShell;

#[component]
pub fn About() -> impl IntoView {
    view! {
        // <AnimatedInlet intro_class="fly-in-transition">
        <PageShell>
            <div class="inline-block w-full h-full text-center text-white bg-rose-500">"About"</div>
        </PageShell>
    }
}
