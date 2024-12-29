use std::collections::HashMap;

use leptos::{html::Div, logging::log, prelude::*};
use leptos_router::{
    components::{Outlet, ToHref},
    hooks::use_location,
};
use web_sys::wasm_bindgen::{JsCast, prelude::Closure};

#[derive(Clone, Debug)]
pub struct FluidManager {
    pub is_transitioning: RwSignal<bool>,
    outlet_routes: RwSignal<HashMap<String, (NodeRef<Div>, RwSignal<bool>)>>,
    inlet_routes: RwSignal<HashMap<String, NodeRef<Div>>>,
}

impl FluidManager {
    pub fn new() -> Self {
        if use_context::<FluidManager>().is_some() {
            panic!("Fluid Manager has already been initialized");
        }

        let manager = FluidManager {
            is_transitioning: RwSignal::new(false),
            outlet_routes: RwSignal::new(HashMap::new()),
            inlet_routes: RwSignal::new(HashMap::new()),
        };

        manager.listen();

        manager
    }

    #[inline(always)]
    pub fn get_manager() -> FluidManager {
        use_context::<FluidManager>()
            .expect("Fluid Manager needs to be initialized at the root level")
    }

    // TODO: Dummy transition for now, changes to react to actual route
    fn transition(&self, route: &String) {
        // TODO: Possibly need to check the option on the hashmap, might be valid to be None
        let cloned_node = self
            .inlet_routes
            .read_untracked()
            .get(&"/".to_string())
            .expect("Inlet route should exist")
            .get_untracked()
            .expect("Inlet route Node should be mounted")
            .clone_node_with_deep(true)
            .unwrap();

        let (outlet_node, is_transitioning) = self.outlet_routes.with_untracked(|outlet_routes| {
            *outlet_routes
                .get(&"/".to_string())
                .expect("Inlet route should exist")
        });

        outlet_node
            .get_untracked()
            .expect("Outlet node should be mounted")
            .append_child(&cloned_node)
            .unwrap();

        is_transitioning.set(true);
    }

    fn add_outlet_route(
        &mut self,
        route: String,
        outlet_node_combo: (NodeRef<Div>, RwSignal<bool>),
    ) {
        self.outlet_routes.update(|routes| {
            routes.insert(route, outlet_node_combo);
        });
    }

    fn add_inlet_route(&mut self, route: String, node: NodeRef<Div>) {
        self.inlet_routes.update(|routes| {
            routes.insert(route, node);
        });
    }

    // TODO: Probably need an effect listener to the route
    // so we can check if we're going backwards or forwards
    fn listen(&self) {
        let closure = Closure::wrap(Box::new(move |event: web_sys::PopStateEvent| {
            log!("PopState event triggered");
        }) as Box<dyn FnMut(web_sys::PopStateEvent)>);

        window()
            .add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref())
            .expect("should register popstate listener");

        closure.forget();
    }
}

#[component]
pub fn FluidOutlet(outro_class: &'static str) -> impl IntoView {
    let location = use_location();
    let node_ref = NodeRef::new();
    let mut manager = FluidManager::get_manager();
    let transitioning = manager.is_transitioning;

    manager.add_outlet_route("/".to_string(), (node_ref, transitioning));

    log!("Path in outlet: {}", location.pathname.get());

    // TODO: Test while navigating fast
    let outro = move || {
        if transitioning.get() { outro_class } else { "" }
    };

    // TODO: remove children when outro_ends
    let outro_ends = move |_| {
        node_ref
            .get_untracked()
            .expect("Node ref should be mounted in outro end")
            .replace_children_with_node_0();
        transitioning.set(false)
    };

    view! {
        <section style="width: 100%; height: 100%; position: relative; overflow-x: hidden;">
            <div
                node_ref=node_ref
                on:animationend=outro_ends
                class=outro
                style="width: 100%; height: 100%; position: absolute; top: 0; left: 0; pointer-events: none; overflow: hidden;"
            ></div>
            <Outlet />

        </section>
    }
}

#[component]
pub fn AnimatedInlet(
    #[prop(optional)] class: &'static str,
    intro_class: &'static str,
    children: Children,
) -> impl IntoView {
    let location = use_location();
    let mut manager = FluidManager::get_manager();
    let node_ref = NodeRef::new();
    let animation_class = RwSignal::new("");
    log!("Path in Inlet: {}", location.pathname.get());
    Effect::new(move || request_animation_frame(move || animation_class.set(intro_class)));

    manager.add_inlet_route("/".to_string(), node_ref);

    view! {
        <div
            style="width: 100%; height: 100%"
            class=move || class.to_string() + " " + animation_class.get()
        >
            <div node_ref=node_ref style="width: 100%; height: 100%">
                {children()}
            </div>
        </div>
    }
}

// TODO: Spread the rest of the A tag props onto the underlying A tag
#[component]
pub fn A<H>(href: H, children: Children) -> impl IntoView
where
    H: ToHref + Send + Sync + 'static,
{
    let manager = FluidManager::get_manager();

    let href_fn = href.to_href();

    let route = href_fn();

    view! {
        <leptos_router::components::A
            on:click=move |_| manager.transition(&route)
            href=route.clone()
        >
            {children()}
        </leptos_router::components::A>
    }
}
