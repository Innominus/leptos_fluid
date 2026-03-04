use std::rc::Rc;

use leptos::prelude::{
    Effect, GetValue, LocalStorage, ReadValue, RwSignal, Set, SetValue, Signal, StoredValue,
    WriteValue,
};
use web_sys::Element;

use crate::animator::{ActiveAnimation, animate_to, cancel_active_animation, set_immediate};
use crate::{FluidSignal, FluidStyle, Transition};

#[derive(Clone)]
enum AnimationTarget {
    Element(Element),
    Resolver(Rc<dyn Fn() -> Option<Element>>),
}

impl AnimationTarget {
    fn resolve(&self) -> Option<Element> {
        match self {
            Self::Element(element) => Some(element.clone()),
            Self::Resolver(resolver) => resolver(),
        }
    }
}

#[derive(Clone)]
struct PendingCommand {
    style: FluidStyle,
    transition: Option<Transition>,
    immediate: bool,
}

#[derive(Clone)]
struct AnimationControllerInner {
    default_transition: StoredValue<Transition>,
    target: StoredValue<Option<AnimationTarget>, LocalStorage>,
    pending_command: StoredValue<Option<PendingCommand>, LocalStorage>,
    active_animation: StoredValue<Option<ActiveAnimation>, LocalStorage>,
    animation_generation: StoredValue<u32, LocalStorage>,
    last_element: StoredValue<Option<Element>, LocalStorage>,
    is_animating: RwSignal<bool>,
}

/// Element-agnostic animation controller.
///
/// Attach a target element (or resolver) and then drive updates with
/// `animate`, `animate_with`, or `set_immediate`.
#[derive(Clone, Copy)]
pub struct AnimationController {
    inner: StoredValue<AnimationControllerInner>,
}

impl Default for AnimationController {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationController {
    pub fn new() -> Self {
        Self::with_transition(Transition::default())
    }

    pub fn with_transition(transition: Transition) -> Self {
        let inner = AnimationControllerInner {
            default_transition: StoredValue::new(transition),
            target: StoredValue::new_local(None),
            pending_command: StoredValue::new_local(None),
            active_animation: StoredValue::new_local(None),
            animation_generation: StoredValue::new_local(0),
            last_element: StoredValue::new_local(None),
            is_animating: RwSignal::new(false),
        };

        Self {
            inner: StoredValue::new(inner),
        }
    }

    pub fn set_transition(&self, transition: Transition) {
        self.inner
            .write_value()
            .default_transition
            .set_value(transition);
    }

    pub fn transition(&self) -> Transition {
        self.inner.read_value().default_transition.get_value()
    }

    pub fn is_animating(&self) -> Signal<bool> {
        self.inner.read_value().is_animating.into()
    }

    pub fn attach_element(&self, element: Element) {
        let inner = self.inner.get_value();
        let previous = inner.last_element.get_value();
        if let Some(previous) = previous
            && previous != element
        {
            cancel_active_animation(&previous, inner.active_animation);
            inner.is_animating.set(false);
        }

        inner
            .target
            .set_value(Some(AnimationTarget::Element(element.clone())));
        inner.last_element.set_value(Some(element));
        self.flush_pending();
    }

    pub fn attach_resolver<F>(&self, resolver: F)
    where
        F: Fn() -> Option<Element> + 'static,
    {
        self.inner
            .write_value()
            .target
            .set_value(Some(AnimationTarget::Resolver(Rc::new(resolver))));
        self.flush_pending();
    }

    pub fn clear_target(&self) {
        let inner = self.inner.get_value();
        if let Some(previous) = inner.last_element.get_value() {
            cancel_active_animation(&previous, inner.active_animation);
        }

        inner.target.set_value(None);
        inner.pending_command.set_value(None);
        inner.last_element.set_value(None);
        inner.is_animating.set(false);
    }

    pub fn animate(&self, style: FluidStyle) {
        self.execute_or_queue(PendingCommand {
            style,
            transition: None,
            immediate: false,
        });
    }

    pub fn animate_with(&self, style: FluidStyle, transition: Transition) {
        self.execute_or_queue(PendingCommand {
            style,
            transition: Some(transition),
            immediate: false,
        });
    }

    pub fn set_immediate(&self, style: FluidStyle) {
        self.execute_or_queue(PendingCommand {
            style,
            transition: None,
            immediate: true,
        });
    }

    pub fn stop(&self) {
        let inner = self.inner.get_value();
        inner.pending_command.set_value(None);

        if let Some(element) =
            resolve_target_element(&inner).or_else(|| inner.last_element.get_value())
        {
            cancel_active_animation(&element, inner.active_animation);
        } else {
            inner.active_animation.set_value(None);
        }

        let generation = inner.animation_generation.get_value().wrapping_add(1);
        inner.animation_generation.set_value(generation);
        inner.is_animating.set(false);
    }

    pub fn bind<T>(&self, style: T)
    where
        T: Into<FluidSignal<FluidStyle>>,
    {
        let style = style.into();
        let controller = *self;
        Effect::new(move || {
            controller.animate(style.get());
        });
    }

    pub fn bind_with<T>(&self, style: T, transition: Transition)
    where
        T: Into<FluidSignal<FluidStyle>>,
    {
        let style = style.into();
        let controller = *self;
        Effect::new(move || {
            controller.animate_with(style.get(), transition.clone());
        });
    }

    fn execute_or_queue(&self, command: PendingCommand) {
        let inner = self.inner.get_value();
        let Some(element) = resolve_target_element(&inner) else {
            inner.pending_command.set_value(Some(command));
            return;
        };

        inner.pending_command.set_value(None);

        if command.immediate {
            set_immediate(
                &element,
                &command.style,
                inner.active_animation,
                inner.animation_generation,
                Some(inner.is_animating),
            );
            return;
        }

        let transition = command
            .transition
            .unwrap_or_else(|| inner.default_transition.get_value());
        animate_to(
            &element,
            &command.style,
            &transition,
            inner.active_animation,
            inner.animation_generation,
            Some(inner.is_animating),
        );
    }

    fn flush_pending(&self) {
        let Some(command) = self.inner.read_value().pending_command.get_value() else {
            return;
        };
        self.execute_or_queue(command);
    }
}

fn resolve_target_element(inner: &AnimationControllerInner) -> Option<Element> {
    let target = inner.target.get_value()?;
    let element = target.resolve()?;

    if let Some(previous) = inner.last_element.get_value()
        && previous != element
    {
        cancel_active_animation(&previous, inner.active_animation);
    }
    inner.last_element.set_value(Some(element.clone()));

    Some(element)
}
