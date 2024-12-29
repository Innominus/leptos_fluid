use leptos::prelude::*;
use leptos_fluid::animators::fluid_outlet::{FluidOutlet, A};

#[component]
pub fn Overlay() -> impl IntoView {
    view! {
        <main class="flex flex-col w-full h-full bg-gray-50">
            <div>
                <A attr:class="btn" href="/">
                    "HOME"
                </A>
                <A attr:class="btn" href="/about">
                    "ABOUT"
                </A>
            </div>
            <FluidOutlet outro_class="fly-out-transition" />
        </main>
    }
}
