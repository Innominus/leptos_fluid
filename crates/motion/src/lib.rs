mod presence;
mod spring_math;
mod spring_value;
mod style;
mod timeline;
mod timing;
mod transition;

mod components;
mod signal;

pub use presence::{FluidPresence, FluidSwap, PresenceMode};
#[cfg(feature = "bench")]
pub use spring_value::spring_step;
pub use spring_value::{use_spring, SpringValue};
pub use style::{FluidStyle, FluidValue, Transform};
pub use timeline::{FluidStep, FluidTimeline};
pub use transition::{Easing, Spring, Transition};

pub use components::{FluidButton, FluidDiv, FluidElement, FluidNodeRef, FluidSpan};
pub use signal::FluidSignal;

pub mod prelude {
    pub use crate::{
        use_spring, Easing, FluidElement, FluidNodeRef, FluidPresence, FluidStep, FluidStyle,
        FluidSwap, FluidTimeline, FluidValue, PresenceMode, Spring, SpringValue, Transform,
        Transition,
    };

    pub use crate::{FluidButton, FluidDiv, FluidSignal, FluidSpan};
}
