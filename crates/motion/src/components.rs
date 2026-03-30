use leptos::prelude::*;
use web_sys::Element;
use web_sys::wasm_bindgen::JsCast;

use crate::{AnimationController, FluidSignal, FluidStyle, Transition};

/// Node reference type used by motion elements.
pub type FluidNodeRef = NodeRef<leptos::html::Custom<&'static str>>;

#[derive(Clone, Copy)]
struct FluidElementLifecycle {
    initialized: StoredValue<bool>,
    reset_last: StoredValue<u32>,
    init_generation: StoredValue<u32>,
    mounted_element: StoredValue<Option<Element>, LocalStorage>,
}

impl FluidElementLifecycle {
    fn new(reset_value: u32) -> Self {
        Self {
            initialized: StoredValue::new(false),
            reset_last: StoredValue::new(reset_value),
            init_generation: StoredValue::new(0),
            mounted_element: StoredValue::new_local(None),
        }
    }

    fn prepare_mount(&self, reset_value: u32, element: &Element) -> bool {
        let mut needs_init = !self.initialized.get_value();

        if reset_value != self.reset_last.get_value() {
            self.reset_last.set_value(reset_value);
            self.invalidate();
            needs_init = true;
        }

        let element_changed = match self.mounted_element.get_value() {
            Some(previous) => previous != *element,
            None => true,
        };
        if element_changed {
            self.mounted_element.set_value(Some(element.clone()));
            self.invalidate();
            needs_init = true;
        }

        needs_init
    }

    fn mark_unmounted(&self) {
        if self.mounted_element.get_value().is_none() {
            return;
        }

        self.mounted_element.set_value(None);
        self.invalidate();
    }

    fn current_generation(&self) -> u32 {
        self.init_generation.get_value()
    }

    fn finish_initialization(&self) {
        self.initialized.set_value(true);
    }

    fn is_initialized(&self) -> bool {
        self.initialized.get_value()
    }

    fn invalidate(&self) {
        let next_generation = self.init_generation.get_value().wrapping_add(1);
        self.init_generation.set_value(next_generation);
        self.initialized.set_value(false);
    }
}

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

fn plan_mount_targets(
    initial: &FluidStyle,
    animate: &FluidStyle,
) -> (FluidStyle, Option<FluidStyle>) {
    if animate.is_empty() {
        (initial.clone(), None)
    } else {
        (animate.clone(), Some(animate.clone()))
    }
}

fn animate_interaction_target(
    controller: AnimationController,
    transition_store: StoredValue<Transition>,
    base_style: StoredValue<FluidStyle>,
    interactions: InteractionState,
    while_hover: Option<FluidStyle>,
    while_tap: Option<FluidStyle>,
) {
    let base = base_style.get_value();
    let target = interactions.target(&base, while_hover.as_ref(), while_tap.as_ref());
    controller.animate_with(target, transition_store.get_value());
}

