//! Typed builder for [`ScrollTrigger`].
//!
//! Mirrors `AnimationControllerBuilder<State>` in `crates/motion/src/builders.rs`:
//! a `PhantomData<State>`-parameterized struct that transitions from
//! [`ScrollTriggerBuilderNeedsTarget`] to [`ScrollTriggerBuilderReady`] once a
//! target (or resolver) is attached. Config and callback setters are available in
//! any state; motion bindings are feature-gated. [`ReadyScrollTriggerBuilder::install`]
//! finalizes the trigger via [`ScrollTrigger::with_config`] (which registers with
//! the engine and installs `on_cleanup`), then attaches the deferred target and
//! any motion bindings, and finally runs [`ScrollTrigger::refresh`] so the
//! freshly-attached target's geometry is measured.

use std::marker::PhantomData;

use web_sys::Element;

use crate::config::{ScrollTriggerConfig, Scrub, ToggleActions};
use crate::trigger::{ScrollTrigger, TriggerTargetSource};

type TargetInstaller = Box<dyn FnOnce(ScrollTrigger)>;
#[cfg(feature = "controller")]
type ControllerInstaller = Box<dyn FnOnce(&ScrollTrigger)>;
#[cfg(feature = "timeline")]
type TimelineInstaller = Box<dyn FnOnce(&ScrollTrigger)>;

struct ScrollTriggerBuilderState {
    config: ScrollTriggerConfig,
    target: Option<TargetInstaller>,
    #[cfg(feature = "controller")]
    controller_binding: Option<ControllerInstaller>,
    #[cfg(feature = "timeline")]
    timeline_binding: Option<TimelineInstaller>,
}

#[doc(hidden)]
pub struct ScrollTriggerBuilderNeedsTarget;

#[doc(hidden)]
pub struct ScrollTriggerBuilderReady;

/// Typed builder for [`ScrollTrigger`].
///
/// Starts in the [`ScrollTriggerBuilderNeedsTarget`] state. Call
/// [`ScrollTriggerBuilder::target`] or [`ScrollTriggerBuilder::resolver`] to
/// attach a target and transition to [`ReadyScrollTriggerBuilder`], then
/// [`ReadyScrollTriggerBuilder::install`] to finalize.
pub struct ScrollTriggerBuilder<State = ScrollTriggerBuilderNeedsTarget> {
    state: ScrollTriggerBuilderState,
    _marker: PhantomData<State>,
}

/// Ready-to-install builder with a target attached.
pub type ReadyScrollTriggerBuilder = ScrollTriggerBuilder<ScrollTriggerBuilderReady>;

impl ScrollTrigger {
    /// Starts building a typed scroll trigger configuration.
    pub fn builder() -> ScrollTriggerBuilder {
        ScrollTriggerBuilder {
            state: ScrollTriggerBuilderState {
                config: ScrollTriggerConfig::new(),
                target: None,
                #[cfg(feature = "controller")]
                controller_binding: None,
                #[cfg(feature = "timeline")]
                timeline_binding: None,
            },
            _marker: PhantomData,
        }
    }
}

impl<State> ScrollTriggerBuilder<State> {
    #[inline]
    fn map<Next>(self, f: impl FnOnce(&mut ScrollTriggerBuilderState)) -> ScrollTriggerBuilder<Next> {
        let mut state = self.state;
        f(&mut state);
        ScrollTriggerBuilder {
            state,
            _marker: PhantomData,
        }
    }

    /// Sets the `start` position string (e.g. `"top center"`).
    pub fn start(self, s: impl Into<String>) -> Self {
        self.map(|state| state.config.start = s.into())
    }

    /// Sets the `end` position string (e.g. `"bottom 80%"`).
    pub fn end(self, s: impl Into<String>) -> Self {
        self.map(|state| state.config.end = s.into())
    }

    /// Sets the scrub mode.
    pub fn scrub(self, s: Scrub) -> Self {
        self.map(|state| state.config.scrub = s)
    }

    /// Sets the `toggleActions` mapping.
    pub fn toggle_actions(self, t: ToggleActions) -> Self {
        self.map(|state| state.config.toggle_actions = t)
    }

