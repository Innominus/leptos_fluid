//! Scroll-trigger event payload and pure velocity estimation.
//!
//! This module deliberately avoids any DOM or timer dependency: callers feed
//! in absolute timestamps (`time_ms`) and scroll positions, keeping the logic
//! fully host-testable. Phase 4+ motion integrations may also accept Leptos
//! `Callback<ScrollTriggerEvent>` where reactive ownership is required; in this
//! pure-logic phase we expose only `Rc<dyn Fn>` callback slots so no `leptos`
//! dependency is needed here.

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
/// Backed by a fixed 32-slot ring buffer (no heap allocation). 32 samples
/// covers a 100ms window at 240Hz (24 samples) and a 200ms window at 120Hz.
/// Samples older than `window_ms` from the newest sample are evicted on each
/// `push`. Velocity is computed as `(latest_pos - oldest_pos) / dt_seconds`.
#[derive(Clone)]
pub struct VelocityTracker {
    samples: [(f64, f64); 32],
    head: usize,
    len: usize,
    window_ms: f64,
}

impl VelocityTracker {
    const CAP: usize = 32;

    /// Creates a tracker with the default 100ms window.
    pub fn new() -> Self {
        Self::with_window(100.0)
    }

    /// Creates a tracker with a custom window length in milliseconds.
    pub fn with_window(window_ms: f64) -> Self {
        Self {
            samples: [(0.0, 0.0); 32],
            head: 0,
            len: 0,
            window_ms: window_ms.max(0.0),
        }
    }

    /// Records a sample at `time_ms` (absolute) with scroll position `scroll_pos`,
    /// evicting samples older than `time_ms - window_ms`.
    pub fn push(&mut self, time_ms: f64, scroll_pos: f64) {
        let cutoff = time_ms - self.window_ms;
        // Evict oldest-first while the front sample is strictly older than cutoff.
        while self.len > 0 {
            let front_idx = self.head;
            if self.samples[front_idx].0 < cutoff {
                self.head = (self.head + 1) % Self::CAP;
                self.len -= 1;
            } else {
                break;
            }
        }
        if self.len < Self::CAP {
            let idx = (self.head + self.len) % Self::CAP;
            self.samples[idx] = (time_ms, scroll_pos);
            self.len += 1;
        } else {
            // Full: overwrite the oldest (head) and advance head.
            self.samples[self.head] = (time_ms, scroll_pos);
            self.head = (self.head + 1) % Self::CAP;
        }
    }

    /// Returns the current velocity in pixels per second, or `0.0` if fewer than
    /// two samples are available or the time delta is zero.
    pub fn velocity(&self) -> f64 {
        self.velocity_now(self.back_time())
    }

    /// Returns the velocity at an explicit "now" timestamp, applying GSAP's
    /// Observer idle-drop: if more than 500ms has elapsed since the newest
    /// sample, the samples are stale (e.g. rAF was paused while the tab was
    /// hidden) and the velocity is reported as 0.0.
    pub fn velocity_now(&self, now_ms: f64) -> f64 {
        if self.len < 2 {
            return 0.0;
        }
        let back_time = self.back_time();
        // GSAP-parity: drop velocity to 0 after 500ms of inactivity so stale
        // samples from a paused rAF loop don't report a phantom velocity.
        if now_ms - back_time > 500.0 {
            return 0.0;
        }
        let (front_time, front_pos) = self.samples[self.head];
        let back_idx = (self.head + self.len - 1) % Self::CAP;
        let back_pos = self.samples[back_idx].1;
        let dt = back_time - front_time;
        if dt == 0.0 {
            return 0.0;
        }
        (back_pos - front_pos) / (dt / 1000.0)
    }

    /// Returns the timestamp of the newest sample, or `0.0` if empty.
    fn back_time(&self) -> f64 {
        if self.len == 0 {
            return 0.0;
        }
        let back_idx = (self.head + self.len - 1) % Self::CAP;
        self.samples[back_idx].0
    }

    /// Number of samples currently retained (exposed for tests).
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.len
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

    mod event {
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
    }

    mod velocity_basic {
        use super::*;

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

    mod velocity_edge_cases {
        use super::*;

