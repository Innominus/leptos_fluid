//! Scroll-driven [`AnimationController`] bindings.
//!
//! Mirrors `crates/motion/src/controller.rs:443-464` (`bind_signal`): each
//! binding creates a Leptos `Effect` that reads `ScrollTrigger::progress()` and
//! dispatches the derived [`FluidStyle`] to the controller. The first sample is
//! applied immediately (no tween) via an `initialized` flag, so the controller
//! adopts the current scroll state as its baseline rather than tweening into it
//! on mount. Subsequent samples animate via the controller's default transition
//! ([`ScrollTrigger::bind_controller`]) or a fixed per-call transition override
//! ([`ScrollTrigger::bind_controller_with`]).
//!
//! For `scrub: Number`, the scroll engine already smooths `progress()` (see
//! `crates/scroll/src/trigger.rs::step_scrub`), so `style_fn` receives the
//! smoothed value and the binding never double-smooths. For `scrub: Bool(false)`
//! (callback-only mode) `progress()` still updates as the user scrolls, so the
//! binding works the same way; whether callbacks also fire is independent of the
//! controller binding.

use leptos::prelude::{Effect, Get, GetValue, LocalStorage, SetValue, StoredValue};
use leptos_fluid_motion::{AnimationController, FluidStyle, Transition};

use crate::ScrollTrigger;

impl ScrollTrigger {
    /// Binds a reactive style source to an [`AnimationController`], driven by
    /// scroll progress. `style_fn` receives the current (smoothed when
    /// `scrub: Number`) progress in `0.0..=1.0` and returns the target
    /// [`FluidStyle`]. The first value is applied immediately (no tween);
    /// subsequent values animate via the controller's default transition.
    ///
    /// The effect's lifetime follows the current reactive owner scope, matching
    /// the lifecycle of `AnimationController::bind` in `crates/motion/src/controller.rs`.
    pub fn bind_controller<F>(&self, controller: AnimationController, style_fn: F)
    where
        F: Fn(f64) -> FluidStyle + 'static,
    {
        let progress = self.progress();
        let initialized: StoredValue<bool, LocalStorage> = StoredValue::new_local(false);
        Effect::new(move || {
            let p = progress.get();
            let style = style_fn(p);
            if initialized.get_value() {
                controller.animate(style);
            } else {
                controller.set_immediate(style);
                initialized.set_value(true);
            }
        });
    }

    /// Same as [`ScrollTrigger::bind_controller`] but uses a fixed
    /// [`Transition`] override per update via [`AnimationController::animate_with`].
    ///
    /// The transition is cloned per update; pass lightweight transitions or
    /// precompute as needed, matching the contract of
    /// `AnimationController::bind_with`.
    pub fn bind_controller_with<F>(
        &self,
        controller: AnimationController,
        transition: Transition,
        style_fn: F,
    ) where
        F: Fn(f64) -> FluidStyle + 'static,
    {
        let progress = self.progress();
        let initialized: StoredValue<bool, LocalStorage> = StoredValue::new_local(false);
        Effect::new(move || {
            let p = progress.get();
            let style = style_fn(p);
            if initialized.get_value() {
                controller.animate_with(style, transition.clone());
            } else {
                controller.set_immediate(style);
                initialized.set_value(true);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ScrollTriggerConfig;
    use crate::Scrub;
    use leptos::prelude::{Get, ReadValue, RwSignal, Set};
    use leptos::reactive::owner::Owner;

    #[test]
    fn bind_controller_runs_effect_without_panicking_with_no_target() {
        let _ = any_spawner::Executor::init_futures_executor();
        Owner::new().with(|| {
            let trigger = ScrollTrigger::host_test_trigger(
                ScrollTriggerConfig::default().scrub(Scrub::Bool(true)),
                0.0,
                false,
                0,
            );
            let progress_signal = trigger.inner.read_value().progress;
            let controller = AnimationController::new();
            let observed = RwSignal::new(0u32);
            let observed_handle = observed;
            trigger.bind_controller(controller, move |p| {
                observed_handle.set((p * 1000.0) as u32);
                FluidStyle::new().opacity(p)
            });
            any_spawner::Executor::poll_local();
            assert_eq!(observed.get(), 0);
            progress_signal.set(0.5);
            any_spawner::Executor::poll_local();
            assert_eq!(observed.get(), 500);
            progress_signal.set(1.0);
            any_spawner::Executor::poll_local();
            assert_eq!(observed.get(), 1000);
        });
    }

    #[test]
    fn bind_controller_with_runs_effect_without_panicking_with_no_target() {
        let _ = any_spawner::Executor::init_futures_executor();
        Owner::new().with(|| {
            let trigger = ScrollTrigger::host_test_trigger(
                ScrollTriggerConfig::default().scrub(Scrub::Bool(true)),
                0.0,
                false,
                0,
            );
            let progress_signal = trigger.inner.read_value().progress;
            let controller = AnimationController::new();
            let observed = RwSignal::new(0u32);
            let observed_handle = observed;
            trigger.bind_controller_with(controller, Transition::default(), move |p| {
                observed_handle.set((p * 1000.0) as u32);
                FluidStyle::new().opacity(p)
            });
            any_spawner::Executor::poll_local();
            assert_eq!(observed.get(), 0);
            progress_signal.set(0.25);
            any_spawner::Executor::poll_local();
            assert_eq!(observed.get(), 250);
        });
    }
}