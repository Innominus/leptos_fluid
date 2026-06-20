//! Scroll-driven [`FluidTimeline`] bindings.
//!
//! Two integration modes mirror GSAP ScrollTrigger's timeline coupling:
//!
//! - [`ScrollTrigger::bind_timeline`] maps the four-phase `toggleActions`
//!   (`onEnter` / `onLeave` / `onEnterBack` / `onLeaveBack`) to `FluidTimeline`
//!   methods. The binding watches `is_active()` and `direction()` and dispatches
//!   the configured [`Action`] on each phase transition.
//! - [`ScrollTrigger::bind_timeline_scrub`] maps scroll `progress()` to a
//!   discrete step index and jumps the timeline to that step via
//!   `set_immediate`. Continuous interpolated scrubbing is deferred: see the
//!   doc comment on `bind_timeline_scrub` for the rationale.

use leptos::prelude::{Effect, Get, GetValue, LocalStorage, SetValue, StoredValue};
use leptos_fluid_motion::{FluidStyle, FluidTimeline};

use crate::toggle::{Action, TogglePhase};
use crate::ScrollTrigger;

impl ScrollTrigger {
    /// Drives a [`FluidTimeline`] via `toggleActions` when the trigger's active
    /// state changes.
    ///
    /// `toggle_actions` is a four-token string like `"play pause resume reset"`
    /// mapping to `onEnter / onLeave / onEnterBack / onLeaveBack` (the GSAP
    /// order). The binding tracks the previous `is_active` value and dispatches
    /// the action for the phase transition `(prev_active, active, direction)`.
    ///
    /// `Reset`, `Complete`, and `Reverse` have no perfect `FluidTimeline`
    /// equivalents: see [`apply_timeline_action`] for the chosen mappings and
    /// their limitations.
    pub fn bind_timeline(&self, timeline: FluidTimeline, toggle_actions: &str) {
        let actions = crate::toggle::parse_toggle_actions(toggle_actions)
            .unwrap_or([Action::Play, Action::None, Action::None, Action::None]);

        let is_active = self.is_active();
        let direction = self.direction();
        let prev_active: StoredValue<bool, LocalStorage> = StoredValue::new_local(false);

        Effect::new(move || {
            let active = is_active.get();
            let dir = direction.get();
            let prev = prev_active.get_value();
            if active == prev {
                return;
            }
            prev_active.set_value(active);

            let phase = if !prev && active && dir >= 0 {
                TogglePhase::OnEnter
            } else if prev && !active && dir >= 0 {
                TogglePhase::OnLeave
            } else if !prev && active && dir < 0 {
                TogglePhase::OnEnterBack
            } else if prev && !active && dir < 0 {
                TogglePhase::OnLeaveBack
            } else {
                return;
            };

            let action = actions[phase as usize];
            apply_timeline_action(&timeline, action);
        });
    }

    /// Discrete-step scrubbing of a [`FluidTimeline`] by scroll progress.
    ///
    /// `style_fn` receives `(step_index, progress)` where `step_index` is the
    /// target step computed as `(progress * step_count).floor()` clamped to
    /// `step_count - 1`, and `progress` is the raw (already-smoothed when
    /// `scrub: Number`) progress in `0.0..=1.0`. When the target index changes,
    /// the binding calls `timeline.set_immediate(style_fn(...))` so the timeline
    /// jumps between steps as the user scrolls.
    ///
    /// `step_count` is supplied by the caller because `FluidTimeline` does not
    /// expose its step list for reading (`set_steps` is write-only and
    /// `step_index()` returns the running index, not the list length).
    ///
    /// **Limitation:** `FluidTimeline` is step-index based with `wait_ms` per
    /// step, not a continuous time-based timeline, and `FluidStyle` has no
    /// built-in lerp. This binding therefore jumps between steps rather than
    /// interpolating. Continuous interpolated scrubbing is deferred until
    /// `FluidStyle` gains an interpolation helper. For smooth scrubbing today,
    /// use [`ScrollTrigger::bind_controller`] with a style function.
    pub fn bind_timeline_scrub<F>(&self, timeline: FluidTimeline, step_count: usize, style_fn: F)
    where
        F: Fn(usize, f64) -> FluidStyle + 'static,
    {
        let progress = self.progress();
        let prev_index: StoredValue<Option<usize>, LocalStorage> = StoredValue::new_local(None);
        Effect::new(move || {
            let p = progress.get();
            let target = if step_count == 0 {
                0
            } else {
                ((p * step_count as f64).floor() as usize).min(step_count - 1)
            };
            if prev_index.get_value() == Some(target) {
                return;
            }
            prev_index.set_value(Some(target));
            let style = style_fn(target, p);
            timeline.set_immediate(style);
        });
    }
}

