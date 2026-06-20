use std::ops::Deref;
use std::rc::Rc;

use leptos::html::ElementType;
use leptos::prelude::{
    Effect, GetUntracked, GetValue, LocalStorage, NodeRef, RwSignal, Set, SetValue, StoredValue,
    on_cleanup,
};
use leptos::wasm_bindgen::JsCast;
use web_sys::Element;
use web_sys::wasm_bindgen::JsValue;
use web_sys::wasm_bindgen::closure::Closure;

use crate::{AnimationController, FluidSignal, FluidStyle};

#[derive(Clone, Copy)]
struct InteractionState {
    hovered: RwSignal<bool>,
    pressed: RwSignal<bool>,
}

impl InteractionState {
    fn new() -> Self {
        Self {
            hovered: RwSignal::new(false),
            pressed: RwSignal::new(false),
        }
    }

    fn is_interacting(&self) -> bool {
        self.hovered.get_untracked() || self.pressed.get_untracked()
    }

    fn set_hovered(&self, hovered: bool) {
        self.hovered.set(hovered);
        if !hovered {
            self.pressed.set(false);
        }
    }

    fn press(&self) {
        self.pressed.set(true);
    }

    fn release(&self) -> bool {
        if !self.pressed.get_untracked() {
            return false;
        }

        self.pressed.set(false);
        true
    }

    fn target(
        &self,
        base: &FluidStyle,
        while_hover: Option<&FluidStyle>,
        while_tap: Option<&FluidStyle>,
    ) -> FluidStyle {
        resolve_interaction_target(
            base,
            while_hover,
            while_tap,
            self.hovered.get_untracked(),
            self.pressed.get_untracked(),
        )
    }
}

fn resolve_interaction_target(
    base: &FluidStyle,
    while_hover: Option<&FluidStyle>,
    while_tap: Option<&FluidStyle>,
    hovered: bool,
    pressed: bool,
) -> FluidStyle {
    if pressed {
        if let Some(tap) = while_tap {
            return tap.clone();
        }
        if hovered && let Some(hover) = while_hover {
            return hover.clone();
        }
        return base.clone();
    }

    if hovered && let Some(hover) = while_hover {
        return hover.clone();
    }

    base.clone()
}

#[derive(Clone)]
struct InteractionListeners {
    enter: Rc<Closure<dyn FnMut(JsValue)>>,
    leave: Rc<Closure<dyn FnMut(JsValue)>>,
    down: Rc<Closure<dyn FnMut(JsValue)>>,
    up: Rc<Closure<dyn FnMut(JsValue)>>,
    cancel: Rc<Closure<dyn FnMut(JsValue)>>,
    element: Element,
}

impl InteractionListeners {
    fn detach(&self) {
        let _ = self.element.remove_event_listener_with_callback(
            "pointerenter",
            self.enter.deref().as_ref().unchecked_ref(),
        );
        let _ = self.element.remove_event_listener_with_callback(
            "pointerleave",
            self.leave.deref().as_ref().unchecked_ref(),
        );
        let _ = self.element.remove_event_listener_with_callback(
            "pointerdown",
            self.down.deref().as_ref().unchecked_ref(),
        );
        let _ = self.element.remove_event_listener_with_callback(
            "pointerup",
            self.up.deref().as_ref().unchecked_ref(),
        );
        let _ = self.element.remove_event_listener_with_callback(
            "pointercancel",
            self.cancel.deref().as_ref().unchecked_ref(),
        );
    }
}

#[derive(Clone)]
struct InteractionBindingInner {
    controller: AnimationController,
    interactions: InteractionState,
    base_style: StoredValue<FluidStyle>,
    while_hover: Option<FluidStyle>,
    while_tap: Option<FluidStyle>,
    attached_element: StoredValue<Option<Element>, LocalStorage>,
    listeners: StoredValue<Option<InteractionListeners>, LocalStorage>,
}