    /// Sets the fire-once flag.
    pub fn once(self, b: bool) -> Self {
        self.map(|state| state.config.once = b)
    }

    /// Sets an optional trigger id for labeling/debug.
    pub fn id(self, id: impl Into<String>) -> Self {
        self.map(|state| state.config.id = Some(id.into()))
    }

    /// Sets the `onEnter` callback.
    pub fn on_enter(self, f: impl Fn(crate::ScrollTriggerEvent) + 'static) -> Self {
        self.map(|state| state.config.on_enter = Some(crate::callbacks::scroll_callback(f)))
    }

    /// Sets the `onLeave` callback.
    pub fn on_leave(self, f: impl Fn(crate::ScrollTriggerEvent) + 'static) -> Self {
        self.map(|state| state.config.on_leave = Some(crate::callbacks::scroll_callback(f)))
    }

    /// Sets the `onEnterBack` callback.
    pub fn on_enter_back(self, f: impl Fn(crate::ScrollTriggerEvent) + 'static) -> Self {
        self.map(|state| state.config.on_enter_back = Some(crate::callbacks::scroll_callback(f)))
    }

    /// Sets the `onLeaveBack` callback.
    pub fn on_leave_back(self, f: impl Fn(crate::ScrollTriggerEvent) + 'static) -> Self {
        self.map(|state| state.config.on_leave_back = Some(crate::callbacks::scroll_callback(f)))
    }

    /// Sets the `onToggle` callback.
    pub fn on_toggle(self, f: impl Fn(crate::ScrollTriggerEvent) + 'static) -> Self {
        self.map(|state| state.config.on_toggle = Some(crate::callbacks::scroll_callback(f)))
    }

    /// Sets the `onUpdate` callback.
    pub fn on_update(self, f: impl Fn(crate::ScrollTriggerEvent) + 'static) -> Self {
        self.map(|state| state.config.on_update = Some(crate::callbacks::scroll_callback(f)))
    }

    /// Sets the `onRefresh` callback.
    pub fn on_refresh(self, f: impl Fn(crate::ScrollTriggerEvent) + 'static) -> Self {
        self.map(|state| state.config.on_refresh = Some(crate::callbacks::scroll_callback(f)))
    }
}

impl ScrollTriggerBuilder<ScrollTriggerBuilderNeedsTarget> {
    /// Attaches a stable target such as a `NodeRef` or DOM `Element`.
    pub fn target<T>(self, target: T) -> ReadyScrollTriggerBuilder
    where
        T: TriggerTargetSource + 'static,
    {
        self.map(move |state| {
            state.target = Some(Box::new(move |trigger| target.attach_to(trigger)));
        })
    }

    /// Attaches a resolver closure for dynamic target lookup.
    pub fn resolver<F>(self, resolver: F) -> ReadyScrollTriggerBuilder
    where
        F: Fn() -> Option<Element> + 'static,
    {
        self.map(move |state| {
            state.target = Some(Box::new(move |trigger| trigger.attach_resolver(resolver)));
        })
    }
}

#[cfg(feature = "controller")]
impl<State> ScrollTriggerBuilder<State> {
    /// Binds an [`leptos_fluid_motion::AnimationController`] to the trigger,
    /// driven by scroll progress. See
    /// [`ScrollTrigger::bind_controller`](crate::ScrollTrigger::bind_controller).
    pub fn bind_controller<F>(
        self,
        controller: leptos_fluid_motion::AnimationController,
        style_fn: F,
    ) -> Self
    where
        F: Fn(f64) -> leptos_fluid_motion::FluidStyle + 'static,
    {
        self.map(move |state| {
            state.controller_binding = Some(Box::new(move |trigger| {
                trigger.bind_controller(controller, style_fn);
            }));
        })
    }

    /// Same as [`Self::bind_controller`] but uses a fixed
    /// [`leptos_fluid_motion::Transition`] override per update. See
    /// [`ScrollTrigger::bind_controller_with`](crate::ScrollTrigger::bind_controller_with).
    pub fn bind_controller_with<F>(
        self,
        controller: leptos_fluid_motion::AnimationController,
        transition: leptos_fluid_motion::Transition,
        style_fn: F,
    ) -> Self
    where
        F: Fn(f64) -> leptos_fluid_motion::FluidStyle + 'static,
    {
        self.map(move |state| {
            state.controller_binding = Some(Box::new(move |trigger| {
                trigger.bind_controller_with(controller, transition, style_fn);
            }));
        })
    }
}

