use leptos::prelude::*;

use crate::{MotionSignal, MotionStyle, Transition};

use web_sys::wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};

pub type MotionNodeRef = NodeRef<leptos::html::Custom<&'static str>>;

fn apply_style(element: &Element, style: &MotionStyle) {
    let Some(html_element) = element.dyn_ref::<HtmlElement>() else {
        return;
    };

    let style_decl = html_element.style();
    for (key, value) in style.to_props() {
        let _ = style_decl.set_property(key.as_ref(), &value);
    }
}

fn transition_value(transition: &Transition) -> String {
    if transition.duration_ms == 0 && transition.delay_ms == 0 {
        "none".to_string()
    } else {
        let easing = transition.easing_string();
        format!(
            "all {}ms {} {}ms",
            transition.duration_ms, easing, transition.delay_ms
        )
    }
}

fn apply_transition(element: &HtmlElement, transition: &Transition) {
    let style_decl = element.style();
    let _ = style_decl.set_property("transition", &transition_value(transition));
}

fn animate_to(element: &Element, to: &MotionStyle, transition: &Transition) {
    let Some(html_element) = element.dyn_ref::<HtmlElement>() else {
        return;
    };

    apply_transition(html_element, transition);
    apply_style(element, to);
}

fn motion_element_view(
    tag: &'static str,
    initial: MotionStyle,
    animate: MotionSignal<MotionStyle>,
    transition: Transition,
    while_hover: Option<MotionStyle>,
    while_tap: Option<MotionStyle>,
    class: MotionSignal<String>,
    style: MotionSignal<String>,
    node_ref: MotionNodeRef,
    children: Option<Children>,
) -> impl IntoView {
    let is_hovered = RwSignal::new(false);
    let is_pressed = RwSignal::new(false);

    let base_style = StoredValue::new(initial.clone());
    let transition_store = StoredValue::new(transition);
    let initialized = StoredValue::new(false);

    Effect::new({
        let node_ref = node_ref.clone();
        let initial = initial.clone();
        let animate = animate.clone();
        let base_style = base_style.clone();
        let transition_store = transition_store.clone();
        let initialized = initialized.clone();
        move || {
            if initialized.get_value() {
                return;
            }
            let Some(element) = node_ref.get() else {
                return;
            };
            let element: Element = element.unchecked_into();
            if !initial.is_empty() {
                let Some(html_element) = element.dyn_ref::<HtmlElement>() else {
                    return;
                };
                apply_transition(html_element, &Transition::new().duration_ms(0));
                apply_style(&element, &initial);
            }

            let target = animate.get();
            let next_base = if target.is_empty() {
                initial.clone()
            } else {
                target.clone()
            };
            base_style.set_value(next_base);
            if !target.is_empty() {
                let transition = transition_store.get_value();
                let target_for_anim = target.clone();
                request_animation_frame(move || {
                    animate_to(&element, &target_for_anim, &transition);
                });
            }

            initialized.set_value(true);
        }
    });

    Effect::new({
        let node_ref = node_ref.clone();
        let animate = animate.clone();
        let base_style = base_style.clone();
        let transition_store = transition_store.clone();
        let initialized = initialized.clone();
        let is_hovered = is_hovered.clone();
        let is_pressed = is_pressed.clone();
        move || {
            let target = animate.get();

            if !initialized.get_value() {
                return;
            }

            if target.is_empty() {
                return;
            }
            base_style.set_value(target.clone());

            if is_hovered.get_untracked() || is_pressed.get_untracked() {
                return;
            }

            let Some(element) = node_ref.get_untracked() else {
                return;
            };
            let element: Element = element.unchecked_into();
            let transition = transition_store.get_value();
            animate_to(&element, &target, &transition);
        }
    });

    let on_mouseenter = {
        let node_ref = node_ref.clone();
        let transition_store = transition_store.clone();
        let while_hover = while_hover.clone();
        let is_hovered = is_hovered.clone();
        move |_| {
            let Some(hover_style) = while_hover.clone() else {
                return;
            };
            let Some(element) = node_ref.get_untracked() else {
                return;
            };
            is_hovered.set(true);
            let element: Element = element.unchecked_into();
            let transition = transition_store.get_value();
            animate_to(&element, &hover_style, &transition);
        }
    };

    let on_mouseleave = {
        let node_ref = node_ref.clone();
        let base_style = base_style.clone();
        let transition_store = transition_store.clone();
        let is_hovered = is_hovered.clone();
        let is_pressed = is_pressed.clone();
        move |_| {
            is_hovered.set(false);
            if is_pressed.get_untracked() {
                return;
            }

            let Some(element) = node_ref.get_untracked() else {
                return;
            };
            let element: Element = element.unchecked_into();
            let transition = transition_store.get_value();
            let target = base_style.get_value();

            animate_to(&element, &target, &transition);
        }
    };

    let on_pointerdown = {
        let node_ref = node_ref.clone();
        let transition_store = transition_store.clone();
        let while_tap = while_tap.clone();
        let is_pressed = is_pressed.clone();
        move |_| {
            let Some(tap_style) = while_tap.clone() else {
                return;
            };
            let Some(element) = node_ref.get_untracked() else {
                return;
            };
            is_pressed.set(true);
            let element: Element = element.unchecked_into();
            let transition = transition_store.get_value();
            animate_to(&element, &tap_style, &transition);
        }
    };

    let on_pointerup = {
        let node_ref = node_ref.clone();
        let base_style = base_style.clone();
        let transition_store = transition_store.clone();
        let while_hover = while_hover.clone();
        let is_pressed = is_pressed.clone();
        let is_hovered = is_hovered.clone();
        move |_| {
            if !is_pressed.get_untracked() {
                return;
            }
            is_pressed.set(false);
            let Some(element) = node_ref.get_untracked() else {
                return;
            };
            let element: Element = element.unchecked_into();
            let transition = transition_store.get_value();
            let target = if is_hovered.get_untracked() {
                while_hover
                    .clone()
                    .unwrap_or_else(|| base_style.get_value())
            } else {
                base_style.get_value()
            };
            animate_to(&element, &target, &transition);
        }
    };

    let on_pointercancel = {
        let node_ref = node_ref.clone();
        let base_style = base_style.clone();
        let transition_store = transition_store.clone();
        let while_hover = while_hover.clone();
        let is_pressed = is_pressed.clone();
        let is_hovered = is_hovered.clone();
        move |_| {
            if !is_pressed.get_untracked() {
                return;
            }
            is_pressed.set(false);
            let Some(element) = node_ref.get_untracked() else {
                return;
            };
            let element: Element = element.unchecked_into();
            let transition = transition_store.get_value();
            let target = if is_hovered.get_untracked() {
                while_hover
                    .clone()
                    .unwrap_or_else(|| base_style.get_value())
            } else {
                base_style.get_value()
            };
            animate_to(&element, &target, &transition);
        }
    };

    let on_pointerleave = {
        let node_ref = node_ref.clone();
        let base_style = base_style.clone();
        let transition_store = transition_store.clone();
        let while_hover = while_hover.clone();
        let is_pressed = is_pressed.clone();
        let is_hovered = is_hovered.clone();
        move |_| {
            if !is_pressed.get_untracked() {
                return;
            }
            is_pressed.set(false);
            let Some(element) = node_ref.get_untracked() else {
                return;
            };
            let element: Element = element.unchecked_into();
            let transition = transition_store.get_value();
            let target = if is_hovered.get_untracked() {
                while_hover
                    .clone()
                    .unwrap_or_else(|| base_style.get_value())
            } else {
                base_style.get_value()
            };
            animate_to(&element, &target, &transition);
        }
    };

    let content = children.map(|c| c()).unwrap_or_else(|| ().into_any());

    leptos::html::custom(tag)
        .node_ref(node_ref)
        .class(move || class.get())
        .style(move || style.get())
        .on(leptos::ev::mouseenter, on_mouseenter)
        .on(leptos::ev::mouseleave, on_mouseleave)
        .on(leptos::ev::pointerdown, on_pointerdown)
        .on(leptos::ev::pointerup, on_pointerup)
        .on(leptos::ev::pointercancel, on_pointercancel)
        .on(leptos::ev::pointerleave, on_pointerleave)
        .child(content)
}

