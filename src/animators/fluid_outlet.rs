use std::collections::HashMap;

use leptos::{html::Div, logging::log, prelude::*};
use leptos_router::{
    components::Outlet,
    hooks::{use_location, use_matched},
};
use web_sys::wasm_bindgen::{prelude::Closure, JsCast};

#[derive(Clone, Debug)]
pub struct OutletNodes {
    outlet_route: StoredValue<String>,
    intro_node: NodeRef<Div>,
    outro_node: NodeRef<Div>,
    is_transitioning: RwSignal<bool>,
}

#[derive(Clone, Debug)]
pub struct FluidManager {
    pub is_transitioning: RwSignal<bool>,
    outlet_nodes: RwSignal<HashMap<String, OutletNodes>>,
    location: StoredValue<Memo<String>>,
    // TODO: Use this for backwards navigation
    recent_navigations: RwSignal<Vec<String>>,
    outlet_route_cache: RwSignal<Vec<String>>,
    navigate_backwards: StoredValue<bool>,
    initialized: StoredValue<bool>,
    current_location: StoredValue<String>,
}

impl FluidManager {
    pub fn new() -> Self {
        if use_context::<FluidManager>().is_some() {
            panic!("Fluid Manager has already been initialized");
        }

        let manager = FluidManager {
            is_transitioning: RwSignal::new(false),
            outlet_nodes: RwSignal::new(HashMap::new()),
            location: StoredValue::new(Memo::new(|_| String::from("/"))),
            recent_navigations: RwSignal::new(Vec::new()),
            outlet_route_cache: RwSignal::new(Vec::new()),
            navigate_backwards: StoredValue::new(false),
            initialized: StoredValue::new(false),
            current_location: StoredValue::new(String::new()),
        };

        manager.listen();

        manager
    }

    pub fn get_manager() -> FluidManager {
        use_context::<FluidManager>()
            .expect("Fluid Manager needs to be initialized at the root level")
    }

    fn transition(&self) {
        let candidates = &*self.outlet_route_cache.read_untracked();
        let matched_outlet = match Self::match_location_to_outlet(
            self.location.get_value().get_untracked(),
            candidates,
        ) {
            Some(matched_val) => matched_val,
            // Unwrapping because we know that current_location will exist in the cached outlets vec
            None => Self::match_location_to_outlet(self.current_location.get_value(), candidates)
                .unwrap(),
        };
        log!(
            "TRANSITIONING! Current route: {}, New route: {}, Matched outlet: {}",
            self.current_location.get_value(),
            self.location.get_value().get_untracked(),
            matched_outlet
        );
        log!(
            "Cached routes: {:?}",
            self.outlet_route_cache.get_untracked(),
        );
        log!(
            "Total Outlet Node Routes: {:?}",
            self.outlet_nodes
                .get_untracked()
                .iter()
                .map(|nodes| nodes.0.clone())
                .collect::<Vec<String>>(),
        );
        // TODO: Possibly need to check the option on the hashmap, might be valid to be None
        let cloned_intro_node = self
            .outlet_nodes
            .read_untracked()
            .get(matched_outlet)
            .expect("Intro route should exist in hashmap")
            .intro_node
            .get_untracked()
            .expect("Intro route Node should be mounted")
            .clone_node_with_deep(true)
            .unwrap();

        let matched_outlet_nodes = self.outlet_nodes.with_untracked(|outlet_routes| {
            outlet_routes
                .get(matched_outlet)
                .expect("Intro route should exist")
                .clone()
        });

        matched_outlet_nodes
            .outro_node
            .get_untracked()
            .expect("Outro node should be mounted")
            .append_child(&cloned_intro_node)
            .unwrap();

        matched_outlet_nodes.is_transitioning.set(true);
        self.current_location
            .set_value(self.location.get_value().get_untracked());
    }

