//! Scroll-trigger event payload and pure velocity estimation.
//!
//! This module deliberately avoids any DOM or timer dependency: callers feed
//! in absolute timestamps (`time_ms`) and scroll positions, keeping the logic
//! fully host-testable. Phase 4+ motion integrations may also accept Leptos
//! `Callback<ScrollTriggerEvent>` where reactive ownership is required; in this
//! pure-logic phase we expose only `Rc<dyn Fn>` callback slots so no `leptos`
//! dependency is needed here.

use std::collections::VecDeque;
use std::rc::Rc;

/// Snapshot of a scroll trigger's state passed to callbacks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollTriggerEvent {
    /// Clamped progress in `0.0..=1.0` through the start/end range.
    pub progress: f64,
    /// `1` for forward, `-1` for backward, `0` for initial or no change.
    pub direction: i8,
    /// Whether the current scroll position is within the active range.
    pub is_active: bool,
    /// Estimated scroll velocity in pixels per second.
    pub velocity: f64,
}

impl ScrollTriggerEvent {
    /// Constructs an event, clamping `progress` to `[0.0, 1.0]`.
    pub fn new(progress: f64, direction: i8, is_active: bool, velocity: f64) -> Self {
        Self {
            progress: progress.max(0.0).min(1.0),
            direction,
            is_active,
            velocity,
        }
    }
}

/// Callback slot type used by `ScrollTriggerConfig`.
pub type ScrollCallback = Rc<dyn Fn(ScrollTriggerEvent)>;

/// Wraps a closure into a [`ScrollCallback`] slot.
pub fn scroll_callback<F: Fn(ScrollTriggerEvent) + 'static>(f: F) -> ScrollCallback {
    Rc::new(f)
}

/// Rolling-window velocity estimator for scroll position samples.
///
/// Samples older than `window_ms` from the newest sample are evicted on each
/// `push`. Velocity is computed as `(latest_pos - oldest_pos) / dt_seconds`.
#[derive(Clone, Debug)]
pub struct VelocityTracker {
    samples: VecDeque<(f64, f64)>,
    window_ms: f64,
}

impl VelocityTracker {
    /// Creates a tracker with the default 100ms window.
    pub fn new() -> Self {
        Self::with_window(100.0)
    }

    /// Creates a tracker with a custom window length in milliseconds.
    pub fn with_window(window_ms: f64) -> Self {
        Self {
            samples: VecDeque::new(),
            window_ms: window_ms.max(0.0),
        }
    }

    /// Records a sample at `time_ms` (absolute) with scroll position `scroll_pos`,
    /// evicting samples older than `time_ms - window_ms`.
    pub fn push(&mut self, time_ms: f64, scroll_pos: f64) {
        let cutoff = time_ms - self.window_ms;
        while let Some(&(front_time, _)) = self.samples.front() {
            if front_time < cutoff {
                self.samples.pop_front();
            } else {
                break;
            }
        }
        self.samples.push_back((time_ms, scroll_pos));
    }

    /// Returns the current velocity in pixels per second, or `0.0` if fewer than
    /// two samples are available or the time delta is zero.
    pub fn velocity(&self) -> f64 {
        let Some((front_time, front_pos)) = self.samples.front().copied() else {
            return 0.0;
        };
        let Some((back_time, back_pos)) = self.samples.back().copied() else {
            return 0.0;
        };
        let dt = back_time - front_time;
        if dt == 0.0 {
            return 0.0;
        }
        (back_pos - front_pos) / (dt / 1000.0)
    }
}

impl Default for VelocityTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_progress() {
        let ev = ScrollTriggerEvent::new(-1.0, 1, true, 10.0);
        assert_eq!(ev.progress, 0.0);
        let ev = ScrollTriggerEvent::new(2.0, 1, true, 10.0);
        assert_eq!(ev.progress, 1.0);
        let ev = ScrollTriggerEvent::new(0.5, 1, true, 10.0);
        assert_eq!(ev.progress, 0.5);
    }

    #[test]
    fn default_matches_new() {
        assert_eq!(VelocityTracker::default().window_ms, VelocityTracker::new().window_ms);
    }

    #[test]
    fn velocity_with_two_samples() {
        let mut tracker = VelocityTracker::with_window(2000.0);
        tracker.push(0.0, 0.0);
        tracker.push(1000.0, 500.0);
        assert_eq!(tracker.velocity(), 500.0);
    }

    #[test]
    fn velocity_evicts_old_samples() {
        let mut tracker = VelocityTracker::with_window(150.0);
        tracker.push(0.0, 0.0);
        tracker.push(50.0, 100.0);
        tracker.push(200.0, 200.0);
        assert_eq!(tracker.samples.len(), 2);
        assert_eq!(tracker.velocity(), (200.0 - 100.0) / ((200.0 - 50.0) / 1000.0));
    }

    #[test]
    fn velocity_zero_for_fewer_than_two_samples() {
        let mut tracker = VelocityTracker::new();
        assert_eq!(tracker.velocity(), 0.0);
        tracker.push(100.0, 50.0);
        assert_eq!(tracker.velocity(), 0.0);
    }

    #[test]
    fn velocity_zero_for_zero_time_delta() {
        let mut tracker = VelocityTracker::new();
        tracker.push(100.0, 50.0);
        tracker.push(100.0, 150.0);
        assert_eq!(tracker.velocity(), 0.0);
    }
}