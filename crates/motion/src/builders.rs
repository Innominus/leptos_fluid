use std::marker::PhantomData;

#[cfg(feature = "controller")]
use web_sys::Element;

use crate::controller::{AnimationController, ControllerTarget};
use crate::macro_support::watch_on_change;
use crate::{FluidSignal, FluidStyle, Transition};
#[cfg(feature = "timeline")]
use crate::{FluidStep, FluidTimeline};

type ControllerInstaller = Box<dyn FnOnce(AnimationController)>;
#[cfg(feature = "timeline")]
type TimelineInstaller = Box<dyn FnOnce(FluidTimeline)>;

enum ControllerBinding {
    Animate(FluidSignal<FluidStyle>),
    AnimateWith(FluidSignal<FluidStyle>, Transition),
}

struct ControllerBuilderState {
    transition: Transition,
    attachment: Option<ControllerInstaller>,
    initial: Option<FluidStyle>,
    binding: Option<ControllerBinding>,
}

#[doc(hidden)]
pub struct ControllerBuilderNeedsAttachment;

#[doc(hidden)]
pub struct ControllerBuilderReady;

/// Typed builder for [`AnimationController`].
pub struct AnimationControllerBuilder<State = ControllerBuilderNeedsAttachment> {
    state: ControllerBuilderState,
    _marker: PhantomData<State>,
}

/// Ready-to-install controller builder with an attachment source.
pub type ReadyAnimationControllerBuilder = AnimationControllerBuilder<ControllerBuilderReady>;

#[cfg(feature = "timeline")]
struct TimelineBuilderState {
    controller: AnimationController,
    initial: FluidStyle,
    autoplay: bool,
    auto_loop: bool,
    steps: Vec<FluidStep>,
    installers: Vec<TimelineInstaller>,
}

#[doc(hidden)]
#[cfg(feature = "timeline")]
pub struct TimelineBuilderNeedsStep;

#[doc(hidden)]
#[cfg(feature = "timeline")]
pub struct TimelineBuilderReady;

/// Typed builder for [`FluidTimeline`].
#[cfg(feature = "timeline")]
pub struct FluidTimelineBuilder<State = TimelineBuilderNeedsStep> {
    state: TimelineBuilderState,
    _marker: PhantomData<State>,
}

/// Ready-to-install timeline builder with at least one step.
#[cfg(feature = "timeline")]
pub type ReadyFluidTimelineBuilder = FluidTimelineBuilder<TimelineBuilderReady>;

impl AnimationController {
    /// Starts building a typed controller configuration.
    pub fn builder() -> AnimationControllerBuilder {
        AnimationControllerBuilder {
            state: ControllerBuilderState {
                transition: Transition::default(),
                attachment: None,
                initial: None,
                binding: None,
            },
            _marker: PhantomData,
        }
    }

    /// Installs a typed on-change rule without macro syntax.
    pub fn on_change<T, Source, OnChange>(&self, source: Source, mut on_change: OnChange)
    where
        T: Clone + PartialEq + 'static,
        Source: Fn() -> T + 'static,
        OnChange: FnMut(T, AnimationController) + 'static,
    {
        let controller = *self;
        watch_on_change(Box::new(source), Box::new(move |next| on_change(next, controller)));
    }
}

impl<State> AnimationControllerBuilder<State> {
    #[inline]
    fn map<Next>(
        self,
        f: impl FnOnce(&mut ControllerBuilderState),
    ) -> AnimationControllerBuilder<Next> {
        let mut state = self.state;
        f(&mut state);
        AnimationControllerBuilder {
            state,
            _marker: PhantomData,
        }
    }

    /// Sets the default transition used by `animate(...)` calls.
    pub fn transition(self, transition: Transition) -> Self {
        self.map(|state| state.transition = transition)
    }

    /// Applies an initial style immediately during installation.
    pub fn initial(self, style: FluidStyle) -> Self {
        self.map(|state| state.initial = Some(style))
    }

    /// Binds a reactive style source using the controller's default transition.
    pub fn animate<T>(self, style: T) -> Self
    where
        T: Into<FluidSignal<FluidStyle>>,
    {
        let style = style.into();
        self.map(move |state| state.binding = Some(ControllerBinding::Animate(style)))
    }

    /// Binds a reactive style source using a fixed transition override.
    pub fn animate_with<T>(self, style: T, transition: Transition) -> Self
    where
        T: Into<FluidSignal<FluidStyle>>,
    {
        let style = style.into();
        self.map(move |state| {
            state.binding = Some(ControllerBinding::AnimateWith(style, transition))
        })
    }
}