    fn add_outlet_route_nodes(
        &mut self,
        route: String,
        intro_node: NodeRef<Div>,
        outro_node: NodeRef<Div>,
        is_transitioning: RwSignal<bool>,
    ) {
        self.outlet_route_cache
            .update(|cache| cache.push(route.clone()));
        self.outlet_nodes.update(|nodes| {
            // TODO: cleanup when outlet no longer exists
            nodes.insert(
                route.clone(),
                OutletNodes {
                    outlet_route: StoredValue::new(route),
                    intro_node,
                    outro_node,
                    is_transitioning,
                },
            );
        });
    }

    fn remove_disposed_outlet_route(&mut self, route: String) {
        log!("Disposing of route: {}", route);
        self.outlet_route_cache
            .update(|cache| cache.retain_mut(|val| *val != route));
        self.outlet_nodes.update(|nodes| {
            nodes
                .remove(&route)
                .expect("Removal of nodes by old route should yield the old nodes");
        });
    }

    // don't have to add a whole new outlet node because the previous node refs and values should be valid
    fn update_outlet_nodes_route(&mut self, previous_route: String, new_route: String) {
        self.outlet_route_cache.update(|cache| {
            cache.retain_mut(|val| val != &previous_route);
            cache.push(new_route.clone());
        });

        self.outlet_nodes.update(|nodes| {
            let outlet_node = nodes
                .remove(&previous_route)
                .expect("Removal of nodes by old route should yield the old nodes");

            nodes.insert(new_route, outlet_node);
        })
    }

    // TODO TEST THIS
    // ADD NEW OUTLETS TO A VEC OF STRINGS TO MATCH AGAINST THE CURRENT LOCATION!
    // USE THIS FUNCTION TO RETRIEVE THE STRING AND GRAB THE TARGET OUTLET FOR TRANSITION
    fn match_location_to_outlet<'a>(
        target: String,
        candidates: &'a [String],
    ) -> Option<&'a String> {
        fn tokenize(path: &str) -> Vec<&str> {
            path.split('/').filter(|&s| !s.is_empty()).collect()
        }

        fn calculate_similarity(target_tokens: &[&str], candidate_tokens: &[&str]) -> usize {
            target_tokens
                .iter()
                .zip(candidate_tokens.iter())
                .take_while(|(t, c)| t == c)
                .count()
        }

        let target_tokens = tokenize(&target);

        candidates.iter().max_by_key(|candidate| {
            let candidate_tokens = tokenize(candidate);
            let similarity = calculate_similarity(&target_tokens, &candidate_tokens);
            // Prefer higher similarity; if equal, prefer shorter path
            (similarity, usize::MAX - candidate.len())
        })
    }

    // TODO: Probably need an effect listener to the route
    // so we can check if we're going backwards or forwards
    fn listen(&self) {
        let inner_manager = self.clone();
        let closure = Closure::wrap(Box::new(move |_: web_sys::PopStateEvent| {
            // TODO: Use this to reverse animations
            inner_manager.navigate_backwards.set_value(true);
            // TODO: Figure out transitioning route
            // inner_manager.transition();
            log!("PopState event triggered");
        }) as Box<dyn FnMut(web_sys::PopStateEvent)>);

        window()
            .add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref())
            .expect("should register popstate listener");

        closure.forget();
    }
}

