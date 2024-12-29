use leptos::prelude::*;
use leptos_fluid::animators::fluid_outlet::FluidManager;
use leptos_meta::*;
use leptos_router::{
    components::{ParentRoute, Route, Router, Routes},
    StaticSegment,
};

use crate::{
    components::overlay::Overlay,
    pages::{about::view::About, home::view::Home},
};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_context(FluidManager::new());

    view! {
        <Router>
            <Routes fallback=|| "Page not found">
                <ParentRoute path=StaticSegment("") view=Overlay>
                    <Route path=StaticSegment("") view=Home />
                    <Route path=StaticSegment("about") view=About />
                </ParentRoute>
            </Routes>
        </Router>
    }
}
