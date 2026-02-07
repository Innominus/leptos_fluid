use leptos::{html::Div, prelude::*};
use leptos_router::{
    components::Outlet,
    hooks::{use_location, use_matched},
};
use web_sys::{AnimationEvent, Node, wasm_bindgen::JsCast};

use crate::fluid_manager::FluidManager;

const NO_ANIMATION_CSS: &str = r#"
    .no-animations * {
      animation-duration: 0s !important;
      transition-duration: 0s !important;
      animation-delay: 0s !important;
      transition-delay: 0s !important;
      animation-iteration-count: 1 !important;
      scroll-behavior: auto !important;
    }
"#;

#[component]
pub fn FluidOutlet(
    #[prop(into)] intro_class: Signal<&'static str>,
    #[prop(into)] outro_class: Signal<&'static str>,
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
            inner_manager.transition();
        });

        manager.initialized.set_value(true);
    }

    manager.add_outlet_route_nodes(
        matched_route.get_untracked(),
        intro_node_ref,
        outro_node_ref,
        is_transitioning,
    );

    let animation_classes = move || {
        if is_transitioning.get() {
            if navigate_backwards.get_untracked() {
                (intro_class.get_untracked(), outro_class.get_untracked())
            } else {
                (outro_class.get_untracked(), intro_class.get_untracked())
            }
        } else {
            ("", "")
        }
    };

    let (intro_handler, outro_handler) = setup_animation_handlers(
        intro_node_ref,
        outro_node_ref,
        is_transitioning,
        navigate_backwards,
    );

    let animation_direction = move || {
        if navigate_backwards.get() {
            Some(true)
        } else {
            None
        }
    };

    let z_index = move || {
        if navigate_backwards.get() {
            "z-index: 1;"
        } else {
            ""
        }
    };

    let hide_while_animating = move || {
        if is_transitioning.get() {
            "overflow: hidden;"
        } else {
            ""
        }
    };

    on_cleanup(move || {
        manager.remove_disposed_outlet_route(matched_route.get_untracked());
    });

    view! {
        <style>{NO_ANIMATION_CSS}</style>
        <section style=move || {
            "width: 100%; height: 100%; position: relative; isolation: isolate;".to_string()
                + hide_while_animating()
        }>
            <div
                data-reverse=animation_direction
                node_ref=outro_node_ref
                on:animationend=outro_handler
                class=move || animation_classes().0.to_string() + " no-animations"
                style=move || {
                    "width: 100%; height: 100%; position: absolute; top: 0; left: 0; pointer-events: none; overflow: hidden;"
                        .to_string()
                        + z_index()
                }
            ></div>
            <div
                data-reverse=animation_direction
                node_ref=intro_node_ref
                on:animationend=intro_handler
                style=move || "width: 100%; height: 100%;".to_string()
                class=move || animation_classes().1
            >
                <Outlet />
            </div>
        </section>
    }
}

fn setup_animation_handlers(
    intro_node_ref: NodeRef<Div>,
    outro_node_ref: NodeRef<Div>,
    is_transitioning: RwSignal<bool>,
    navigate_backwards: RwSignal<bool>,
) -> (impl Fn(AnimationEvent), impl Fn(AnimationEvent)) {
    let intro_has_ended = RwSignal::new(false);
    let outro_has_ended = RwSignal::new(false);

    let cleanup_fn = move || {
        if intro_has_ended.get() && outro_has_ended.get() {
            outro_node_ref
                .get_untracked()
                .expect("Node ref should be mounted in outro end")
                .replace_children_with_node_0();
            is_transitioning.set(false);
            navigate_backwards.set(false);
            intro_has_ended.set(false);
            outro_has_ended.set(false);
        }
    };

    let intro_ends = create_animation_handler(intro_node_ref, intro_has_ended, cleanup_fn);
    let outro_ends = create_animation_handler(outro_node_ref, outro_has_ended, cleanup_fn);

    (intro_ends, outro_ends)
}

fn create_animation_handler(
    node: NodeRef<Div>,
    has_ended: RwSignal<bool>,
    cleanup_fn: impl Fn(),
) -> impl Fn(AnimationEvent) {
    move |e: AnimationEvent| {
        if e.target().unwrap().unchecked_ref::<Node>()
            == node.get_untracked().unwrap().unchecked_ref::<Node>()
        {
            has_ended.set(true);
            cleanup_fn();
        }
    }
}
