//! Scroll-triggered animations for Leptos (CSR): a focused GSAP ScrollTrigger clone.
//!
//! In its minimal configuration (`default-features = false`) the crate provides a
//! pure-callback mode with no dependency on `leptos_fluid_motion`. Enabling the
//! `controller`, `timeline`, `builders`, or `macros` features pulls in motion
//! integrations so scroll-driven updates can drive `AnimationController`s and
//! `FluidTimeline`s.

mod callbacks;
mod config;
mod engine;
mod position;
mod scroller;
mod toggle;
mod trigger;

#[cfg(feature = "controller")]
mod controller_binding;
#[cfg(feature = "timeline")]
mod timeline_binding;
#[cfg(feature = "builders")]
mod builders;
#[cfg(feature = "macros")]
mod macros;
#[cfg(any(feature = "builders", feature = "macros"))]
mod macro_support;

pub use callbacks::{ScrollCallback, ScrollTriggerEvent, VelocityTracker, scroll_callback};
pub use config::{ReducedMotion, ScrollTriggerConfig, Scrub, ToggleActions};
pub use position::{
    Rect, ScrollOffset, ScrollPoint, ScrollPosition, clamp_value, parse_offset, parse_point,
    parse_position, parse_start_end, resolve_start, strip_clamp,
};
pub use scroller::{Scroller, ScrollListenerHandle};
pub use toggle::{
    Action, ScrollDirection, TogglePhase, action_for, parse_action, parse_toggle_actions,
};
pub use trigger::{ScrollTrigger, TriggerTargetSource};
pub use engine::set_reduced_motion;
#[cfg(feature = "builders")]
pub use builders::{ReadyScrollTriggerBuilder, ScrollTriggerBuilder};

#[doc(hidden)]
#[cfg(any(feature = "builders", feature = "macros"))]
pub mod __private {
    pub use crate::macro_support::{ScrubAuto, ScrubKind, watch_progress};
}

pub mod prelude {
    //! Re-exports of the public API.
    pub use crate::{
        Action, Rect, ScrollCallback, ScrollDirection, ScrollListenerHandle, ScrollOffset,
        ScrollPoint, ScrollPosition, ScrollTrigger, ScrollTriggerConfig, ScrollTriggerEvent,
        Scroller, Scrub, ToggleActions, TogglePhase, TriggerTargetSource, VelocityTracker,
        ReducedMotion, action_for, clamp_value, parse_action, parse_offset, parse_point,
        parse_position, parse_start_end, resolve_start, scroll_callback, set_reduced_motion,
        strip_clamp,
    };
    #[cfg(feature = "builders")]
    pub use crate::{ReadyScrollTriggerBuilder, ScrollTriggerBuilder};
    #[cfg(feature = "macros")]
    pub use crate::scroll_trigger;
}