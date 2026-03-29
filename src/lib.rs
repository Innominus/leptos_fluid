//! Feature-gated animation toolkit for Leptos applications.
//!
//! Enable one or more feature modules:
//! - `flip`
//! - `view-transitions`
//! - `motion` or fine-grained `motion-*` features

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