impl InteractionBindingInner {
    fn update_base(&self, base: FluidStyle) {
        self.base_style.set_value(base.clone());

        if self.interactions.is_interacting() {
            return;
        }

        let transition = self.controller.transition();
        self.controller.animate_with(base, transition);
    }

    fn attach(&self, element: Option<Element>) {
        let previous = self.attached_element.get_value();
        if previous == element {
            return;
        }

        if let Some(old) = self.listeners.get_value() {
            old.detach();
            self.listeners.set_value(None);
        }

        self.attached_element.set_value(element.clone());

        let Some(element) = element else {
            return;
        };

        let while_hover = self.while_hover.clone();
        let while_tap = self.while_tap.clone();

        if while_hover.is_none() && while_tap.is_none() {
            return;
        }

        let controller = self.controller;
        let interactions = self.interactions;
        let base_style = self.base_style;

        let enter = {
            let while_hover = while_hover.clone();
            let while_tap = while_tap.clone();
            Rc::new(Closure::wrap(Box::new(move |_: JsValue| {
                if while_hover.is_none() {
                    return;
                }
                interactions.set_hovered(true);
                animate_interaction_inline(
                    controller,
                    interactions,
                    base_style,
                    while_hover.as_ref(),
                    while_tap.as_ref(),
                );
            }) as Box<dyn FnMut(JsValue)>))
        };

        let leave = {
            let while_hover = while_hover.clone();
            let while_tap = while_tap.clone();
            Rc::new(Closure::wrap(Box::new(move |_: JsValue| {
                interactions.set_hovered(false);
                animate_interaction_inline(
                    controller,
                    interactions,
                    base_style,
                    while_hover.as_ref(),
                    while_tap.as_ref(),
                );
            }) as Box<dyn FnMut(JsValue)>))
        };

        let down = {
            let while_hover = while_hover.clone();
            let while_tap = while_tap.clone();
            Rc::new(Closure::wrap(Box::new(move |_: JsValue| {
                if while_tap.is_none() {
                    return;
                }
                interactions.press();
                animate_interaction_inline(
                    controller,
                    interactions,
                    base_style,
                    while_hover.as_ref(),
                    while_tap.as_ref(),
                );
            }) as Box<dyn FnMut(JsValue)>))
        };

        let make_release = || {
            let while_hover = while_hover.clone();
            let while_tap = while_tap.clone();
            Rc::new(Closure::wrap(Box::new(move |_: JsValue| {
                if !interactions.release() {
                    return;
                }
                animate_interaction_inline(
                    controller,
                    interactions,
                    base_style,
                    while_hover.as_ref(),
                    while_tap.as_ref(),
                );
            }) as Box<dyn FnMut(JsValue)>))
        };

        let up = make_release();
        let cancel = make_release();

        let _ = element.add_event_listener_with_callback(
            "pointerenter",
            enter.deref().as_ref().unchecked_ref(),
        );
        let _ = element.add_event_listener_with_callback(
            "pointerleave",
            leave.deref().as_ref().unchecked_ref(),
        );
        let _ = element
            .add_event_listener_with_callback("pointerdown", down.deref().as_ref().unchecked_ref());
        let _ = element
            .add_event_listener_with_callback("pointerup", up.deref().as_ref().unchecked_ref());
        let _ = element.add_event_listener_with_callback(
            "pointercancel",
            cancel.deref().as_ref().unchecked_ref(),
        );

        self.listeners.set_value(Some(InteractionListeners {
            enter,
            leave,
            down,
            up,
            cancel,
            element: element.clone(),
        }));
    }
}

fn animate_interaction_inline(
    controller: AnimationController,
    interactions: InteractionState,
    base_style: StoredValue<FluidStyle>,
    while_hover: Option<&FluidStyle>,
    while_tap: Option<&FluidStyle>,
) {
    let base = base_style.get_value();
    let target = interactions.target(&base, while_hover, while_tap);
    let transition = controller.transition();
    controller.animate_with(target, transition);
}

