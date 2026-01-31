use leptos::prelude::*;

use crate::{MotionElement, MotionNodeRef, MotionSignal, MotionStyle, Transition};

use js_sys::Date;

#[component]
pub fn AnimatePresence(
    /// Whether the element should be present in the DOM.
    #[prop(into)]
    show: Signal<bool>,
    /// Initial style applied on mount before animating to `animate`.
    #[prop(default = MotionStyle::default())]
    initial: MotionStyle,
    /// Style applied while present.
    #[prop(default = MotionStyle::default())]
    animate: MotionStyle,
    /// Style applied before exit.
    #[prop(default = MotionStyle::default())]
    exit: MotionStyle,
    /// Transition used for enter/exit.
    #[prop(default = Transition::default())]
    transition: Transition,
    /// Underlying HTML tag (defaults to "div").
    #[prop(default = "div")]
    tag: &'static str,
    /// Optional style applied while the pointer is hovering the element.
    #[prop(optional)]
    while_hover: Option<MotionStyle>,
    /// Optional style applied while the pointer is pressed down.
    #[prop(optional)]
    while_tap: Option<MotionStyle>,
    /// Class attribute (static or reactive).
    #[prop(default = MotionSignal::static_value(String::new()), into)]
    class: MotionSignal<String>,
    /// Extra CSS style string (non-animated); useful for layout/base styles.
    #[prop(default = MotionSignal::static_value(String::new()), into)]
    style: MotionSignal<String>,
    /// NodeRef for the underlying element; created automatically if omitted.
    #[prop(optional)]
    node_ref: Option<MotionNodeRef>,
    /// Optional callback fired after the exit animation completes.
    #[prop(optional)]
    on_exit_complete: Option<Callback<()>>,
    /// Child view(s) inside the motion element.
    #[prop(optional)]
    children: Option<ChildrenFn>,
) -> impl IntoView {
    let initial_present = show.get_untracked();
    let present = RwSignal::new(initial_present);
    let anim_style = RwSignal::new(animate.clone());
    let exit_generation = StoredValue::new(0u32);
    let animate_store = StoredValue::new(animate);
    let exit_store = StoredValue::new(exit);
    let transition_store = StoredValue::new(transition);
    let initial_store = StoredValue::new(initial);

    Effect::new({
        let present = present.clone();
        let anim_style = anim_style.clone();
        let exit_generation = exit_generation.clone();
        let animate_store = animate_store.clone();
        let exit_store = exit_store.clone();
        let transition_store = transition_store.clone();
        let on_exit_complete = on_exit_complete.clone();
        move || {
            let is_showing = show.get();
            if is_showing {
                exit_generation.set_value(exit_generation.get_value().wrapping_add(1));
                present.set(true);
                anim_style.set(animate_store.get_value());
                return;
            }

            if !present.get_untracked() {
                return;
            }

            let exit_style = exit_store.get_value();
            if exit_style.is_empty() {
                present.set(false);
                if let Some(callback) = on_exit_complete.as_ref() {
                    callback.run(());
                }
                return;
            }

            anim_style.set(exit_style);
            let transition = transition_store.get_value();
            let total_ms = transition.duration_ms + transition.delay_ms;
            if total_ms == 0 {
                present.set(false);
                if let Some(callback) = on_exit_complete.as_ref() {
                    callback.run(());
                }
                return;
            }

            let generation = exit_generation.get_value().wrapping_add(1);
            exit_generation.set_value(generation);
            schedule_exit(
                generation,
                exit_generation,
                present,
                transition,
                on_exit_complete.clone(),
                Date::now(),
            );
        }
    });

    let node_ref = node_ref.unwrap_or_else(MotionNodeRef::new);
    let children_store = StoredValue::new(children);
    let while_hover_store = StoredValue::new(while_hover);
    let while_tap_store = StoredValue::new(while_tap);
    let class_store = StoredValue::new(class);
    let style_store = StoredValue::new(style);
    let node_ref_store = StoredValue::new(node_ref);

    view! {
        <Show when=move || {
            present.get()
        }>
            {move || {
                let content = children_store
                    .with_value(|children| { children.as_ref().map(|c| c()) });
                let node_ref = node_ref_store.get_value();
                let class = class_store.get_value();
                let style = style_store.get_value();
                let initial = initial_store.get_value();
                let transition = transition_store.get_value();
                let while_hover = while_hover_store.get_value();
                let while_tap = while_tap_store.get_value();
                match (while_hover, while_tap) {
                    (Some(hover), Some(tap)) => {
                        view! {
                            <MotionElement
                                tag=tag
                                initial=initial
                                animate=move || anim_style.get()
                                transition=transition
                                while_hover=hover
                                while_tap=tap
                                class=class
                                style=style
                                node_ref=node_ref
                            >
                                {content}
                            </MotionElement>
                        }
                            .into_any()
                    }
                    (Some(hover), None) => {
                        view! {
                            <MotionElement
                                tag=tag
                                initial=initial
                                animate=move || anim_style.get()
                                transition=transition
                                while_hover=hover
                                class=class
                                style=style
                                node_ref=node_ref
                            >
                                {content}
                            </MotionElement>
                        }
                            .into_any()
                    }
                    (None, Some(tap)) => {
                        view! {
                            <MotionElement
                                tag=tag
                                initial=initial
                                animate=move || anim_style.get()
                                transition=transition
                                while_tap=tap
                                class=class
                                style=style
                                node_ref=node_ref
                            >
                                {content}
                            </MotionElement>
                        }
                            .into_any()
                    }
                    (None, None) => {
                        view! {
                            <MotionElement
                                tag=tag
                                initial=initial
                                animate=move || anim_style.get()
                                transition=transition
                                class=class
                                style=style
                                node_ref=node_ref
                            >
                                {content}
                            </MotionElement>
                        }
                            .into_any()
                    }
                }
            }}
        </Show>
    }
}

fn schedule_exit(
    generation: u32,
    exit_generation: StoredValue<u32>,
    present: RwSignal<bool>,
    transition: Transition,
    on_exit_complete: Option<Callback<()>>,
    start_ms: f64,
) {
    request_animation_frame(move || {
        if exit_generation.get_value() != generation {
            return;
        }

        let elapsed = Date::now() - start_ms;
        let total_ms = (transition.duration_ms + transition.delay_ms) as f64;
        if elapsed >= total_ms {
            present.set(false);
            if let Some(callback) = on_exit_complete.as_ref() {
                callback.run(());
            }
            return;
        }

        schedule_exit(
            generation,
            exit_generation,
            present,
            transition,
            on_exit_complete,
            start_ms,
        );
    });
}
