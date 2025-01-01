use leptos::prelude::*;
use leptos_fluid::animators::fluid_outlet::FluidOutlet;
use leptos_router::components::A;

#[component]
pub fn Overlay() -> impl IntoView {
    view! {
        <main class="flex flex-col w-full h-full bg-gray-50">
            <div>
                <A attr:class="btn" href="/">
                    "HOME TOP OUTLET"
                </A>
                <A attr:class="btn" href="/about">
                    "ABOUT TOP OUTLET"
                </A>
                <A attr:class="btn" href="/new-route/">
                    "HOME"
                </A>
                <A attr:class="btn" href="/new-route/about">
                    "ABOUT"
                </A>
                <A attr:class="btn" href="/new-route/32/">
                    "HOME 32"
                </A>
                <A attr:class="btn" href="/new-route/46/about">
                    "ABOUT 46"
                </A>
            </div>
            <FluidOutlet intro_class="fly-in-transition" outro_class="fly-out-transition" />
        </main>
    }
}
