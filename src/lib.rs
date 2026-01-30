#[cfg(feature = "flip")]
pub mod flip {
    pub use leptos_fluid_flip::*;
}

#[cfg(feature = "view-transitions")]
pub mod view_transitions {
    pub use leptos_fluid_view_transitions::*;
}

#[cfg(feature = "motion")]
pub mod motion {
    pub use leptos_fluid_motion::*;
}

// Back-compat shim for previous `animators` module paths.
#[cfg(any(feature = "flip", feature = "view-transitions"))]
pub mod animators {
    #[cfg(feature = "flip")]
    pub mod flip {
        pub use leptos_fluid_flip::*;
    }

    #[cfg(feature = "view-transitions")]
    pub mod view_transitions {
        pub use leptos_fluid_view_transitions::*;
    }
}