#[component]
pub fn FluidOutlet(intro_class: &'static str, outro_class: &'static str) -> impl IntoView {
    // Setup variables needed for each stage
    // TODO: Probably refactor and make this a lot neater once working
    let mut manager = FluidManager::get_manager();

    let intro_node_ref = NodeRef::new();
    let outro_node_ref = NodeRef::<Div>::new();

    let is_transitioning = RwSignal::new(false);
    let animation_class = RwSignal::new(intro_class);
    let navigate_backwards = manager.navigate_backwards;

    let matched_route = use_matched();
    let location = use_location().pathname;

    log!("Path in outlet: {}", matched_route.get());

    // TRACKS CHANGES IN THE CURRENT ROUTE IF A PARENT ROUTE HAS PARAM SEGMENTS/DYNAMIC CHANGES
    let outlet_current_route = StoredValue::new(matched_route.get_untracked());
    let outlet_initialized = StoredValue::new(false);
    let mut inner_manager = manager.clone();
    Effect::new(move || {
        matched_route.track();
        if !outlet_initialized.get_value() {
            outlet_initialized.set_value(true);
            return;
        }

        inner_manager.update_outlet_nodes_route(
            outlet_current_route.get_value(),
            matched_route.get_untracked(),
        );

        outlet_current_route.set_value(matched_route.get_untracked());
        log!("CHANGED PATH IN OUTLET: {}", matched_route.get());
    });

    // TODO: Breakout initialization into its own function
    if !manager.initialized.get_value() {
        let root_outlet_ran_first_time = StoredValue::new(false);
        manager.location.set_value(location);
        manager
            .current_location
            .set_value(matched_route.get_untracked());
        let inner_manager = manager.clone();
        Effect::new(move || {
            location.track();
            log!("Matched path in outlet: {}", matched_route.get());
            log!("Location: {}", location.get());
            if !root_outlet_ran_first_time.get_value() {
                root_outlet_ran_first_time.set_value(true);
                return;
            }
            // Ensure old node is cleaned up for fast navigations
            outro_node_ref
                .get_untracked()
                .expect("Node ref should be mounted for outro node ref")
                .replace_children_with_node_0();

            // Perform outbound and inbound transition
            // Cloning this twice isn't fantastic to be honest
            // Refactor maybe behind a pointer
            let inner_manager = inner_manager.clone();
            animation_class.set("");
            inner_manager.transition();
            request_animation_frame(move || {
                animation_class.set(intro_class);
            });
        });

        manager.initialized.set_value(true);
    }

    // Add nodes to manager for transitioning
    manager.add_outlet_route_nodes(
        matched_route.get_untracked(),
        intro_node_ref,
        outro_node_ref,
        is_transitioning,
    );

    // Effect::new(move || request_animation_frame(move || animation_class.set(intro_class)));

    // TODO: Test while navigating fast
    let outro = move || {
        if is_transitioning.get() {
            if navigate_backwards.get_value() {
                intro_class
            } else {
                outro_class
            }
        } else {
            ""
        }
    };

    // TODO: remove children when outro_ends
    let outro_ends = move |_| {
        outro_node_ref
            .get_untracked()
            .expect("Node ref should be mounted in outro end")
            .replace_children_with_node_0();
        is_transitioning.set(false);
        log!("Outro ended");
    };

    on_cleanup(move || {
        log!(
            "Parent route outlet {} being cleaned up :)",
            matched_route.get_untracked()
        );
        manager.remove_disposed_outlet_route(matched_route.get_untracked());
    });

    view! {
        <section style="width: 100%; height: 100%; position: relative; overflow-x: hidden;">
            <div
                node_ref=outro_node_ref
                on:animationend=outro_ends
                class=outro
                style="width: 100%; height: 100%; position: absolute; top: 0; left: 0; pointer-events: none; overflow: hidden;"
            ></div>

            <div style="width: 100%; height: 100%" class=move || animation_class.get()>
                <div node_ref=intro_node_ref style="width: 100%; height: 100%">
                    <Outlet />
                </div>
            </div>
        </section>
    }
}

// #[component]
// pub fn AnimatedInlet(
//     #[prop(optional)] class: &'static str,
//     intro_class: &'static str,
//     children: Children,
// ) -> impl IntoView {
//     let matched_route = use_matched();
//     let location = use_location();
//     let mut manager = FluidManager::get_manager();
//     let node_ref = NodeRef::new();
//     let animation_class = RwSignal::new("");
//     log!("Path in Inlet: {}", location.pathname.get());
//     Effect::new(move || request_animation_frame(move || animation_class.set(intro_class)));

//     manager.add_inlet_route("/".to_string(), node_ref);

//     view! {
//         <div
//             style="width: 100%; height: 100%"
//             class=move || class.to_string() + " " + animation_class.get()
//         >
//             <div node_ref=node_ref style="width: 100%; height: 100%">
//                 {children()}
//             </div>
//         </div>
//     }
// }
