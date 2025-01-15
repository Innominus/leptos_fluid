use leptos::{html::Div, logging::log, prelude::*};
use leptos_router::{
    components::Outlet,
    hooks::{use_location, use_matched},
};
use web_sys::{wasm_bindgen::JsCast, AnimationEvent, Node};

use crate::animators::fluid_manager::FluidManager;

#[component]
pub fn FluidOutlet(intro_class: &'static str, outro_class: &'static str) -> impl IntoView {
    // Setup variables needed for each stage
    // TODO: Probably refactor and make this a lot neater once working
    let mut manager = FluidManager::get_manager();

    let intro_node_ref = NodeRef::new();
    let outro_node_ref = NodeRef::<Div>::new();

    let is_transitioning = RwSignal::new(false);
    let navigate_backwards = manager.navigate_backwards;

    let matched_route = use_matched();
    let location = use_location().pathname;

    // TRACKS CHANGES IN THE CURRENT ROUTE IF A PARENT ROUTE HAS PARAM SEGMENTS/DYNAMIC CHANGES
    let outlet_current_route = StoredValue::new(matched_route.get_untracked());
    let outlet_initialized = StoredValue::new(false);
    let mut inner_manager = manager.clone();
    Effect::new(move || {
        if !outlet_initialized.get_value() {
            outlet_initialized.set_value(true);
            return;
        }

        inner_manager
            .update_outlet_nodes_route(outlet_current_route.get_value(), matched_route.get());

        outlet_current_route.set_value(matched_route.get());
    });

    // TODO: Breakout initialization into its own function
    if !manager.initialized.get_value() {
        let root_outlet_ran_first_time = StoredValue::new(false);
        manager.location.set_value(location);
        manager.current_location.set_value(location.get_untracked());
        let inner_manager = manager.clone();
        Effect::new(move || {
            location.track();
            if !root_outlet_ran_first_time.get_value() {
                root_outlet_ran_first_time.set_value(true);
                return;
            }

            let mut inner_manager = inner_manager.clone();
            // Ensure old node is cleaned up for fast navigations

            // Perform outbound and inbound transition
            // Cloning this twice isn't fantastic to be honest
            // Refactor maybe behind a pointer
            inner_manager.transition();
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

    let animation_classes = move || {
        if is_transitioning.get() {
            if navigate_backwards.get_untracked() {
                (intro_class, outro_class)
            } else {
                (outro_class, intro_class)
            }
        } else {
            ("", "")
        }
    };

    let outro_ends = move |e: AnimationEvent| {
        // checking if the animation that's ended is on the local FluidOutlet div node
        if e.target().unwrap().unchecked_ref::<Node>()
            == outro_node_ref
                .get_untracked()
                .unwrap()
                .unchecked_ref::<Node>()
        {
            outro_node_ref
                .get_untracked()
                .expect("Node ref should be mounted in outro end")
                .replace_children_with_node_0();
            is_transitioning.set(false);
            navigate_backwards.set(false);
        }
    };

    let animation_direction = move || {
        if navigate_backwards.get() {
            " animation-direction: reverse;"
        } else {
            ""
        }
    };

    on_cleanup(move || {
        manager.remove_disposed_outlet_route(matched_route.get_untracked());
    });

    view! {
        <section style="width: 100%; height: 100%; position: relative; overflow-x: hidden;">
            <div
                node_ref=outro_node_ref
                on:animationend=outro_ends
                class=move || animation_classes().0
                style=move || {
                    "width: 100%; height: 100%; position: absolute; top: 0; left: 0; pointer-events: none; overflow: hidden;"
                        .to_string() + animation_direction()
                }
            ></div>
            <div
                style=move || { "width: 100%; height: 100%;".to_string() + animation_direction() }
                class=move || animation_classes().1
            >
                <div node_ref=intro_node_ref style="width: 100%; height: 100%;">
                    <Outlet />
                </div>
            </div>
        </section>
    }
}
