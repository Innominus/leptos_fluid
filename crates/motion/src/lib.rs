//! Reactive motion primitives for Leptos components.

mod spring_math;
#[cfg(feature = "spring")]
mod spring_value;
mod style;
#[cfg(feature = "timeline")]
mod timeline;
#[cfg(any(feature = "spring", feature = "timeline"))]
mod timing;
mod transition;

#[cfg(feature = "controller")]
mod animator;
#[cfg(feature = "auto-size")]
mod auto_size;
#[cfg(feature = "builders")]
mod builders;
#[cfg(feature = "components")]
mod components;
#[cfg(feature = "controller")]
mod controller;
#[cfg(any(feature = "builders", feature = "macros"))]
mod macro_support;
#[cfg(feature = "macros")]
mod macros;
mod signal;

#[cfg(feature = "bench")]
#[cfg(feature = "spring")]
pub use spring_value::spring_step;
#[cfg(feature = "spring")]
pub use spring_value::{SpringValue, use_spring};
pub use style::{FluidStyle, FluidValue, Transform};
#[cfg(feature = "timeline")]
pub use timeline::{FluidStep, FluidTimeline};
pub use transition::{Easing, Spring, Transition};

#[cfg(feature = "auto-size")]
pub use auto_size::{
    AutoSizeAxis, AutoSizeOptions, bind_auto_height, bind_auto_height_with, bind_auto_size,
    bind_auto_size_with, bind_auto_width, bind_auto_width_with,
};
#[cfg(feature = "builders")]
pub use builders::{AnimationControllerBuilder, ReadyAnimationControllerBuilder};
#[cfg(all(feature = "builders", feature = "timeline"))]
pub use builders::{FluidTimelineBuilder, ReadyFluidTimelineBuilder};
#[cfg(feature = "wrappers")]
pub use components::{FluidButton, FluidDiv, FluidSpan};
#[cfg(feature = "components")]
pub use components::{FluidElement, FluidNodeRef};
#[cfg(feature = "controller")]
pub use controller::{AnimationController, ControllerTarget};
pub use signal::FluidSignal;

#[doc(hidden)]
#[cfg(any(feature = "builders", feature = "macros"))]
pub mod __private {
    pub use crate::macro_support::watch_on_change;
}

pub mod prelude {
    pub use crate::style;
    #[cfg(all(feature = "macros", feature = "timeline"))]
    pub use crate::timeline;
    #[cfg(feature = "controller")]
    pub use crate::{AnimationController, ControllerTarget};
    #[cfg(feature = "builders")]
    pub use crate::{AnimationControllerBuilder, ReadyAnimationControllerBuilder};
    #[cfg(feature = "auto-size")]
    pub use crate::{
        AutoSizeAxis, AutoSizeOptions, bind_auto_height, bind_auto_height_with, bind_auto_size,
        bind_auto_size_with, bind_auto_width, bind_auto_width_with,
    };
    pub use crate::{Easing, FluidSignal, FluidStyle, FluidValue, Spring, Transform, Transition};
    #[cfg(feature = "wrappers")]
    pub use crate::{FluidButton, FluidDiv, FluidSpan};
    #[cfg(feature = "components")]
    pub use crate::{FluidElement, FluidNodeRef};
    #[cfg(feature = "timeline")]
    pub use crate::{FluidStep, FluidTimeline};
    #[cfg(all(feature = "builders", feature = "timeline"))]
    pub use crate::{FluidTimelineBuilder, ReadyFluidTimelineBuilder};
    #[cfg(feature = "spring")]
    pub use crate::{SpringValue, use_spring};
    #[cfg(feature = "macros")]
    pub use crate::{controller, when};
}