#[allow(clippy::too_many_arguments)]
#[inline(never)]
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
) -> AnyView {
    let interactions = InteractionState::new();
    let controller = AnimationController::with_transition(transition.clone());
    let base_style = StoredValue::new(initial.clone());
    let transition_store = StoredValue::new(transition);
    let lifecycle = FluidElementLifecycle::new(reset.get_untracked());

    controller.attach_node_ref(node_ref);

    Effect::new({
        let animate = animate.clone();
        move || {
            let reset_value = reset.get();
            let Some(element) = node_ref.get() else {
                lifecycle.mark_unmounted();
                return;
            };
            let element: Element = element.unchecked_into();

            if !lifecycle.prepare_mount(reset_value, &element) {
                return;
            }

            controller.stop();
            if !initial.is_empty() {
                controller.set_immediate(initial.clone());
            }

            let target = animate.get();
            let (next_base, scheduled_target) = plan_mount_targets(&initial, &target);
            base_style.set_value(next_base);

            if let Some(target) = scheduled_target {
                let transition = transition_store.get_value();
                let generation = lifecycle.current_generation();
                let init_generation = lifecycle.init_generation;
                request_animation_frame(move || {
                    if init_generation.get_value() != generation {
                        return;
                    }
                    controller.animate_with(target, transition);
                });
            }

            lifecycle.finish_initialization();
        }
    });

    Effect::new(move || {
        let target = animate.get();

        if !lifecycle.is_initialized() {
            return;
        }

        if target.is_empty() {
            return;
        }
        base_style.set_value(target.clone());

        if interactions.is_interacting() {
            return;
        }

        let transition = transition_store.get_value();
        controller.animate_with(target, transition);
    });

    let on_pointerenter = {
        let while_hover = while_hover.clone();
        let while_tap = while_tap.clone();
        move |_| {
            if while_hover.is_none() {
                return;
            }

            interactions.set_hovered(true);
            animate_interaction_target(
                controller,
                transition_store,
                base_style,
                interactions,
                while_hover.clone(),
                while_tap.clone(),
            );
        }
    };

    let on_pointerleave = {
        let while_hover = while_hover.clone();
        let while_tap = while_tap.clone();
        move |_| {
            interactions.set_hovered(false);
            animate_interaction_target(
                controller,
                transition_store,
                base_style,
                interactions,
                while_hover.clone(),
                while_tap.clone(),
            );
        }
    };

    let on_pointerdown = {
        let while_hover = while_hover.clone();
        let while_tap = while_tap.clone();
        move |_| {
            if while_tap.is_none() {
                return;
            }

            interactions.press();
            animate_interaction_target(
                controller,
                transition_store,
                base_style,
                interactions,
                while_hover.clone(),
                while_tap.clone(),
            );
        }
    };

    let make_pointer_release = move || {
        let while_hover = while_hover.clone();
        let while_tap = while_tap.clone();
        move |_| {
            if !interactions.release() {
                return;
            }

            animate_interaction_target(
                controller,
                transition_store,
                base_style,
                interactions,
                while_hover.clone(),
                while_tap.clone(),
            );
        }
    };

    let on_pointerup = make_pointer_release();
    let on_pointercancel = make_pointer_release();

    on_cleanup(move || {
        controller.clear_target();
    });

    leptos::html::custom(tag)
        .node_ref(node_ref)
        .class(move || class.get())
        .style(move || style.get())
        .on(leptos::ev::pointerenter, on_pointerenter)
        .on(leptos::ev::pointerleave, on_pointerleave)
        .on(leptos::ev::pointerdown, on_pointerdown)
        .on(leptos::ev::pointerup, on_pointerup)
        .on(leptos::ev::pointercancel, on_pointercancel)
        .child(content)
        .into_any()
}

#[cfg(test)]
mod tests {
    use super::{plan_mount_targets, resolve_interaction_target};
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

    #[test]
    fn mount_plan_uses_initial_when_animate_is_empty() {
        let initial = named_style("initial");
        let animate = FluidStyle::new();

        let (base, scheduled) = plan_mount_targets(&initial, &animate);
        assert_eq!(base, initial);
        assert_eq!(scheduled, None);
    }

    #[test]
    fn mount_plan_schedules_non_empty_target() {
        let initial = named_style("initial");
        let animate = named_style("animate");

        let (base, scheduled) = plan_mount_targets(&initial, &animate);
        assert_eq!(base, animate);
        assert_eq!(scheduled, Some(animate));
    }
}

/// Motion-enabled element component for arbitrary HTML tags.
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
    ///
    /// Spring transitions only interpolate supported numeric properties.
    /// Unsupported properties are applied immediately and do not interpolate in
    /// spring mode.
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
) -> AnyView {
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

#[cfg(feature = "wrappers")]
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
            ///
            /// Spring transitions only interpolate supported numeric properties.
            /// Unsupported properties are applied immediately and do not interpolate in
            /// spring mode.
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
        ) -> AnyView {
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

#[cfg(feature = "wrappers")]
fluid_wrapper!(FluidDiv, "div");
#[cfg(feature = "wrappers")]
fluid_wrapper!(FluidSpan, "span");
#[cfg(feature = "wrappers")]
fluid_wrapper!(FluidButton, "button");
