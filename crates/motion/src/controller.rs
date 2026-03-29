use std::rc::Rc;

use leptos::html::ElementType;
use leptos::prelude::{
    Effect, GetUntracked, GetValue, LocalStorage, NodeRef, ReadValue, RwSignal, Set, SetValue,
    Signal, StoredValue, WriteValue,
};
use leptos::wasm_bindgen::JsCast;
use web_sys::Element;

use crate::animator::{
    ActiveAnimation, animate_to, cancel_active_animation, pause_active_animation,
    resume_active_animation, set_immediate,
};
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

/// A stable target that can be attached to an [`AnimationController`].
///
/// This is intentionally limited to concrete elements and `NodeRef`s. Dynamic
/// lookup belongs to [`AnimationController::attach_resolver`].
pub trait ControllerTarget {
    fn attach_to(self, controller: AnimationController);
}

impl ControllerTarget for Element {
    fn attach_to(self, controller: AnimationController) {
        controller.attach_element(self);
    }
}

impl<E> ControllerTarget for NodeRef<E>
where
    E: ElementType,
    E::Output: JsCast + Clone + 'static,
{
    fn attach_to(self, controller: AnimationController) {
        controller.attach_node_ref(self);
    }
}

#[derive(Clone)]
struct AnimationCommand {
    style: FluidStyle,
    transition: Option<Transition>,
    mode: CommandMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CommandMode {
    Animate,
    Immediate,
}

impl AnimationCommand {
    fn animate(style: FluidStyle) -> Self {
        Self {
            style,
            transition: None,
            mode: CommandMode::Animate,
        }
    }

    fn animate_with(style: FluidStyle, transition: Transition) -> Self {
        Self {
            style,
            transition: Some(transition),
            mode: CommandMode::Animate,
        }
    }

    fn immediate(style: FluidStyle) -> Self {
        Self {
            style,
            transition: None,
            mode: CommandMode::Immediate,
        }
    }
}

#[derive(Clone)]
struct AnimationControllerInner {
    default_transition: StoredValue<Transition>,
    target_source: StoredValue<Option<AnimationTarget>, LocalStorage>,
    queued_command: StoredValue<Option<AnimationCommand>, LocalStorage>,
    active_animation: StoredValue<Option<ActiveAnimation>, LocalStorage>,
    animation_generation: StoredValue<u32, LocalStorage>,
    resolved_target: StoredValue<Option<Element>, LocalStorage>,
    is_animating: RwSignal<bool>,
}

impl AnimationControllerInner {
    fn queue_latest(&self, command: AnimationCommand) {
        self.queued_command.set_value(Some(command));
    }

    fn clear_queued_command(&self) {
        self.queued_command.set_value(None);
    }

    fn bump_generation(&self) -> u32 {
        let generation = self.animation_generation.get_value().wrapping_add(1);
        self.animation_generation.set_value(generation);
        generation
    }

    fn sync_resolved_target(&self, next: Option<Element>) {
        let previous = self.resolved_target.get_value();
        if previous == next {
            return;
        }

        if let Some(previous) = previous {
            cancel_active_animation(&previous, self.active_animation);
        } else {
            self.active_animation.set_value(None);
        }

        self.resolved_target.set_value(next);
        self.is_animating.set(false);
    }

    fn detach_resolved_target(&self) {
        self.sync_resolved_target(None);
    }

    fn set_target_source(&self, target: Option<AnimationTarget>) {
        match &target {
            Some(AnimationTarget::Element(element)) => {
                self.sync_resolved_target(Some(element.clone()));
            }
            Some(AnimationTarget::Resolver(_)) | None => {
                self.detach_resolved_target();
            }
        }

        self.target_source.set_value(target);
    }

    fn resolve_target(&self) -> Option<Element> {
        let target = self.target_source.get_value()?;
        let Some(element) = target.resolve() else {
            self.detach_resolved_target();
            return None;
        };

        self.sync_resolved_target(Some(element.clone()));
        Some(element)
    }

