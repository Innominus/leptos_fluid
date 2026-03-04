//! Reactive motion primitives for Leptos components.

mod spring_math;
mod spring_value;
mod style;
mod timeline;
mod timing;
mod transition;

mod animator;
mod components;
mod controller;
mod signal;

#[cfg(feature = "bench")]
pub use spring_value::spring_step;
pub use spring_value::{SpringValue, use_spring};
pub use style::{FluidStyle, FluidValue, Transform};
pub use timeline::{FluidStep, FluidTimeline};
pub use transition::{Easing, Spring, Transition};

pub use components::{FluidButton, FluidDiv, FluidElement, FluidNodeRef, FluidSpan};
pub use controller::AnimationController;
pub use signal::FluidSignal;

pub mod prelude {
    pub use crate::{
        AnimationController, Easing, FluidElement, FluidNodeRef, FluidStep, FluidStyle,
        FluidTimeline, FluidValue, Spring, SpringValue, Transform, Transition, use_spring,
    };

    pub use crate::{FluidButton, FluidDiv, FluidSignal, FluidSpan};
}
