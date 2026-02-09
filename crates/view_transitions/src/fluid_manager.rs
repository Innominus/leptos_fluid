use std::collections::HashMap;

use leptos::{html::Div, prelude::*};
use web_sys::wasm_bindgen::{prelude::Closure, JsCast};

use crate::utils::{get_scroll_pos_of_attr_children, set_scroll_pos_to_children_with_attr};

const SCROLLABLE_ATTR: &str = "data-scrollable";

/// Internal outlet node references tracked by `FluidManager`.
#[derive(Clone, Debug)]
pub struct OutletNodes {
    pub(crate) intro_node: NodeRef<Div>,
    pub(crate) outro_node: NodeRef<Div>,
    pub(crate) is_transitioning: RwSignal<bool>,
}

/// Shared transition coordinator for `FluidRoutes` + `FluidOutlet`.
#[derive(Clone, Debug)]
pub struct FluidManager {
    /// Global transition flag across registered outlets.
    pub is_transitioning: RwSignal<bool>,
    pub(crate) outlet_nodes: RwSignal<HashMap<String, OutletNodes>>,
    pub(crate) outlet_route_cache: RwSignal<Vec<String>>,
    pub(crate) navigate_backwards: RwSignal<bool>,
    pub(crate) skip_transition: StoredValue<bool>,
    pub(crate) initialized: StoredValue<bool>,
    pub(crate) current_location: StoredValue<String>,
    pub(crate) generated_routes: StoredValue<Vec<Vec<String>>>,
}

impl FluidManager {
    /// Builds manager state and registers browser-compatibility listeners.
    ///
    /// Provide exactly once near the router root with `provide_context`.
    pub fn new() -> Self {
        #[cfg(debug_assertions)]
        if use_context::<FluidManager>().is_some() {
            leptos::logging::warn!("Fluid Manager has already been initialized");
        }

        let manager = FluidManager {
            is_transitioning: RwSignal::new(false),
            outlet_nodes: RwSignal::new(HashMap::new()),
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

    /// Reads the manager from Leptos context and fails fast when missing.
    pub fn get_manager() -> FluidManager {
        use_context::<FluidManager>().unwrap()
    }

    pub(crate) fn transition(&mut self, next_location: String) {
        let matched_outlet = self.match_location_to_outlet(&next_location).unwrap();

        if self.check_skip_transition() {
            self.current_location.set_value(next_location);
            self.clean_cache_hierarchy(&matched_outlet);
            return;
        }

        self.set_reversal(&next_location);

        let matched_outlet_nodes = self
            .outlet_nodes
            .with_untracked(|outlet_routes| outlet_routes.get(&matched_outlet).cloned().unwrap());

        let intro_element = matched_outlet_nodes.intro_node.get_untracked().unwrap();

        let scroll_positions = get_scroll_pos_of_attr_children(&intro_element, SCROLLABLE_ATTR);

        // Clone currently visible intro content into the outro layer so both
        // route states can animate simultaneously.
        let cloned_intro_node = intro_element.clone_node_with_deep(true).unwrap();
        let outro_node = matched_outlet_nodes.outro_node.get_untracked().unwrap();

        outro_node.replace_children_with_node_0();
        outro_node.append_child(&cloned_intro_node).unwrap();
        // Preserve scroll positions for explicitly marked nested scroll containers.
        set_scroll_pos_to_children_with_attr(&outro_node, SCROLLABLE_ATTR, scroll_positions);

        matched_outlet_nodes.is_transitioning.set(true);
        self.current_location.set_value(next_location);
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
            nodes.insert(
                route,
                OutletNodes {
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
                routes.truncate(pos + 1);
            }
        })
    }

    pub(crate) fn remove_disposed_outlet_route(&mut self, route: String) {
        self.outlet_route_cache
            .update_untracked(|cache| cache.retain(|val| val != &route));
        self.outlet_nodes
            .update_untracked(|nodes| nodes.remove(&route));
    }

    pub(crate) fn update_outlet_nodes_route(&mut self, previous_route: String, new_route: String) {
        self.outlet_route_cache.update_untracked(|cache| {
            cache.retain(|val| val != &previous_route);
            cache.push(new_route.clone());
        });

        self.outlet_nodes.update_untracked(|nodes| {
            let outlet_node = nodes.remove(&previous_route).unwrap();

            nodes.insert(new_route, outlet_node);
        })
    }

    pub(crate) fn match_location_to_outlet(&self, target: &str) -> Option<String> {
        self.outlet_route_cache
            .read()
            .iter()
            .max_by_key(|candidate| {
                let similarity = common_prefix_len(target, candidate);
                // Prefer longest common prefix; break ties by shorter route length.
                (similarity, usize::MAX - candidate.len())
            })
            .cloned()
    }

    pub(crate) fn set_reversal(&self, new_location: &str) {
        let current_location = self.current_location.read_value();
        let is_backward = self.generated_routes.with_value(|generated_routes| {
            route_index(&current_location, generated_routes).unwrap_or(0)
                > route_index(new_location, generated_routes).unwrap_or(0)
        });

        if is_backward {
            self.navigate_backwards.set(true);
        }
    }

    fn setup_incompatibility_listener(&self) {
        if is_back_button_compatible() {
            return;
        }

        let inner_manager = self.clone();
        let closure = Closure::wrap(Box::new(move |_: web_sys::PopStateEvent| {
            // On known-incompatible engines, skip one transition on popstate.
            inner_manager.skip_transition.set_value(true);
        }) as Box<dyn FnMut(web_sys::PopStateEvent)>);

        window()
            .add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref())
            .unwrap();

        closure.forget();
    }

    fn check_skip_transition(&self) -> bool {
        if self.skip_transition.get_value() {
            self.skip_transition.set_value(false);
            return true;
        }

        false
    }
}

fn route_index(incoming_route: &str, generated_routes: &[Vec<String>]) -> Option<usize> {
    let incoming_segment_count = incoming_route.split('/').count();
    generated_routes
        .iter()
        .enumerate()
        .find_map(|(index, route)| {
            if route.len() != incoming_segment_count {
                return None;
            }

            let is_match = incoming_route
                .split('/')
                .zip(route.iter())
                .all(|(token, pattern)| pattern.starts_with(':') || pattern == token);

            if is_match {
                Some(index)
            } else {
                None
            }
        })
}

fn common_prefix_len(a: &str, b: &str) -> usize {
    a.split('/')
        .filter(|segment| !segment.is_empty())
        .zip(b.split('/').filter(|segment| !segment.is_empty()))
        .take_while(|(a_segment, b_segment)| a_segment == b_segment)
        .count()
}

fn is_back_button_compatible() -> bool {
    if let Ok(mut agent_lower) = window().navigator().user_agent() {
        agent_lower.make_ascii_lowercase();

        if agent_lower.contains("ipad")
            || agent_lower.contains("iphone")
            || agent_lower.contains("ipod")
        {
            return false;
        }

        if agent_lower.contains("safari") && !agent_lower.contains("chrome") {
            return false;
        }
    }

    true
}
