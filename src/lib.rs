//! Feature-gated animation toolkit for Leptos applications.
//!
//! Enable one or more feature modules:
//! - `flip`
//! - `view-transitions`
//! - `motion` for the common controller/component/wrapper surface
//! - fine-grained `motion-*` features for smaller motion builds
//! - `scroll` for scroll-triggered animations
//! - fine-grained `scroll-*` features for smaller scroll builds
//!
//! `leptos_fluid::motion` is available whenever `motion-core` is enabled,
//! including via `motion`, `motion-full`, and `full`.
//! `leptos_fluid::scroll` is available whenever `scroll` is enabled,
//! including via `scroll-full` and `full`.

#[cfg(feature = "flip")]
pub mod flip {
    //! FLIP layout animation primitives.
    pub use leptos_fluid_flip::*;
}

#[cfg(feature = "view-transitions")]
pub mod view_transitions {
    //! Nested route outlet transition primitives.
    pub use leptos_fluid_view_transitions::*;
}

#[cfg(feature = "motion-core")]
pub mod motion {
    //! Element-level reactive motion primitives.
    pub use leptos_fluid_motion::*;
}

#[cfg(feature = "scroll")]
pub mod scroll {
    //! Scroll-triggered animation primitives (GSAP ScrollTrigger clone).
    pub use leptos_fluid_scroll::*;
}

// Back-compat shim for previous `animators` module paths.
#[cfg(any(feature = "flip", feature = "view-transitions"))]
pub mod animators {
    #[cfg(feature = "flip")]
    pub mod flip {
        //! Backward-compatible `animators::flip` re-export.
        pub use leptos_fluid_flip::*;
    }

    #[cfg(feature = "view-transitions")]
    pub mod view_transitions {
        //! Backward-compatible `animators::view_transitions` re-export.
        pub use leptos_fluid_view_transitions::*;
    }
}