#[component]
pub fn MotionElement(
    /// HTML tag name for the underlying element (e.g. "div", "span").
    #[prop(default = "div")]
    tag: &'static str,
    /// Initial animated style applied on mount before the first transition.
    #[prop(default = MotionStyle::default())]
    initial: MotionStyle,
    /// Target style to animate to; updates reactively when this signal changes.
    #[prop(default = MotionSignal::static_value(MotionStyle::default()), into)]
    animate: MotionSignal<MotionStyle>,
    /// Transition settings for animated properties (duration/easing/spring).
    #[prop(default = Transition::default())]
    transition: Transition,
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
    /// Child view(s) inside the motion element.
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    let node_ref = node_ref.unwrap_or_else(MotionNodeRef::new);
    motion_element_view(
        tag,
        initial,
        animate,
        transition,
        while_hover,
        while_tap,
        class,
        style,
        node_ref,
        children,
    )
}

macro_rules! motion_wrapper {
    ($name:ident, $tag:literal) => {
        #[component]
        pub fn $name(
            /// Initial animated style applied on mount before the first transition.
            #[prop(default = MotionStyle::default())]
            initial: MotionStyle,
            /// Target style to animate to; updates reactively when this signal changes.
            #[prop(default = MotionSignal::static_value(MotionStyle::default()), into)]
            animate: MotionSignal<MotionStyle>,
            /// Transition settings for animated properties (duration/easing/spring).
            #[prop(default = Transition::default())]
            transition: Transition,
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
            /// Child view(s) inside the motion element.
            #[prop(optional)]
            children: Option<Children>,
        ) -> impl IntoView {
            let node_ref = node_ref.unwrap_or_else(MotionNodeRef::new);
            motion_element_view(
                $tag,
                initial,
                animate,
                transition,
                while_hover,
                while_tap,
                class,
                style,
                node_ref,
                children,
            )
        }
    };
}

motion_wrapper!(MotionDiv, "div");
motion_wrapper!(MotionSpan, "span");
motion_wrapper!(MotionButton, "button");