#[cfg(feature = "timeline")]
impl<State> ScrollTriggerBuilder<State> {
    /// Drives a [`leptos_fluid_motion::FluidTimeline`] via `toggleActions` when
    /// the trigger's active state changes. See
    /// [`ScrollTrigger::bind_timeline`](crate::ScrollTrigger::bind_timeline).
    pub fn bind_timeline(
        self,
        timeline: leptos_fluid_motion::FluidTimeline,
        toggle_actions: impl Into<String>,
    ) -> Self {
        let ta = toggle_actions.into();
        self.map(move |state| {
            state.timeline_binding = Some(Box::new(move |trigger| {
                trigger.bind_timeline(timeline, &ta);
            }));
        })
    }

    /// Discrete-step scrubbing of a [`leptos_fluid_motion::FluidTimeline`] by
    /// scroll progress. See
    /// [`ScrollTrigger::bind_timeline_scrub`](crate::ScrollTrigger::bind_timeline_scrub).
    pub fn bind_timeline_scrub<F>(
        self,
        timeline: leptos_fluid_motion::FluidTimeline,
        step_count: usize,
        style_fn: F,
    ) -> Self
    where
        F: Fn(usize, f64) -> leptos_fluid_motion::FluidStyle + 'static,
    {
        self.map(move |state| {
            state.timeline_binding = Some(Box::new(move |trigger| {
                trigger.bind_timeline_scrub(timeline, step_count, style_fn);
            }));
        })
    }
}

impl ReadyScrollTriggerBuilder {
    /// Installs the configured trigger, target, and optional motion bindings.
    ///
    /// Builds the trigger inner via [`ScrollTrigger::with_config`] (which
    /// registers with the shared scroll engine and installs `on_cleanup`),
    /// attaches the deferred target, runs any controller/timeline bindings, and
    /// finally calls [`ScrollTrigger::refresh`] so the freshly-attached target's
    /// geometry is measured.
    pub fn install(self) -> ScrollTrigger {
        let trigger = ScrollTrigger::with_config(self.state.config);
        if let Some(target) = self.state.target {
            target(trigger);
        }
        #[cfg(feature = "controller")]
        if let Some(controller_binding) = self.state.controller_binding {
            controller_binding(&trigger);
        }
        #[cfg(feature = "timeline")]
        if let Some(timeline_binding) = self.state.timeline_binding {
            timeline_binding(&trigger);
        }
        trigger.refresh();
        trigger
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_accumulates_config_without_target() {
        let builder = ScrollTrigger::builder()
            .start("top center")
            .end("bottom 80%")
            .once(true)
            .id("hero");
        assert_eq!(builder.state.config.start, "top center");
        assert_eq!(builder.state.config.end, "bottom 80%");
        assert!(builder.state.config.once);
        assert_eq!(builder.state.config.id.as_deref(), Some("hero"));
        assert!(builder.state.target.is_none());
    }

    #[test]
    fn builder_scrub_and_toggle_actions_set() {
        let builder = ScrollTrigger::builder()
            .scrub(Scrub::Number(0.5))
            .toggle_actions(ToggleActions::parse("play pause resume reset").unwrap());
        assert_eq!(builder.state.config.scrub, Scrub::Number(0.5));
        assert_eq!(
            builder.state.config.toggle_actions.0,
            [
                crate::toggle::Action::Play,
                crate::toggle::Action::Pause,
                crate::toggle::Action::Resume,
                crate::toggle::Action::Reset,
            ]
        );
    }

    #[test]
    fn builder_callback_slots_populate() {
        let builder = ScrollTrigger::builder()
            .on_enter(|_| {})
            .on_leave(|_| {})
            .on_update(|_| {});
        assert!(builder.state.config.on_enter.is_some());
        assert!(builder.state.config.on_leave.is_some());
        assert!(builder.state.config.on_update.is_some());
        assert!(builder.state.config.on_toggle.is_none());
    }
}