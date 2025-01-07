use leptos::prelude::*;
use leptos_fluid::animators::fluid_outlet::FluidOutlet;
use leptos_router::components::A;

#[component]
pub fn Overlay() -> impl IntoView {
    view! {
        <main class="flex flex-col w-full h-full bg-gray-50">
            <div>
                <A attr:class="btn btn-sm text-xs" href="/">
                    "TOP OUTLET"
                </A>
                <A attr:class="btn btn-sm text-xs" href="/about">
                    "TOP OUTLET ABOUT"
                </A>
                <A attr:class="btn btn-sm text-xs" href="/new-route/">
                    "MIDDLE OUTLET"
                </A>
                <A attr:class="btn btn-sm text-xs" href="/new-route/about">
                    "MIDDLE OUTLET ABOUT"
                </A>
                <A attr:class="btn btn-sm text-xs" href="/new-route/32/">
                    "DEEPEST OUTLET - ROUTE PARAM 32"
                </A>
                <A attr:class="btn btn-sm text-xs" href="/new-route/46/about">
                    "DEEPEST OUTLET - ROUTE PARAM 46"
                </A>
            </div>
            <FluidOutlet intro_class="fly-in-transition" outro_class="fly-out-transition" />
        </main>
    }
}