/// Dispatches a single `toggleActions` [`Action`] to a [`FluidTimeline`].
///
/// `Play` / `Pause` / `Resume` / `Restart` / `None` map directly to the
/// equivalent `FluidTimeline` methods.
///
/// `Reset`, `Complete`, and `Reverse` have no exact `FluidTimeline` primitive:
///
/// - `Reset` maps to [`FluidTimeline::stop`]. `FluidTimeline` does not expose
///   the initial step style from outside, so the timeline halts at its current
///   position rather than rewinding to the initial state. Callers that need a
///   true rewind should pair this with an explicit `set_immediate(initial_style)`.
/// - `Complete` maps to [`FluidTimeline::play`], letting the sequence run to its
///   final step naturally. `FluidTimeline` has no public "jump to last step"
///   primitive, and reading the last step's style would require access to the
///   step list that the timeline intentionally keeps write-only.
/// - `Reverse` maps to [`FluidTimeline::stop`]. `FluidTimeline` has no reverse
///   primitive. For progress-controlled scrubbing in both directions, use
///   [`ScrollTrigger::bind_timeline_scrub`].
fn apply_timeline_action(timeline: &FluidTimeline, action: Action) {
    use Action::*;
    match action {
        Play => timeline.play(),
        Pause => timeline.pause(),
        Resume => {
            timeline.resume();
        }
        Reset => timeline.stop(),
        Restart => timeline.restart(),
        Complete => timeline.play(),
        Reverse => timeline.stop(),
        None => {}
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use super::*;
    use crate::config::ScrollTriggerConfig;
    use crate::Scrub;
    use leptos::prelude::{GetUntracked, ReadValue, RwSignal, Set};
    use leptos::reactive::owner::Owner;
    use leptos_fluid_motion::{FluidStep, FluidStyle};

    #[test]
    fn apply_timeline_action_play_starts_timeline() {
        let _ = any_spawner::Executor::init_futures_executor();
        Owner::new().with(|| {
            let timeline = FluidTimeline::new(FluidStyle::new());
            timeline.set_steps(vec![FluidStep::new(FluidStyle::new().opacity(1.0)).wait_ms(1000)]);
            apply_timeline_action(&timeline, Action::Play);
            any_spawner::Executor::poll_local();
            assert!(timeline.is_running().get_untracked());
        });
    }

    #[test]
    fn apply_timeline_action_pause_marks_paused_when_running() {
        let _ = any_spawner::Executor::init_futures_executor();
        Owner::new().with(|| {
            let timeline = FluidTimeline::new(FluidStyle::new());
            timeline.set_steps(vec![
                FluidStep::new(FluidStyle::new().opacity(0.5)).wait_ms(500),
                FluidStep::new(FluidStyle::new().opacity(1.0)).wait_ms(500),
            ]);
            apply_timeline_action(&timeline, Action::Play);
            any_spawner::Executor::poll_local();
            apply_timeline_action(&timeline, Action::Pause);
            any_spawner::Executor::poll_local();
            assert!(timeline.is_paused().get_untracked());
            assert!(!timeline.is_running().get_untracked());
        });
    }

    #[test]
    fn apply_timeline_action_none_is_noop() {
        let _ = any_spawner::Executor::init_futures_executor();
        Owner::new().with(|| {
            let timeline = FluidTimeline::new(FluidStyle::new());
            apply_timeline_action(&timeline, Action::None);
            any_spawner::Executor::poll_local();
            assert!(!timeline.is_running().get_untracked());
            assert!(!timeline.is_paused().get_untracked());
        });
    }

    #[test]
    fn apply_timeline_action_stop_clears_running_and_paused() {
        let _ = any_spawner::Executor::init_futures_executor();
        Owner::new().with(|| {
            let timeline = FluidTimeline::new(FluidStyle::new());
            timeline.set_steps(vec![
                FluidStep::new(FluidStyle::new().opacity(0.5)).wait_ms(500),
                FluidStep::new(FluidStyle::new().opacity(1.0)).wait_ms(500),
            ]);
            apply_timeline_action(&timeline, Action::Play);
            any_spawner::Executor::poll_local();
            apply_timeline_action(&timeline, Action::Reset);
            any_spawner::Executor::poll_local();
            assert!(!timeline.is_running().get_untracked());
            assert!(!timeline.is_paused().get_untracked());
        });
    }

    #[test]
    fn bind_timeline_dispatches_play_on_on_enter() {
        let _ = any_spawner::Executor::init_futures_executor();
        Owner::new().with(|| {
            let trigger = ScrollTrigger::host_test_trigger(
                ScrollTriggerConfig::default().scrub(Scrub::Bool(true)),
                0.0,
                false,
                1,
            );
            let is_active_signal = trigger.inner.read_value().is_active;
            let timeline = FluidTimeline::new(FluidStyle::new());
            timeline.set_steps(vec![
                FluidStep::new(FluidStyle::new().opacity(0.5)).wait_ms(500),
                FluidStep::new(FluidStyle::new().opacity(1.0)).wait_ms(500),
            ]);
            trigger.bind_timeline(timeline, "play none none none");
            any_spawner::Executor::poll_local();
            assert!(!timeline.is_running().get_untracked());
            is_active_signal.set(true);
            any_spawner::Executor::poll_local();
            assert!(timeline.is_running().get_untracked());
        });
    }

    #[test]
    fn bind_timeline_scrub_jumps_to_target_step() {
        let _ = any_spawner::Executor::init_futures_executor();
        Owner::new().with(|| {
            let trigger = ScrollTrigger::host_test_trigger(
                ScrollTriggerConfig::default().scrub(Scrub::Bool(true)),
                0.0,
                false,
                1,
            );
            let progress_signal = trigger.inner.read_value().progress;
            let timeline = FluidTimeline::new(FluidStyle::new());
            let observed = RwSignal::new(usize::MAX);
            let observed_handle = observed;
            trigger.bind_timeline_scrub(timeline, 4, move |index, _p| {
                observed_handle.set(index);
                FluidStyle::new().opacity(index as f64 / 4.0)
            });
            any_spawner::Executor::poll_local();
            assert_eq!(observed.get_untracked(), 0);

            progress_signal.set(0.5);
            any_spawner::Executor::poll_local();
            assert_eq!(observed.get_untracked(), 2);

            progress_signal.set(0.9);
            any_spawner::Executor::poll_local();
            assert_eq!(observed.get_untracked(), 3);

            progress_signal.set(0.9);
            any_spawner::Executor::poll_local();
            assert_eq!(observed.get_untracked(), 3);
        });
    }
}