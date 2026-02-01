use leptos::prelude::*;

use crate::{FluidSignal, FluidStyle, Transition};

use web_sys::wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};

pub type FluidNodeRef = NodeRef<leptos::html::Custom<&'static str>>;

fn apply_style(element: &Element, style: &FluidStyle) {
    let Some(html_element) = element.dyn_ref::<HtmlElement>() else {
        return;
    };

    let style_decl = html_element.style();
    for (key, value) in style.to_props() {
        let _ = style_decl.set_property(key.as_ref(), &value);
    }
}

fn apply_transition(element: &HtmlElement, transition: &Transition) {
    let style_decl = element.style();
    let _ = style_decl.set_property("transition", &transition.transition_css());
}

fn animate_to(element: &Element, to: &FluidStyle, transition: &Transition) {
    let Some(html_element) = element.dyn_ref::<HtmlElement>() else {
        return;
    };

    apply_transition(html_element, transition);
    apply_style(element, to);
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn fluid_element_view(
    tag: &'static str,
    initial: FluidStyle,
    animate: FluidSignal<FluidStyle>,
    transition: Transition,
    reset: Signal<u32>,
    while_hover: Option<FluidStyle>,
    while_tap: Option<FluidStyle>,
    class: FluidSignal<String>,
    style: FluidSignal<String>,
    node_ref: FluidNodeRef,
    content: AnyView,
) -> impl IntoView {
    let is_hovered = RwSignal::new(false);
    let is_pressed = RwSignal::new(false);

    let base_style = StoredValue::new(initial.clone());
    let transition_store = StoredValue::new(transition);
    let initialized = StoredValue::new(false);
    let reset_last = StoredValue::new(reset.get_untracked());
    let last_element: StoredValue<Option<Element>, LocalStorage> = StoredValue::new_local(None);

    Effect::new({
        let initial = initial.clone();
        let animate = animate.clone();
        move || {
            let reset_value = reset.get();
            if reset_value != reset_last.get_value() {
                reset_last.set_value(reset_value);
                initialized.set_value(false);
            }
            let Some(element) = node_ref.get() else {
                return;
            };
            let element: Element = element.unchecked_into();
            let element_changed = match last_element.get_value() {
                Some(prev) => prev != element,
                None => true,
            };
            if element_changed {
                last_element.set_value(Some(element.clone()));
                initialized.set_value(false);
            }
            if initialized.get_value() {
                return;
            }
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
        let animate = animate.clone();
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
        let while_hover = while_hover.clone();
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
        let while_tap = while_tap.clone();
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

    let make_pointer_release = {
        let while_hover = while_hover.clone();
        move || {
            let while_hover = while_hover.clone();
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
        }
    };

    let on_pointerup = make_pointer_release();
    let on_pointercancel = make_pointer_release();
    let on_pointerleave = make_pointer_release();

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
pub fn FluidElement(
    /// HTML tag name for the underlying element (e.g. "div", "span").
    #[prop(default = "div")]
    tag: &'static str,
    /// Initial animated style applied on mount before the first transition.
    #[prop(default = FluidStyle::default())]
    initial: FluidStyle,
    /// Target style to animate to; updates reactively when this signal changes.
    #[prop(default = FluidSignal::static_value(FluidStyle::default()), into)]
    animate: FluidSignal<FluidStyle>,
    /// Transition settings for animated properties (duration/easing/spring).
    #[prop(default = Transition::default())]
    transition: Transition,
    /// Forces the initial animation to re-run when this counter changes.
    #[prop(default = Signal::derive(|| 0), into)]
    reset: Signal<u32>,
    /// Optional style applied while the pointer is hovering the element.
    #[prop(optional)]
    while_hover: Option<FluidStyle>,
    /// Optional style applied while the pointer is pressed down.
    #[prop(optional)]
    while_tap: Option<FluidStyle>,
    /// Class attribute (static or reactive).
    #[prop(default = FluidSignal::static_value(String::new()), into)]
    class: FluidSignal<String>,
    /// Extra CSS style string (non-animated); useful for layout/base styles.
    #[prop(default = FluidSignal::static_value(String::new()), into)]
    style: FluidSignal<String>,
    /// NodeRef for the underlying element; created automatically if omitted.
    #[prop(optional)]
    node_ref: Option<FluidNodeRef>,
    /// Child view(s) inside the fluid element.
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    let node_ref = node_ref.unwrap_or_default();
    let content = children.map(|c| c()).unwrap_or_else(|| ().into_any());
    fluid_element_view(
        tag,
        initial,
        animate,
        transition,
        reset,
        while_hover,
        while_tap,
        class,
        style,
        node_ref,
        content,
    )
}

macro_rules! fluid_wrapper {
    ($name:ident, $tag:literal) => {
        #[component]
        pub fn $name(
            /// Initial animated style applied on mount before the first transition.
            #[prop(default = FluidStyle::default())]
            initial: FluidStyle,
            /// Target style to animate to; updates reactively when this signal changes.
            #[prop(default = FluidSignal::static_value(FluidStyle::default()), into)]
            animate: FluidSignal<FluidStyle>,
            /// Transition settings for animated properties (duration/easing/spring).
            #[prop(default = Transition::default())]
            transition: Transition,
            /// Forces the initial animation to re-run when this counter changes.
            #[prop(default = Signal::derive(|| 0), into)]
            reset: Signal<u32>,
            /// Optional style applied while the pointer is hovering the element.
            #[prop(optional)]
            while_hover: Option<FluidStyle>,
            /// Optional style applied while the pointer is pressed down.
            #[prop(optional)]
            while_tap: Option<FluidStyle>,
            /// Class attribute (static or reactive).
            #[prop(default = FluidSignal::static_value(String::new()), into)]
            class: FluidSignal<String>,
            /// Extra CSS style string (non-animated); useful for layout/base styles.
            #[prop(default = FluidSignal::static_value(String::new()), into)]
            style: FluidSignal<String>,
            /// NodeRef for the underlying element; created automatically if omitted.
            #[prop(optional)]
            node_ref: Option<FluidNodeRef>,
            /// Child view(s) inside the fluid element.
            #[prop(optional)]
            children: Option<Children>,
        ) -> impl IntoView {
            let node_ref = node_ref.unwrap_or_default();
            let content = children.map(|c| c()).unwrap_or_else(|| ().into_any());
            fluid_element_view(
                $tag,
                initial,
                animate,
                transition,
                reset,
                while_hover,
                while_tap,
                class,
                style,
                node_ref,
                content,
            )
        }
    };
}

fluid_wrapper!(FluidDiv, "div");
fluid_wrapper!(FluidSpan, "span");
fluid_wrapper!(FluidButton, "button");
