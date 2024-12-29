use leptos::{logging::log, prelude::*};
use leptos_fluid::animators::fluid_outlet::AnimatedInlet;

use crate::components::common::PageShell;

#[component]
pub fn Home() -> impl IntoView {
    log!("Home Navigated");
    view! {
        <AnimatedInlet intro_class="fly-in-transition">
            <PageShell>
                <div class="inline-block w-full h-full text-center text-white bg-teal-500">
                    "Home"
                </div>
            </PageShell>
        </AnimatedInlet>
    }
}
