use leptos::{html::Div, prelude::*};
use leptos_router::{
    components::Outlet,
    hooks::{use_location, use_matched},
};
use web_sys::AnimationEvent;

use crate::fluid_manager::FluidManager;

// Child animations are disabled on the cloned outro layer so only the route-level
// intro/outro animations drive transition timing.
const NO_ANIMATION_CSS: &str = ".no-animations *{animation-duration:0s!important;transition-duration:0s!important;animation-delay:0s!important;transition-delay:0s!important;animation-iteration-count:1!important;scroll-behavior:auto!important;}";
const SECTION_STYLE: &str = "width:100%;height:100%;position:relative;isolation:isolate;";
const SECTION_STYLE_TRANSITIONING: &str =
    "width:100%;height:100%;position:relative;isolation:isolate;overflow:hidden;";
const OUTRO_STYLE: &str =
    "width:100%;height:100%;position:absolute;top:0;left:0;pointer-events:none;overflow:hidden;";
const OUTRO_STYLE_REVERSED: &str = "width:100%;height:100%;position:absolute;top:0;left:0;pointer-events:none;overflow:hidden;z-index:1;";
const INTRO_STYLE: &str = "width:100%;height:100%;";
const INTRO_DONE: u8 = 0b01;
const OUTRO_DONE: u8 = 0b10;
const BOTH_DONE: u8 = INTRO_DONE | OUTRO_DONE;

/// Outlet replacement that renders incoming and outgoing route layers.
#[component]
pub fn FluidOutlet(
    /// CSS animation class used for the incoming layer.
    #[prop(into)]
    intro_class: Signal<&'static str>,
    /// CSS animation class used for the outgoing layer.
    #[prop(into)]
    outro_class: Signal<&'static str>,
) -> impl IntoView {
    let mut manager = FluidManager::get_manager();

    let intro_node_ref = NodeRef::new();
    let outro_node_ref = NodeRef::<Div>::new();

    let is_transitioning = RwSignal::new(false);
    let navigate_backwards = manager.navigate_backwards;

    let matched_route = use_matched();
    let location = use_location().pathname;

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

    if !manager.initialized.get_value() {
        let root_outlet_ran_first_time = StoredValue::new(false);
        manager.current_location.set_value(location.get_untracked());
        let inner_manager = manager.clone();
        Effect::new(move || {
            let next_location = location.get();
            if !root_outlet_ran_first_time.get_value() {
                root_outlet_ran_first_time.set_value(true);
                return;
            }

            let mut inner_manager = inner_manager.clone();
            inner_manager.transition(next_location);
        });

        manager.initialized.set_value(true);
    }

    manager.add_outlet_route_nodes(
        matched_route.get_untracked(),
        intro_node_ref,
        outro_node_ref,
        is_transitioning,
    );

    let intro_animation_class = move || {
        if !is_transitioning.get() {
            return "";
        }

        if navigate_backwards.get_untracked() {
            // Reverse intro/outro class assignment for backward navigation.
            outro_class.get_untracked()
        } else {
            intro_class.get_untracked()
        }
    };

    let (intro_handler, outro_handler) =
        setup_animation_handlers(outro_node_ref, is_transitioning, navigate_backwards);

    let outro_animation_class = move || {
        let transition_class = if !is_transitioning.get() {
            ""
        } else if navigate_backwards.get_untracked() {
            intro_class.get_untracked()
        } else {
            outro_class.get_untracked()
        };

        if transition_class.is_empty() {
            "no-animations".to_string()
        } else {
            let mut classes = String::with_capacity(transition_class.len() + 14);
            classes.push_str(transition_class);
            classes.push_str(" no-animations");
            classes
        }
    };

    let animation_direction = move || navigate_backwards.get().then_some(true);

    let section_style = move || {
        if is_transitioning.get() {
            SECTION_STYLE_TRANSITIONING
        } else {
            SECTION_STYLE
        }
    };

    let outro_style = move || {
        if navigate_backwards.get() {
            OUTRO_STYLE_REVERSED
        } else {
            OUTRO_STYLE
        }
    };

    on_cleanup(move || {
        manager.remove_disposed_outlet_route(matched_route.get_untracked());
    });

    view! {
        <style>{NO_ANIMATION_CSS}</style>
        <section style=section_style>
            <div
                data-reverse=animation_direction
                node_ref=outro_node_ref
                on:animationend=outro_handler
                class=outro_animation_class
                style=outro_style
            ></div>
            <div
                data-reverse=animation_direction
                node_ref=intro_node_ref
                on:animationend=intro_handler
                style=INTRO_STYLE
                class=intro_animation_class
            >
                <Outlet />
            </div>
        </section>
    }
}

fn setup_animation_handlers(
    outro_node_ref: NodeRef<Div>,
    is_transitioning: RwSignal<bool>,
    navigate_backwards: RwSignal<bool>,
) -> (impl Fn(AnimationEvent), impl Fn(AnimationEvent)) {
    let finished_layers = RwSignal::new(0u8);
    let intro_ends = create_animation_handler(
        INTRO_DONE,
        finished_layers,
        outro_node_ref,
        is_transitioning,
        navigate_backwards,
    );
    let outro_ends = create_animation_handler(
        OUTRO_DONE,
        finished_layers,
        outro_node_ref,
        is_transitioning,
        navigate_backwards,
    );

    (intro_ends, outro_ends)
}

fn create_animation_handler(
    done_mask: u8,
    finished_layers: RwSignal<u8>,
    outro_node_ref: NodeRef<Div>,
    is_transitioning: RwSignal<bool>,
    navigate_backwards: RwSignal<bool>,
) -> impl Fn(AnimationEvent) {
    move |e: AnimationEvent| {
        // Ignore bubbled child animation events; only react to the wrapper node.
        if e.target() != e.current_target() {
            return;
        }

        finished_layers.update(|state| *state |= done_mask);
        if finished_layers.get_untracked() != BOTH_DONE {
            return;
        }

        // Wait for both layers to finish before clearing the cloned outro DOM.
        if let Some(outro_node) = outro_node_ref.get_untracked() {
            outro_node.replace_children_with_node_0();
        }
        is_transitioning.set(false);
        navigate_backwards.set(false);
        finished_layers.set(0);
    }
}
