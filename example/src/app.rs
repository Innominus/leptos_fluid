use leptos::prelude::*;
use leptos_fluid::view_transitions::{FluidManager, FluidRoutes};
use leptos_meta::*;
use leptos_router::{
    components::{ParentRoute, Route, Router},
    ParamSegment, StaticSegment,
};

use crate::{
    components::overlay::Overlay,
    pages::{about::view::About, home::view::Home, motion::view::Motion},
};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_context(FluidManager::new());

    view! {
        <Router>
            <FluidRoutes fallback=|| "Page not found">
                <ParentRoute path=StaticSegment("/") view=Overlay>
                    <Route path=StaticSegment("") view=Home />
                    <Route path=StaticSegment("about") view=About />
                    <Route path=StaticSegment("motion") view=Motion />
                    <ParentRoute path=StaticSegment("new-route") view=Overlay>
                        <Route path=StaticSegment("") view=Home />
                        <Route path=StaticSegment("about") view=About />
                        <Route path=StaticSegment("motion") view=Motion />

                        <ParentRoute path=ParamSegment("id") view=Overlay>
                            <Route path=StaticSegment("") view=Home />
                            <Route path=StaticSegment("about") view=About />
                            <Route path=StaticSegment("motion") view=Motion />
                        </ParentRoute>
                    </ParentRoute>
                </ParentRoute>
            </FluidRoutes>
        </Router>
    }
}