impl AnimationControllerBuilder<ControllerBuilderNeedsAttachment> {
    /// Attaches a stable target such as a `NodeRef` or DOM `Element`.
    pub fn target<T>(self, target: T) -> ReadyAnimationControllerBuilder
    where
        T: ControllerTarget + 'static,
    {
        self.map(move |state| {
            state.attachment = Some(Box::new(move |controller| controller.attach_target(target)));
        })
    }

    /// Attaches a resolver for dynamic target lookup.
    pub fn resolver<F>(self, resolver: F) -> ReadyAnimationControllerBuilder
    where
        F: Fn() -> Option<Element> + 'static,
    {
        self.map(move |state| {
            state.attachment = Some(Box::new(move |controller| {
                controller.attach_resolver(resolver)
            }));
        })
    }
}

impl ReadyAnimationControllerBuilder {
    /// Installs the configured controller, attachment, and optional bindings.
    pub fn install(self) -> AnimationController {
        let controller = AnimationController::with_transition(self.state.transition);

        if let Some(attachment) = self.state.attachment {
            attachment(controller);
        }
        if let Some(binding) = self.state.binding {
            match binding {
                ControllerBinding::Animate(style) => match self.state.initial {
                    Some(initial) => {
                        controller.set_immediate(initial);
                        controller.bind(style);
                    }
                    None => controller.bind_immediate(style),
                },
                ControllerBinding::AnimateWith(style, transition) => match self.state.initial {
                    Some(initial) => {
                        controller.set_immediate(initial);
                        controller.bind_with(style, transition);
                    }
                    None => controller.bind_with_immediate(style, transition),
                },
            }
        } else if let Some(initial) = self.state.initial {
            controller.set_immediate(initial);
        }

        controller
    }
}

#[cfg(feature = "timeline")]
impl FluidTimeline {
    /// Starts building a typed timeline bound to a controller.
    pub fn builder(controller: AnimationController) -> FluidTimelineBuilder {
        FluidTimelineBuilder {
            state: TimelineBuilderState {
                controller,
                initial: FluidStyle::new(),
                autoplay: false,
                auto_loop: false,
                steps: Vec::new(),
                installers: Vec::new(),
            },
            _marker: PhantomData,
        }
    }

    /// Installs a typed on-change rule without macro syntax.
    pub fn on_change<T, Source, OnChange>(&self, source: Source, mut on_change: OnChange)
    where
        T: Clone + PartialEq + 'static,
        Source: Fn() -> T + 'static,
        OnChange: FnMut(T, FluidTimeline) + 'static,
    {
        let timeline = *self;
        watch_on_change(Box::new(source), Box::new(move |next| on_change(next, timeline)));
    }
}

#[cfg(feature = "timeline")]
impl<State> FluidTimelineBuilder<State> {
    #[inline]
    fn map<Next>(self, f: impl FnOnce(&mut TimelineBuilderState)) -> FluidTimelineBuilder<Next> {
        let mut state = self.state;
        f(&mut state);
        FluidTimelineBuilder {
            state,
            _marker: PhantomData,
        }
    }

    /// Sets the initial timeline style.
    pub fn initial(self, style: FluidStyle) -> Self {
        self.map(|state| state.initial = style)
    }

    /// Enables autoplay immediately after installation.
    pub fn autoplay(self, autoplay: bool) -> Self {
        self.map(|state| state.autoplay = autoplay)
    }

    /// Enables or disables automatic looping.
    pub fn auto_loop(self, auto_loop: bool) -> Self {
        self.map(|state| state.auto_loop = auto_loop)
    }

    /// Appends a step and returns a ready builder.
    pub fn step(self, step: FluidStep) -> ReadyFluidTimelineBuilder {
        self.map(move |state| state.steps.push(step))
    }

    /// Installs a typed on-change rule that runs after the timeline is built.
    pub fn on_change<T, Source, OnChange>(self, source: Source, on_change: OnChange) -> Self
    where
        T: Clone + PartialEq + 'static,
        Source: Fn() -> T + 'static,
        OnChange: FnMut(T, FluidTimeline) + 'static,
    {
        self.map(move |state| {
            state.installers.push(Box::new(move |timeline| {
                timeline.on_change(source, on_change)
            }));
        })
    }
}

#[cfg(feature = "timeline")]
impl ReadyFluidTimelineBuilder {
    /// Installs the configured timeline and optional on-change rules.
    pub fn install(self) -> FluidTimeline {
        let timeline = FluidTimeline::new(self.state.initial);
        timeline.bind(self.state.controller);
        timeline.set_steps(self.state.steps);
        timeline.set_auto_loop(self.state.auto_loop);

        for installer in self.state.installers {
            installer(timeline);
        }

        if self.state.autoplay {
            timeline.play();
        }

        timeline
    }
}
