use leptos::prelude::*;
use leptos_fluid::animators::fluid_outlet::FluidManager;
use leptos_meta::*;
use leptos_router::{
    components::{ParentRoute, Route, Router, Routes},
    ParamSegment, StaticSegment,
};

use crate::{
    components::overlay::Overlay,
    pages::{about::view::About, home::view::Home},
};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    // TODO: Eventually move this into the first Outlet that gets spawned so it's seamless as an API
    provide_context(FluidManager::new());

    view! {
        <Router>
            <Routes fallback=|| "Page not found">
                <ParentRoute path=StaticSegment("/") view=Overlay>
                    <Route path=StaticSegment("") view=Home />
                    <Route path=StaticSegment("about") view=About />
                    <ParentRoute path=StaticSegment("new-route") view=Overlay>
                        <Route path=StaticSegment("") view=Home />
                        <Route path=StaticSegment("about") view=About />

                        <ParentRoute path=ParamSegment(":id") view=Overlay>
                            <Route path=StaticSegment("") view=Home />
                            <Route path=StaticSegment("about") view=About />
                        </ParentRoute>
                    </ParentRoute>
                </ParentRoute>
            </Routes>
        </Router>
    }
}
