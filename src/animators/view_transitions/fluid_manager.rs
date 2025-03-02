use std::{collections::HashMap, fmt::Debug};

use leptos::{html::Div, prelude::*};
use web_sys::{
    wasm_bindgen::{prelude::Closure, JsCast},
    Element, HtmlElement,
};

use crate::animators::utils::{
    get_scroll_pos_of_attr_children, set_scroll_pos_to_children_with_attr,
};

const SCROLLABLE_ATTR: &'static str = "data-scrollable";

#[derive(Clone, Debug)]
pub struct OutletNodes {
    pub(crate) outlet_route: StoredValue<String>,
    pub(crate) intro_node: NodeRef<Div>,
    pub(crate) outro_node: NodeRef<Div>,
    pub(crate) is_transitioning: RwSignal<bool>,
    // pub(crate) scroll_nodes: Vec<NodeRef<Box<dyn ElementType>>>,
}

#[derive(Clone, Debug)]
pub struct FluidManager {
    pub is_transitioning: RwSignal<bool>,
    pub(crate) outlet_nodes: RwSignal<HashMap<String, OutletNodes>>,
    pub(crate) location: StoredValue<Memo<String>>,
    pub(crate) outlet_route_cache: RwSignal<Vec<String>>,
    pub(crate) navigate_backwards: RwSignal<bool>,
    pub(crate) skip_transition: StoredValue<bool>,
    pub(crate) initialized: StoredValue<bool>,
    pub(crate) current_location: StoredValue<String>,
    pub(crate) generated_routes: StoredValue<Vec<Vec<String>>>,
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
            outlet_route_cache: RwSignal::new(Vec::new()),
            navigate_backwards: RwSignal::new(false),
            skip_transition: StoredValue::new(false),
            initialized: StoredValue::new(false),
            current_location: StoredValue::new(String::new()),
            generated_routes: StoredValue::new(Vec::new()),
        };

        manager.setup_incompatibility_listener();

        manager
    }

    pub fn get_manager() -> FluidManager {
        use_context::<FluidManager>()
            .expect("Fluid Manager needs to be initialized at the root level")
    }

    pub(crate) fn transition(&mut self) {
        let matched_outlet = self
            .match_location_to_outlet(self.location.get_value().get_untracked())
            .unwrap();

        if self.check_skip_transition() {
            self.current_location
                .set_value(self.location.get_value().get_untracked());
            self.clean_cache_hierarchy(&matched_outlet);
            return;
        }

        self.set_reversal();

        let intro_element = self
            .outlet_nodes
            .read_untracked()
            .get(&matched_outlet)
            .expect("Intro route should exist in hashmap")
            .intro_node
            .get_untracked()
            .expect("Intro route Node should be mounted");

        let scroll_positions = get_scroll_pos_of_attr_children(&intro_element, SCROLLABLE_ATTR);

        let cloned_intro_node = intro_element.clone_node_with_deep(true).unwrap();

        let matched_outlet_nodes = self.outlet_nodes.with_untracked(|outlet_routes| {
            outlet_routes
                .get(&matched_outlet)
                .expect("Intro route should exist")
                .clone()
        });

        let outro_node = matched_outlet_nodes
            .outro_node
            .get_untracked()
            .expect("Outro node should be mounted");

        outro_node.replace_children_with_node_0();
        outro_node.append_child(&cloned_intro_node).unwrap();
        set_scroll_pos_to_children_with_attr(&outro_node, SCROLLABLE_ATTR, scroll_positions);

        matched_outlet_nodes.is_transitioning.set(true);
        self.current_location
            .set_value(self.location.get_value().get_untracked());
        self.clean_cache_hierarchy(&matched_outlet);
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
                // TODO: Test if this theory is always correct
                routes.truncate(pos + 1);
            }
        })
    }

    pub(crate) fn remove_disposed_outlet_route(&mut self, route: String) {
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
    pub(crate) fn match_location_to_outlet<'a>(&self, target: String) -> Option<String> {
        let target_tokens = tokenize(&target);

        self.outlet_route_cache
            .read()
            .iter()
            .max_by_key(|candidate| {
                let candidate_tokens = tokenize(candidate);
                let similarity = calculate_similarity(&target_tokens, &candidate_tokens);
                // Prefer higher similarity; if equal, prefer shorter path
                (similarity, usize::MAX - candidate.len())
            })
            .cloned()
    }

    pub(crate) fn set_reversal(&self) {
        fn get_route_index<'a>(
            incoming_route: &str,
            generated_routes: &'a [Vec<String>],
        ) -> Option<usize> {
            // Tokenize the incoming route into components
            let incoming_tokens: Vec<&str> = incoming_route.split('/').collect();

            // Iterate through the generated routes and match
            for route in generated_routes {
                if route.len() != incoming_tokens.len() {
                    continue; // Skip routes with different lengths
                }

                let mut is_match = true;
                for (token, &ref pattern) in incoming_tokens.iter().zip(route) {
                    if pattern.starts_with(':') || pattern.starts_with('*') {
                        // This is a wildcard segment, match any incoming token
                        continue;
                    } else if pattern != *token {
                        // Static segments must match exactly
                        is_match = false;
                        break;
                    }
                }

                if is_match {
                    return generated_routes
                        .iter()
                        .position(|generated_route| generated_route == route);
                }
            }

            None // No match found
        }
        let read_value = &self.current_location.read_value();
        let current_location_index =
            get_route_index(&read_value, self.generated_routes.get_value().as_slice());

        let read_value = &self.location.get_value().read_untracked();
        let new_location_index =
            get_route_index(&read_value, self.generated_routes.get_value().as_slice());

        if current_location_index.unwrap_or(0) > new_location_index.unwrap_or(0) {
            self.navigate_backwards.set(true);
        }
    }

    // TODO: Probably need an effect listener to the route
    // so we can check if we're going backwards or forwards
    fn setup_incompatibility_listener(&self) {
        if is_back_button_compatible() {
            return;
        }

        let inner_manager = self.clone();
        let closure = Closure::wrap(Box::new(move |_: web_sys::PopStateEvent| {
            inner_manager.skip_transition.set_value(true);
        }) as Box<dyn FnMut(web_sys::PopStateEvent)>);

        window()
            .add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref())
            .expect("should register popstate listener");

        closure.forget();
    }

    fn check_skip_transition(&mut self) -> bool {
        if self.skip_transition.get_value() {
            self.skip_transition.set_value(false);
            return true;
        }

        false
    }
}

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

fn is_back_button_compatible() -> bool {
    // Safari on MacOS and all Apple devices that aren't a Macintosh lead to blips and glitches
    // due to the backswipe animation built into the safari browser engine
    let incompatible_agent_keywords = vec!["safari", "IPad", "IPhone", "IPod"];

    let user_agent = window().navigator().user_agent();

    if let Ok(agent_string) = user_agent {
        return incompatible_agent_keywords
            .iter()
            .find(|os| {
                agent_string
                    .to_lowercase()
                    .contains(os.to_lowercase().as_str())
            })
            .is_none();
    }

    true
}
