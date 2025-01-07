use std::collections::HashMap;

use leptos::{html::Div, logging::log, prelude::*};
use web_sys::wasm_bindgen::{prelude::Closure, JsCast};

#[derive(Clone, Debug)]
pub struct OutletNodes {
    pub(crate) outlet_route: StoredValue<String>,
    pub(crate) intro_node: NodeRef<Div>,
    pub(crate) outro_node: NodeRef<Div>,
    pub(crate) is_transitioning: RwSignal<bool>,
}

#[derive(Clone, Debug)]
pub struct FluidManager {
    pub is_transitioning: RwSignal<bool>,
    pub(crate) outlet_nodes: RwSignal<HashMap<String, OutletNodes>>,
    pub(crate) location: StoredValue<Memo<String>>,
    // TODO: Use this for backwards navigation
    pub(crate) recent_navigations: RwSignal<Vec<String>>,
    pub(crate) outlet_route_cache: RwSignal<Vec<String>>,
    pub(crate) navigate_backwards: RwSignal<bool>,
    pub(crate) initialized: StoredValue<bool>,
    pub(crate) current_location: StoredValue<String>,
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
            navigate_backwards: RwSignal::new(false),
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

    pub(crate) fn transition(&mut self) {
        let candidates = self.outlet_route_cache.get();

        let matched_outlet =
            Self::match_location_to_outlet(self.location.get_value().get_untracked(), &candidates)
                .unwrap();

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

        let matched_outlet_nodes = self.outlet_nodes.with_untracked(|outlet_routes| {
            outlet_routes
                .get(matched_outlet)
                .expect("Intro route should exist")
                .clone()
        });

        log!(
            "This is the matched outlet: {}",
            matched_outlet_nodes.outlet_route.get_value()
        );

        let outro_node = matched_outlet_nodes
            .outro_node
            .get_untracked()
            .expect("Outro node should be mounted");

        outro_node.replace_children_with_node_0();
        outro_node.append_child(&cloned_intro_node).unwrap();

        matched_outlet_nodes.is_transitioning.set(true);
        self.current_location
            .set_value(self.location.get_value().get_untracked());
        self.clean_cache_hierarchy(matched_outlet);
    }

    pub(crate) fn add_outlet_route_nodes(
        &mut self,
        route: String,
        intro_node: NodeRef<Div>,
        outro_node: NodeRef<Div>,
        is_transitioning: RwSignal<bool>,
    ) {
        self.outlet_route_cache
            .update_untracked(|cache| cache.push(route.clone()));
        self.outlet_nodes.update_untracked(|nodes| {
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

    pub(crate) fn clean_cache_hierarchy(&mut self, route: &str) {
        self.outlet_route_cache.update_untracked(|routes| {
            if let Some(pos) = routes.iter().position(|r| r == route) {
                // Retain up to and including the matched route
                // This is because outlets are added in their hierarchy order
                routes.truncate(pos + 1);
            }
        })
    }

    pub(crate) fn remove_disposed_outlet_route(&mut self, route: String) {
        log!("Disposing of route: {}", route);
        self.outlet_route_cache
            .update_untracked(|cache| cache.retain_mut(|val| *val != route));
        self.outlet_nodes.update_untracked(|nodes| {
            nodes.remove(&route)
            // We would love to have an expect here
            // But clean ups can sometimes be delayed and then happen close together, which causes this to crash
            // Attempts to clean up twice
            // .expect("Removal of nodes by old route should yield the old nodes");
        });
    }

    // don't have to add a whole new outlet node because the previous node refs and values should be valid
    pub(crate) fn update_outlet_nodes_route(&mut self, previous_route: String, new_route: String) {
        self.outlet_route_cache.update_untracked(|cache| {
            cache.retain_mut(|val| val != &previous_route);
            cache.push(new_route.clone());
        });

        self.outlet_nodes.update_untracked(|nodes| {
            let outlet_node = nodes
                .remove(&previous_route)
                .expect("Removal of nodes by old route should yield the old nodes");

            nodes.insert(new_route, outlet_node);
        })
    }

    // TODO TEST THIS
    // ADD NEW OUTLETS TO A VEC OF STRINGS TO MATCH AGAINST THE CURRENT LOCATION!
    // USE THIS FUNCTION TO RETRIEVE THE STRING AND GRAB THE TARGET OUTLET FOR TRANSITION
    pub(crate) fn match_location_to_outlet<'a>(
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

    // fn match_location_to_outlet<'a>(
    //     target1: &str,
    //     target2: &str,
    //     candidates: &'a [String],
    // ) -> Option<&'a String> {
    //     fn tokenize(path: &str) -> Vec<&str> {
    //         path.split('/').filter(|&s| !s.is_empty()).collect()
    //     }

    //     fn calculate_similarity(target_tokens: &[&str], candidate_tokens: &[&str]) -> usize {
    //         target_tokens
    //             .iter()
    //             .zip(candidate_tokens.iter())
    //             .take_while(|(t, c)| t == c)
    //             .count()
    //     }

    //     fn score_similarity(
    //         target_tokens: &[&str],
    //         candidate: &str,
    //         candidate_tokens: &[&str],
    //     ) -> (usize, usize) {
    //         let similarity = calculate_similarity(target_tokens, candidate_tokens);
    //         let penalty_for_length = usize::MAX - candidate.len(); // Shorter paths win on ties
    //         (similarity, penalty_for_length)
    //     }

    //     let target1_tokens = tokenize(target1);
    //     let target2_tokens = tokenize(target2);

    //     candidates.iter().max_by_key(|candidate| {
    //         let candidate_tokens = tokenize(candidate);

    //         // Calculate scores for both targets
    //         let score1 = score_similarity(&target1_tokens, candidate, &candidate_tokens);
    //         let score2 = score_similarity(&target2_tokens, candidate, &candidate_tokens);

    //         // Use the best score for this candidate
    //         score1.max(score2)
    //     })
    // }

    // TODO: Probably need an effect listener to the route
    // so we can check if we're going backwards or forwards
    fn listen(&self) {
        let inner_manager = self.clone();
        let closure = Closure::wrap(Box::new(move |_: web_sys::PopStateEvent| {
            // TODO: Use this to reverse animations
            inner_manager.navigate_backwards.set(true);
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