/// A `Copy` handle to an interaction binding, mirroring [`AnimationController`]'s
/// ergonomics so it can be captured in `Send + Sync` cleanup closures.
#[derive(Clone, Copy)]
struct InteractionHandle {
    inner: StoredValue<InteractionBindingInner>,
}

impl InteractionHandle {
    fn new(
        controller: AnimationController,
        while_hover: Option<FluidStyle>,
        while_tap: Option<FluidStyle>,
    ) -> Self {
        let inner = InteractionBindingInner {
            controller,
            interactions: InteractionState::new(),
            base_style: StoredValue::new(FluidStyle::new()),
            while_hover,
            while_tap,
            attached_element: StoredValue::new_local(None),
            listeners: StoredValue::new_local(None),
        };
        Self {
            inner: StoredValue::new(inner),
        }
    }

    fn update_base(&self, base: FluidStyle) {
        self.inner.get_value().update_base(base);
    }

    fn attach(&self, element: Option<Element>) {
        self.inner.get_value().attach(element);
    }
}

/// Declaratively binds hover/tap interaction styles to a controller-managed
/// element.
///
/// `base` is the reactive resting style for the element. When the pointer is
/// hovering or pressed, `while_hover` / `while_tap` override the base and the
/// controller animates to the resolved target. When the interaction ends, the
/// controller animates back to the current `base`.
///
/// The resolver is queried reactively; listeners are reinstalled whenever the
/// resolved element changes and cleaned up on scope disposal.
pub fn bind_interaction<F, S>(
    controller: AnimationController,
    resolver: F,
    base: S,
    while_hover: Option<FluidStyle>,
    while_tap: Option<FluidStyle>,
) where
    F: Fn() -> Option<Element> + 'static,
    S: Into<FluidSignal<FluidStyle>>,
{
    let base_signal = base.into();
    let handle = InteractionHandle::new(controller, while_hover, while_tap);

    Effect::new(move || {
        let next = base_signal.get();
        handle.update_base(next);
    });

    Effect::new(move || {
        let element = resolver();
        handle.attach(element);
    });

    on_cleanup(move || {
        handle.attach(None);
    });
}

/// Convenience wrapper for [`bind_interaction`] that resolves a `NodeRef`.
pub fn bind_interaction_node_ref<E, S>(
    controller: AnimationController,
    node_ref: NodeRef<E>,
    base: S,
    while_hover: Option<FluidStyle>,
    while_tap: Option<FluidStyle>,
) where
    E: ElementType,
    E::Output: JsCast + Clone + 'static,
    S: Into<FluidSignal<FluidStyle>>,
{
    bind_interaction(
        controller,
        move || node_ref.get_untracked().map(|node| node.unchecked_into()),
        base,
        while_hover,
        while_tap,
    )
}

#[cfg(test)]
mod tests {
    use super::resolve_interaction_target;
    use crate::FluidStyle;

    fn named_style(name: &'static str) -> FluidStyle {
        FluidStyle::new().with("background", name)
    }

    #[test]
    fn interaction_target_prefers_press_over_hover() {
        let base = named_style("base");
        let hover = named_style("hover");
        let tap = named_style("tap");

        assert_eq!(
            resolve_interaction_target(&base, Some(&hover), Some(&tap), false, false),
            base
        );
        assert_eq!(
            resolve_interaction_target(&base, Some(&hover), Some(&tap), true, false),
            hover
        );
        assert_eq!(
            resolve_interaction_target(&base, Some(&hover), Some(&tap), true, true),
            tap
        );
    }

    #[test]
    fn interaction_target_falls_back_when_variants_are_missing() {
        let base = named_style("base");
        let hover = named_style("hover");

        assert_eq!(
            resolve_interaction_target(&base, Some(&hover), None, true, true),
            hover
        );
        assert_eq!(
            resolve_interaction_target(&base, None, None, true, true),
            base
        );
    }
}