    fn execute_on(&self, element: &Element, command: &AnimationCommand) {
        if command.mode == CommandMode::Immediate {
            set_immediate(
                element,
                &command.style,
                self.active_animation,
                self.animation_generation,
                Some(self.is_animating),
            );
            return;
        }

        let transition = command
            .transition
            .clone()
            .unwrap_or_else(|| self.default_transition.get_value());
        animate_to(
            element,
            &command.style,
            &transition,
            self.active_animation,
            self.animation_generation,
            Some(self.is_animating),
        );
    }
}

/// Element-agnostic animation controller.
///
/// `AnimationController` separates *what* to animate (`FluidStyle`) from
/// *where* to animate (a concrete `Element` or a resolver closure).
///
/// Typical flow:
///
/// 1. Create a controller (`new` / `with_transition`).
/// 2. Attach a target (`attach_node_ref`, `attach_element`, or `attach_resolver`).
/// 3. Drive updates with `animate`, `animate_with`, or `set_immediate`.
///
/// If an animation command is issued before a target is available, the
/// controller stores only the latest pending command and replays it when a
/// target resolves.
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
    /// Creates a controller with [`Transition::default()`] as the fallback
    /// transition for [`Self::animate`].
    pub fn new() -> Self {
        Self::with_transition(Transition::default())
    }

    /// Creates a controller with a caller-provided default transition.
    ///
    /// This transition is used by [`Self::animate`]. Use
    /// [`Self::animate_with`] for per-call overrides.
    pub fn with_transition(transition: Transition) -> Self {
        let inner = AnimationControllerInner {
            default_transition: StoredValue::new(transition),
            target_source: StoredValue::new_local(None),
            queued_command: StoredValue::new_local(None),
            active_animation: StoredValue::new_local(None),
            animation_generation: StoredValue::new_local(0),
            resolved_target: StoredValue::new_local(None),
            is_animating: RwSignal::new(false),
        };

        Self {
            inner: StoredValue::new(inner),
        }
    }

    /// Replaces the controller's default transition.
    ///
    /// This affects future [`Self::animate`] calls. Ongoing animations are not
    /// modified retroactively.
    pub fn set_transition(&self, transition: Transition) {
        self.inner
            .write_value()
            .default_transition
            .set_value(transition);
    }

    /// Returns the current default transition used by [`Self::animate`].
    pub fn transition(&self) -> Transition {
        self.inner.read_value().default_transition.get_value()
    }

    /// Reactive signal indicating whether the controller currently has an
    /// active WAAPI animation.
    ///
    /// The signal flips to `false` for immediate style application and after
    /// animation completion/cancellation.
    pub fn is_animating(&self) -> Signal<bool> {
        self.inner.read_value().is_animating.into()
    }

    /// Attaches a stable controller target.
    pub fn attach_target<T>(&self, target: T)
    where
        T: ControllerTarget,
    {
        target.attach_to(*self);
    }

    /// Attaches any stable controller target.
    pub fn attach<T>(&self, target: T)
    where
        T: ControllerTarget,
    {
        self.attach_target(target);
    }

    /// Attaches a concrete DOM element target.
    ///
    /// If another element was previously attached, its active animation is
    /// canceled and committed before switching to the new target.
    ///
    /// If a pending command exists (queued while detached), it is immediately
    /// flushed against this element.
    pub fn attach_element(&self, element: Element) {
        self.inner
            .get_value()
            .set_target_source(Some(AnimationTarget::Element(element)));
        self.flush_pending();
    }

    /// Attaches a `NodeRef` and resolves it on each command.
    ///
    /// This is the usual controller entry point for plain Leptos elements and
    /// avoids repetitive `attach_resolver + unchecked_into()` boilerplate.
    pub fn attach_node_ref<E>(&self, node_ref: NodeRef<E>)
    where
        E: ElementType,
        E::Output: JsCast + Clone + 'static,
    {
        self.attach_resolver(move || node_ref.get_untracked().map(|node| node.unchecked_into()));
    }

    /// Attaches a resolver closure that returns the current target element.
    ///
    /// This is useful with `NodeRef`-driven lifecycles where the element may
    /// not exist yet during initial setup.
    ///
    /// The resolver is queried on each command execution. If it returns `None`,
    /// the command is stored as the latest pending command.
    pub fn attach_resolver<F>(&self, resolver: F)
    where
        F: Fn() -> Option<Element> + 'static,
    {
        self.inner
            .get_value()
            .set_target_source(Some(AnimationTarget::Resolver(Rc::new(resolver))));
        self.flush_pending();
    }

    /// Detaches the current target and clears queued work.
    ///
    /// This cancels any active animation on the last resolved element and
    /// resets `is_animating` to `false`.
    pub fn clear_target(&self) {
        let inner = self.inner.get_value();
        inner.bump_generation();
        inner.set_target_source(None);
        inner.clear_queued_command();
        inner.is_animating.set(false);
    }

