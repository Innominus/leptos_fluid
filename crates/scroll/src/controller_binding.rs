//! Scroll-driven [`AnimationController`] bindings.
//!
//! Each binding creates a Leptos `Effect` that reads `ScrollTrigger::progress()`
//! and dispatches the derived [`FluidStyle`] to the controller. **Every sample**
//! is applied via [`AnimationController::set_immediate`] (no WAAPI tween per
//! tick), because the scroll engine's scrub smoothing (`Scrub::Number(t)` in
//! `step_scrub`) already provides the interpolation. Starting a WAAPI tween each
//! rAF tick and cancelling it ~16ms later produces mid-tween redirection
//! glitches when scroll direction reverses rapidly; `set_immediate` commits each
//! smoothed value directly, eliminating the churn.
//!
//! For `scrub: Number`, the scroll engine smooths `progress()` (see
//! `crates/scroll/src/trigger.rs::step_scrub`), so `style_fn` receives the
//! smoothed value. For `scrub: Bool(true)` (direct 1:1), `style_fn` receives
//! the raw clamped progress. For `scrub: Bool(false)` (callback-only mode)
//! `progress()` still updates as the user scrolls, so the binding works the same
//! way; whether callbacks also fire is independent of the controller binding.
//!
//! For one-shot entrance animations (play once on enter, not scroll-tracked),
//! use `scrub: Bool(false)` + `on_enter(callback)` +
//! `controller.animate(style)` inside the callback — that starts a single clean
//! WAAPI tween with no redirection issue.

use leptos::prelude::{Effect, Get};
use leptos_fluid_motion::{AnimationController, FluidStyle};

use crate::ScrollTrigger;

impl ScrollTrigger {
    /// Binds a reactive style source to an [`AnimationController`], driven by
    /// scroll progress. `style_fn` receives the current (smoothed when
    /// `scrub: Number`) progress in `0.0..=1.0` and returns the target
    /// [`FluidStyle`]. Every value is applied immediately via
    /// [`AnimationController::set_immediate`] — no per-tick WAAPI tween —
    /// because the scroll engine's scrub smoothing handles interpolation.
    ///
    /// The effect's lifetime follows the current reactive owner scope, matching
    /// the lifecycle of `AnimationController::bind` in `crates/motion/src/controller.rs`.
    pub fn bind_controller(
        &self,
        controller: AnimationController,
        style_fn: Box<dyn Fn(f64) -> FluidStyle + 'static>,
    ) {
        let progress = self.progress();
        Effect::new(move || {
            let p = progress.get();
            let style = style_fn(p);
            controller.set_immediate(style);
        });
    }

    /// Same as [`ScrollTrigger::bind_controller`] but also sets the controller's
    /// default [`Transition`] (used by any subsequent `controller.animate()` call,
    /// e.g. from an `on_enter` callback). The transition is **not used** for
    /// per-tick application — all samples are applied via `set_immediate`, which
    /// bypasses WAAPI and writes inline styles directly. The transition is
    /// retained on the controller so a later `controller.animate(target)` (e.g.
    /// inside an `on_enter` callback) will use it.
    ///
    /// If you have no plans to call `controller.animate()` on this controller,
    /// prefer [`ScrollTrigger::bind_controller`] — there is no per-tick
    /// difference between the two methods.
    pub fn bind_controller_with(
        &self,
        controller: AnimationController,
        transition: leptos_fluid_motion::Transition,
        style_fn: Box<dyn Fn(f64) -> FluidStyle + 'static>,
    ) {
        controller.set_transition(transition);
        self.bind_controller(controller, style_fn);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ScrollTriggerConfig;
    use crate::Scrub;
    use leptos::prelude::{Get, ReadValue, RwSignal, Set};
    use leptos::reactive::owner::Owner;
    use leptos_fluid_motion::Transition;

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
            trigger.bind_controller(controller, Box::new(move |p| {
                observed_handle.set((p * 1000.0) as u32);
                FluidStyle::new().opacity(p)
            }));
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
            trigger.bind_controller_with(controller, Transition::default(), Box::new(move |p| {
                observed_handle.set((p * 1000.0) as u32);
                FluidStyle::new().opacity(p)
            }));
            any_spawner::Executor::poll_local();
            assert_eq!(observed.get(), 0);
            progress_signal.set(0.25);
            any_spawner::Executor::poll_local();
            assert_eq!(observed.get(), 250);
        });
    }
}