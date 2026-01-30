mod spring_value;
mod style;
mod transition;

mod components;
mod signal;

pub use spring_value::{use_spring, SpringValue};
pub use style::{MotionStyle, MotionValue, Transform};
pub use transition::{Easing, Spring, Transition};

pub use components::{MotionButton, MotionDiv, MotionElement, MotionNodeRef, MotionSpan};
pub use signal::MotionSignal;

pub mod prelude {
    pub use crate::{
        use_spring, Easing, MotionElement, MotionNodeRef, MotionStyle, MotionValue, Spring,
        SpringValue, Transform, Transition,
    };

    pub use crate::{MotionButton, MotionDiv, MotionSignal, MotionSpan};
}