    /// Animates to a style using the controller's default transition.
    ///
    /// If no target can be resolved, this command becomes the latest pending
    /// command and will be replayed on the next successful attachment.
    pub fn animate(&self, style: FluidStyle) {
        self.execute_or_queue(AnimationCommand::animate(style));
    }

    /// Animates to a style using a call-specific transition override.
    ///
    /// This does not mutate the controller's default transition.
    pub fn animate_with(&self, style: FluidStyle, transition: Transition) {
        self.execute_or_queue(AnimationCommand::animate_with(style, transition));
    }

    /// Applies style immediately without tweening.
    ///
    /// If an animation is in flight, it is canceled and committed before the
    /// new style is applied.
    ///
    /// If no target can be resolved, this command is queued as the latest
    /// pending command.
    pub fn set_immediate(&self, style: FluidStyle) {
        self.execute_or_queue(AnimationCommand::immediate(style));
    }

    /// Stops current animation work and clears queued commands.
    ///
    /// The controller keeps the current visual state (no automatic reset) and
    /// bumps internal generation counters so stale callbacks are ignored.
    pub fn stop(&self) {
        let inner = self.inner.get_value();
        inner.clear_queued_command();

        if let Some(element) = inner
            .resolve_target()
            .or_else(|| inner.resolved_target.get_value())
        {
            cancel_active_animation(&element, inner.active_animation);
        } else {
            inner.active_animation.set_value(None);
        }

        inner.bump_generation();
        inner.is_animating.set(false);
    }

    /// Pauses the active WAAPI animation if one is currently attached.
    pub fn pause(&self) -> bool {
        let inner = self.inner.get_value();
        pause_active_animation(inner.active_animation)
    }

    /// Resumes the active WAAPI animation if one is currently attached.
    pub fn resume(&self) -> bool {
        let inner = self.inner.get_value();
        resume_active_animation(inner.active_animation)
    }

    /// Declaratively binds a reactive style source to [`Self::animate`].
    ///
    /// Internally this creates a Leptos `Effect` that runs whenever the source
    /// updates. Effect lifetime follows the current reactive owner scope.
    pub fn bind<T>(&self, style: T)
    where
        T: Into<FluidSignal<FluidStyle>>,
    {
        self.bind_signal(style.into(), None, false);
    }

    /// Declaratively binds a reactive style source to
    /// [`Self::animate_with`] using a fixed transition.
    ///
    /// The transition is cloned per update; pass lightweight transition values
    /// or precompute as needed.
    pub fn bind_with<T>(&self, style: T, transition: Transition)
    where
        T: Into<FluidSignal<FluidStyle>>,
    {
        self.bind_signal(style.into(), Some(transition), false);
    }

    /// Binds a reactive style source with a fixed transition and applies the
    /// first value immediately.
    pub fn bind_with_immediate<T>(&self, style: T, transition: Transition)
    where
        T: Into<FluidSignal<FluidStyle>>,
    {
        self.bind_signal(style.into(), Some(transition), true);
    }

    /// Binds a reactive style source and applies the first value immediately.
    ///
    /// This is useful for controller-first declarative APIs where the current
    /// state should become the baseline without an initial tween.
    pub fn bind_immediate<T>(&self, style: T)
    where
        T: Into<FluidSignal<FluidStyle>>,
    {
        self.bind_signal(style.into(), None, true);
    }

    #[inline(never)]
    fn bind_signal(
        &self,
        style: FluidSignal<FluidStyle>,
        transition: Option<Transition>,
        immediate_first: bool,
    ) {
        let controller = *self;
        let initialized: StoredValue<bool, LocalStorage> = StoredValue::new_local(!immediate_first);
        Effect::new(move || {
            let next = style.get();
            if initialized.get_value() {
                if let Some(transition) = transition.as_ref() {
                    controller.animate_with(next, transition.clone());
                } else {
                    controller.animate(next);
                }
            } else {
                controller.set_immediate(next);
                initialized.set_value(true);
            }
        });
    }

    fn execute_or_queue(&self, command: AnimationCommand) {
        let inner = self.inner.get_value();
        let Some(element) = inner.resolve_target() else {
            inner.queue_latest(command);
            return;
        };

        inner.clear_queued_command();
        inner.execute_on(&element, &command);
    }

    fn flush_pending(&self) {
        let Some(command) = self.inner.read_value().queued_command.get_value() else {
            return;
        };
        self.execute_or_queue(command);
    }
}
