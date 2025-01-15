use leptos::{logging::log, prelude::*};

use crate::components::common::PageShell;

#[component]
pub fn Home() -> impl IntoView {
    view! {
        // <AnimatedInlet intro_class="fly-in-transition">
        <PageShell>
            <div class="inline-block w-full h-full text-center text-white bg-teal-500">"Home"</div>
        </PageShell>
    }
}