        #[test]
        fn velocity_evicts_old_samples() {
            let mut tracker = VelocityTracker::with_window(150.0);
            tracker.push(0.0, 0.0);
            tracker.push(50.0, 100.0);
            tracker.push(200.0, 200.0);
            assert_eq!(tracker.len(), 2);
            assert_eq!(tracker.velocity(), (200.0 - 100.0) / ((200.0 - 50.0) / 1000.0));
        }

        #[test]
        fn velocity_negative_scroll_direction() {
            // Scrolling up should produce negative velocity
            let mut tracker = VelocityTracker::with_window(1000.0);
            tracker.push(0.0, 0.0);
            tracker.push(100.0, -500.0); // -500px in 100ms = -5000 px/s
            assert_eq!(tracker.velocity(), -5000.0);
        }

        #[test]
        fn velocity_full_ring_buffer_overwrites_oldest() {
            // Push 33+ samples to test ring buffer overflow (CAP=32)
            let mut tracker = VelocityTracker::with_window(10000.0);
            for i in 0..35 {
                tracker.push(i as f64 * 10.0, i as f64);
            }
            // The oldest samples (0, 1, ...) should be evicted
            // Velocity should use only the last 32 samples
            assert!(tracker.velocity() > 0.0);
            // len should never exceed CAP
            assert_eq!(tracker.len(), 32);
        }

        #[test]
        fn velocity_zero_after_window_expiry() {
            // Samples older than window should be evicted
            let mut tracker = VelocityTracker::with_window(50.0);
            tracker.push(0.0, 0.0);
            tracker.push(100.0, 500.0); // 100ms later, window is 50ms → first sample evicted
            // Only 1 sample left → velocity 0
            assert_eq!(tracker.velocity(), 0.0);
        }

        #[test]
        fn velocity_concurrent_push_and_query() {
            // Push and query in interleaved fashion
            let mut tracker = VelocityTracker::with_window(1000.0);
            tracker.push(0.0, 0.0);
            assert_eq!(tracker.velocity(), 0.0); // 1 sample
            tracker.push(50.0, 100.0);
            assert_eq!(tracker.velocity(), 2000.0); // 100px / 0.05s
            tracker.push(100.0, 200.0);
            assert_eq!(tracker.velocity(), 2000.0); // 200px / 0.1s
            tracker.push(150.0, 150.0); // direction reversal
            assert!(tracker.velocity() < 2000.0); // velocity drops
        }
    }

    mod velocity_idle_drop {
        use super::*;

        #[test]
        fn velocity_zero_after_idle() {
            // GSAP-parity: after 500ms of inactivity the velocity drops to 0 so
            // stale samples (e.g. from a paused rAF loop) don't report motion.
            let mut tracker = VelocityTracker::with_window(2000.0);
            tracker.push(0.0, 0.0);
            tracker.push(100.0, 500.0);
            // 600ms after the newest sample (> 500ms idle threshold) → 0.0.
            assert_eq!(tracker.velocity_now(700.0), 0.0);
            // 400ms after the newest sample (< 500ms) → real velocity
            // (500px over 0.1s = 5000 px/s).
            assert_eq!(tracker.velocity_now(500.0), 5000.0);
        }

        #[test]
        fn velocity_now_after_idle_drops_to_zero() {
            // velocity_now should return 0 when now is >500ms after last sample
            let mut tracker = VelocityTracker::with_window(1000.0);
            tracker.push(0.0, 0.0);
            tracker.push(100.0, 500.0);
            // velocity at t=100 is 5000 px/s
            assert_eq!(tracker.velocity_now(100.0), 5000.0);
            // velocity at t=701 (601ms after last sample) should be 0
            assert_eq!(tracker.velocity_now(701.0), 0.0);
            // velocity at t=600 (exactly 500ms after last sample): the guard is
            // `now - back_time > 500.0`, so exactly 500ms is NOT dropped — the
            // boundary is exclusive (mirrors the existing velocity_zero_after_idle
            // test which asserts 5000 at t=600 / back_time=100).
            assert_eq!(tracker.velocity_now(600.0), 5000.0);
            // velocity at t=601 (501ms after last sample) should be 0
            assert_eq!(tracker.velocity_now(601.0), 0.0);
            // velocity at t=599 (499ms after last sample) should still be 5000
            assert_eq!(tracker.velocity_now(599.0), 5000.0);
        }
    }
}