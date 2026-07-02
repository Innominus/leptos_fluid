//! `ScrollTrigger` runtime handle and lifecycle.
//!
//! Mirrors `AnimationController` in `crates/motion/src/controller.rs`: a
//! `#[derive(Clone, Copy)]` handle wrapping `StoredValue<ScrollTriggerInner>`,
//! with `RwSignal`s for reactive outputs and `StoredValue<..., LocalStorage>`
//! for non-reactive interior state. The shared scroll engine (see
//! [`crate::engine`]) calls back into the inner state via
//! [`ScrollTrigger::engine_update`] on each rAF tick.

use std::rc::Rc;

use leptos::html::ElementType;
use leptos::prelude::{
    GetUntracked, GetValue, LocalStorage, NodeRef, ReadValue, RwSignal, Set, SetValue, Signal,
    StoredValue, WriteValue, on_cleanup,
};
use leptos::wasm_bindgen::JsCast;
use web_sys::Element;

use crate::callbacks::{ScrollTriggerEvent, VelocityTracker};
use crate::config::{ScrollTriggerConfig, Scrub};
use crate::engine;
use crate::position::{Rect, clamp_value, resolve_start};
use crate::scroller::Scroller;
use crate::toggle::TogglePhase;

/// Convergence epsilon for `Scrub::Number` smoothing: when the scrub current
/// value is within `SCRUB_CONVERGENCE_EPS` of the raw target, the trigger snaps
/// to raw and the engine stops self-rescheduling the rAF loop.
const SCRUB_CONVERGENCE_EPS: f64 = 1e-4;
/// Cap on per-frame `dt` for `Scrub::Number` smoothing. A backgrounded tab
/// pauses rAF; on resume the first `dt` can be many seconds, which without a
/// cap drives the expo-tween alpha to 1.0 and snaps instantly. Capping at 0.1s
/// preserves the smooth catch-up feel (mirrors GSAP's tween dt clamping).
const SCRUB_DT_CAP_SECS: f64 = 0.1;

#[derive(Clone)]
enum TriggerTarget {
    Element(Element),
    Resolver(Rc<dyn Fn() -> Option<Element>>),
}

impl TriggerTarget {
    fn resolve(&self) -> Option<Element> {
        match self {
            TriggerTarget::Element(element) => Some(element.clone()),
            TriggerTarget::Resolver(resolver) => resolver(),
        }
    }
}

/// A stable target that can be attached to a [`ScrollTrigger`].
///
/// Mirrors `ControllerTarget` in `crates/motion/src/controller.rs`: limited to
/// concrete elements and `NodeRef`s. Dynamic lookup belongs to
/// [`ScrollTrigger::attach_resolver`].
pub trait TriggerTargetSource {
    fn attach_to(self, trigger: ScrollTrigger);
}

impl TriggerTargetSource for Element {
    fn attach_to(self, trigger: ScrollTrigger) {
        trigger.attach_element(self);
    }
}

impl<E> TriggerTargetSource for NodeRef<E>
where
    E: ElementType,
    E::Output: JsCast + Clone + 'static,
{
    fn attach_to(self, trigger: ScrollTrigger) {
        trigger.attach_node_ref(self);
    }
}

#[derive(Clone)]
pub(crate) struct ScrollTriggerInner {
    config: ScrollTriggerConfig,
    scroller: Scroller,
    target_source: StoredValue<Option<TriggerTarget>, LocalStorage>,
    start_pixels: StoredValue<f64, LocalStorage>,
    end_pixels: StoredValue<f64, LocalStorage>,
    pub(crate) progress: RwSignal<f64>,
    direction: RwSignal<i8>,
    pub(crate) is_active: RwSignal<bool>,
    velocity: RwSignal<f64>,
    scrub_current: StoredValue<f64, LocalStorage>,
    scrub_target: StoredValue<f64, LocalStorage>,
    scrub_last_ms: StoredValue<Option<f64>, LocalStorage>,
    /// Tracks the converged state of `Scrub::Number` smoothing for the
    /// `on_scrub_complete` callback: starts `true` so the first snap from
    /// `dt=0` doesn't fire; transitions to `false` while easing, back to
    /// `true` on convergence (firing the callback on that edge).
    scrub_converged: StoredValue<bool, LocalStorage>,
    registration_id: StoredValue<Option<u32>, LocalStorage>,
    enabled: StoredValue<bool, LocalStorage>,
    killed: StoredValue<bool, LocalStorage>,
    velocity_tracker: StoredValue<VelocityTracker, LocalStorage>,
    prev_active: StoredValue<bool, LocalStorage>,
    prev_progress: StoredValue<f64, LocalStorage>,
}

impl ScrollTrigger {
    /// Constructs a host-only trigger for reactive tests. The trigger has no
    /// target, no engine registration, and no `on_cleanup` hook; tests drive the
    /// `progress` / `is_active` / `direction` signals directly.
    #[cfg(test)]
    pub(crate) fn host_test_trigger(
        config: ScrollTriggerConfig,
        progress: f64,
        is_active: bool,
        direction: i8,
    ) -> Self {
        let inner = ScrollTriggerInner {
            config,
            scroller: Scroller::viewport(),
            target_source: StoredValue::new_local(None),
            start_pixels: StoredValue::new_local(0.0),
            end_pixels: StoredValue::new_local(0.0),
            progress: RwSignal::new(progress),
            direction: RwSignal::new(direction),
            is_active: RwSignal::new(is_active),
            velocity: RwSignal::new(0.0),
            scrub_current: StoredValue::new_local(0.0),
            scrub_target: StoredValue::new_local(0.0),
            scrub_last_ms: StoredValue::new_local(None),
            scrub_converged: StoredValue::new_local(true),
            registration_id: StoredValue::new_local(None),
            enabled: StoredValue::new_local(true),
            killed: StoredValue::new_local(false),
            velocity_tracker: StoredValue::new_local(VelocityTracker::new()),
            prev_active: StoredValue::new_local(false),
            prev_progress: StoredValue::new_local(0.0),
        };
        Self {
            inner: StoredValue::new_local(inner),
        }
    }
}

/// Element-agnostic scroll trigger.
///
/// `ScrollTrigger` separates the *what* (config + callbacks) from *where* (a
/// concrete `Element` or a resolver closure). The shared scroll engine batches
/// scroll/resize updates and calls back into the trigger on each rAF tick.
///
/// Typical flow:
///
/// 1. Create a trigger with [`ScrollTrigger::create`] (or [`ScrollTrigger::new`]).
/// 2. Attach a target (`attach_node_ref`, `attach_element`, or `attach_resolver`)
///    either before or via the `target` argument to `create`.
/// 3. Read reactive signals (`progress`, `direction`, `is_active`, `velocity`)
///    or rely on the configured callbacks.
///
/// The trigger is cleaned up automatically when its reactive owner scope dies
/// (via `on_cleanup`); explicit [`ScrollTrigger::kill`] is also supported.
#[derive(Clone, Copy)]
pub struct ScrollTrigger {
    pub(crate) inner: StoredValue<ScrollTriggerInner, LocalStorage>,
}

impl ScrollTrigger {
    /// Creates a trigger from `config` and attaches `target`.
    ///
    /// The trigger registers with the shared scroll engine and runs an initial
    /// [`ScrollTrigger::refresh`] to compute `start`/`end` pixels. An `on_cleanup`
    /// hook unregisters the trigger when the current reactive owner scope dies.
    pub fn create(config: ScrollTriggerConfig, target: impl TriggerTargetSource) -> Self {
        let trigger = Self::with_config(config);
        target.attach_to(trigger);
        trigger.refresh();
        trigger
    }

    /// Builds the trigger inner from `config`, registers with the shared scroll
    /// engine, and installs an `on_cleanup` hook, but does NOT attach a target.
    ///
    /// This is the builder entry point: [`crate::ScrollTriggerBuilder::install`]
    /// calls this, then attaches the deferred target and optional motion bindings
    /// before running [`ScrollTrigger::refresh`]. Public callers should use
    /// [`ScrollTrigger::create`] instead.
    #[cfg_attr(not(any(feature = "builders", feature = "macros")), allow(dead_code))]
    pub(crate) fn with_config(config: ScrollTriggerConfig) -> Self {
        let inner = ScrollTriggerInner {
            config,
            scroller: Scroller::viewport(),
            target_source: StoredValue::new_local(None),
            start_pixels: StoredValue::new_local(0.0),
            end_pixels: StoredValue::new_local(0.0),
            progress: RwSignal::new(0.0),
            direction: RwSignal::new(0),
            is_active: RwSignal::new(false),
            velocity: RwSignal::new(0.0),
            scrub_current: StoredValue::new_local(0.0),
            scrub_target: StoredValue::new_local(0.0),
            scrub_last_ms: StoredValue::new_local(None),
            scrub_converged: StoredValue::new_local(true),
            registration_id: StoredValue::new_local(None),
            enabled: StoredValue::new_local(true),
            killed: StoredValue::new_local(false),
            velocity_tracker: StoredValue::new_local(VelocityTracker::new()),
            prev_active: StoredValue::new_local(false),
            prev_progress: StoredValue::new_local(0.0),
        };
        let trigger = Self {
            inner: StoredValue::new_local(inner),
        };
        let id = engine::register(trigger);
        trigger.inner.write_value().registration_id.set_value(Some(id));

        let cleanup_trigger = trigger;
        on_cleanup(move || {
            cleanup_trigger.kill();
        });

        trigger
    }

    /// Convenience alias for [`ScrollTrigger::create`].
    pub fn new(config: ScrollTriggerConfig, target: impl TriggerTargetSource) -> Self {
        Self::create(config, target)
    }

    /// Attaches a concrete DOM element target.
    pub fn attach_element(&self, element: Element) {
        self.inner
            .write_value()
            .target_source
            .set_value(Some(TriggerTarget::Element(element)));
    }

    /// Attaches a `NodeRef` and resolves it on each refresh/tick.
    pub fn attach_node_ref<E>(&self, node_ref: NodeRef<E>)
    where
        E: ElementType,
        E::Output: JsCast + Clone + 'static,
    {
        self.attach_resolver(move || node_ref.get_untracked().map(|node| node.unchecked_into()));
    }

    /// Attaches a resolver closure that returns the current target element.
    pub fn attach_resolver<F>(&self, resolver: F)
    where
        F: Fn() -> Option<Element> + 'static,
    {
        self.inner
            .write_value()
            .target_source
            .set_value(Some(TriggerTarget::Resolver(Rc::new(resolver))));
    }

    /// Permanently kills the trigger: unregisters from the engine and marks it
    /// killed. Idempotent.
    pub fn kill(self) {
        let inner = self.inner.write_value();
        if inner.killed.get_value() {
            return;
        }
        inner.killed.set_value(true);
        inner.enabled.set_value(false);
        let id = inner.registration_id.get_value();
        drop(inner);
        if let Some(id) = id {
            engine::unregister(id);
        }
    }

    /// Temporarily disables the trigger; the engine skips it while disabled.
    pub fn disable(&self) {
        self.inner.write_value().enabled.set_value(false);
    }

    /// Re-enables a disabled trigger and recomputes geometry.
    pub fn enable(&self) {
        self.inner.write_value().enabled.set_value(true);
        self.refresh();
    }

    /// Recomputes `start`/`end` pixels from current geometry and re-evaluates
    /// progress. Dispatches `on_refresh`.
    pub fn refresh(&self) {
        let inner = self.inner.read_value();
        if inner.killed.get_value() {
            return;
        }
        let scroller = inner.scroller.clone();
        let viewport_size = scroller.viewport_size();
        let config = inner.config.clone();

        let trigger_rect = match inner.target_source.get_value() {
            Some(target) => match target.resolve() {
                Some(element) => {
                    let rect = element.get_bounding_client_rect();
                    // `get_bounding_client_rect().top` is viewport-relative
                    // (negative for elements above the current viewport).
                    // `scroller.scroll_position()` is the absolute document
                    // scroll offset, and `raw_progress` compares the two, so
                    // we must convert the rect to document-absolute coordinates
                    // by adding the current scroll position. Without this,
                    // reloading the page mid-scroll (browser restores scroll
                    // position) produces wildly wrong start/end pixels and
                    // every trigger thinks it has already been scrolled past.
                    let scroll_offset = scroller.scroll_position();
                    Rect {
                        start: rect.top() as f64 + scroll_offset,
                        size: rect.height() as f64,
                    }
                }
                None => Rect { start: 0.0, size: 0.0 },
            },
            None => Rect { start: 0.0, size: 0.0 },
        };

        let positions = config.parse_positions(false);
        let (start_px, end_px) = match positions {
            Some((start_pos, end_pos)) => {
                let s = resolve_start(trigger_rect, viewport_size, &start_pos);
                let e = resolve_end_pixels(trigger_rect, viewport_size, s, &end_pos);
                (s, e)
            }
            None => (0.0, 0.0),
        };

        inner.start_pixels.set_value(start_px);
        inner.end_pixels.set_value(end_px);

        let scroll_pos = scroller.scroll_position();
        let raw = raw_progress(scroll_pos, start_px, end_px);
        let clamped = clamp_value(raw, 0.0, 1.0);
        // Only fire the reactive signal when the value actually changes.
        // Without this guard, refresh() calls progress.set()/is_active.set()
        // even when the smoothed value hasn't changed, causing subscribed
        // Effects (bind_controller, style: bindings, class: bindings) to
        // fire unnecessarily → set_immediate → cancel_active_animation +
        // apply_style per frame per idle trigger.
        if clamped != inner.progress.get_untracked() {
            inner.progress.set(clamped);
        }
        let active = start_px <= scroll_pos && scroll_pos <= end_px;
        if active != inner.is_active.get_untracked() {
            inner.is_active.set(active);
        }

        if let Some(cb) = inner.config.on_refresh.as_ref() {
            let event = ScrollTriggerEvent::new(
                clamped,
                inner.direction.get_untracked(),
                active,
                inner.velocity.get_untracked(),
            );
            cb(event);
        }
    }

    /// Reactive scroll progress in `0.0..=1.0`.
    pub fn progress(&self) -> Signal<f64> {
        self.inner.read_value().progress.into()
    }

    /// Reactive scroll direction: `1` forward, `-1` backward, `0` initial.
    pub fn direction(&self) -> Signal<i8> {
        self.inner.read_value().direction.into()
    }

    /// Reactive flag: `true` while the scroll position is within the active
    /// range.
    pub fn is_active(&self) -> Signal<bool> {
        self.inner.read_value().is_active.into()
    }

    /// Reactive scroll velocity in pixels per second.
    pub fn velocity(&self) -> Signal<f64> {
        self.inner.read_value().velocity.into()
    }

    /// Cached start position in pixels.
    pub fn start(&self) -> f64 {
        self.inner.read_value().start_pixels.get_value()
    }

    /// Cached end position in pixels.
    pub fn end(&self) -> f64 {
        self.inner.read_value().end_pixels.get_value()
    }

    /// Current scroll position of the scroller in pixels.
    pub fn scroll_position(&self) -> f64 {
        self.inner.read_value().scroller.scroll_position()
    }

    /// Current scroll velocity in pixels per second, read from the trigger's
    /// velocity tracker.
    pub fn get_velocity(&self) -> f64 {
        self.inner
            .read_value()
            .velocity_tracker
            .get_value()
            .velocity()
    }

    /// Resets the per-trigger scrub clock so the next `step_scrub` call uses
    /// `dt = 0` instead of a stale timestamp. Called by the engine on
    /// visibility regain (tab hidden → visible) to avoid a huge first dt.
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub(crate) fn reset_scrub_clock(&self) {
        self.inner
            .read_value()
            .scrub_last_ms
            .set_value(None);
    }

    /// Returns `true` once the trigger has been killed.
    pub(crate) fn is_killed(&self) -> bool {
        self.inner.read_value().killed.get_value()
    }

    /// The per-trigger update invoked by the shared scroll engine on each rAF
    /// tick. Computes raw progress, detects phase transitions, dispatches
    /// callbacks, updates reactive signals, and steps scrub smoothing.
    ///
    /// Returns `true` if this trigger uses `Scrub::Number` smoothing and hasn't
    /// yet converged to its target — the engine uses this to self-reschedule
    /// the rAF loop so smoothing advances every frame (not just on scroll
    /// events). Returns `false` for `Scrub::Bool` (direct 1:1 or callbacks-only)
    /// and after a `once` kill.
    pub(crate) fn engine_update(&self, scroll_pos: f64, velocity: f64, now_ms: f64) -> bool {
        let inner = self.inner.read_value();
        if inner.killed.get_value() || !inner.enabled.get_value() {
            return false;
        }

        let start_px = inner.start_pixels.get_value();
        let end_px = inner.end_pixels.get_value();
        let raw = raw_progress(scroll_pos, start_px, end_px);
        let clamped = clamp_value(raw, 0.0, 1.0);
        let active = start_px <= scroll_pos && scroll_pos <= end_px;

        let prev_progress = inner.prev_progress.get_value();
        let prev_active = inner.prev_active.get_value();
        let direction_sign = if clamped > prev_progress {
            1i8
        } else if clamped < prev_progress {
            -1i8
        } else {
            inner.direction.get_untracked()
        };

        // GSAP-parity: guard direction/velocity signal writes so idle ticks
        // (scroll position unchanged → velocity 0) don't churn subscribers.
        if direction_sign != inner.direction.get_untracked() {
            inner.direction.set(direction_sign);
        }
        if velocity != inner.velocity.get_untracked() {
            inner.velocity.set(velocity);
        }
        inner.velocity_tracker.write_value().push(now_ms, scroll_pos);

        let progress_changed = clamped != prev_progress;
        let active_changed = active != prev_active;

        let event = ScrollTriggerEvent::new(clamped, direction_sign, active, velocity);

        if active_changed {
            let phase = phase_transition(prev_active, active, direction_sign);
            dispatch_phase(&inner.config, phase, event);
            if let Some(cb) = inner.config.on_toggle.as_ref() {
                cb(event);
            }
        }

        if progress_changed {
            if let Some(cb) = inner.config.on_update.as_ref() {
                cb(event);
            }
        }

        let (exposed_progress, just_converged) = if engine::reduced_motion_snaps_scrub() {
            // GSAP-parity: `prefers-reduced-motion: reduce` snaps `Scrub::Number`
            // to raw progress, skipping the continuous smoothing rAF loop. Keep
            // `scrub_target`/`scrub_current`/`scrub_last_ms` in sync with `raw`
            // so a later switch back to `Ignore` doesn't snap from a stale
            // value. `scrub_converged = true` suppresses `on_scrub_complete`
            // (this is a snap, not a convergence). Phase callbacks above still
            // fire.
            inner.scrub_target.set_value(clamped);
            inner.scrub_current.set_value(clamped);
            inner.scrub_last_ms.set_value(Some(now_ms));
            inner.scrub_converged.set_value(true);
            (clamped, false)
        } else {
            step_scrub(&inner, clamped, now_ms)
        };
        // GSAP-parity: fire on_scrub_complete the first time a Scrub::Number
        // trigger settles after being non-converged (the expo-tween edge).
        if just_converged {
            if let Some(cb) = inner.config.on_scrub_complete.as_ref() {
                cb(event);
            }
        }
        // Only fire the reactive signal when the value actually changes.
        // Without this guard, every rAF tick calls progress.set() for every
        // registered trigger even when the smoothed value hasn't changed
        // (triggers outside their active range stay at 0.0 or 1.0), causing
        // all subscribed Effects (bind_controller, style: bindings, class:
        // bindings) to fire unnecessarily → set_immediate →
        // cancel_active_animation + apply_style per frame per idle trigger.
        if exposed_progress != inner.progress.get_untracked() {
            inner.progress.set(exposed_progress);
        }
        if active != inner.is_active.get_untracked() {
            inner.is_active.set(active);
        }
        inner.prev_progress.set_value(clamped);
        inner.prev_active.set_value(active);

        // GSAP-parity: `once` kills on any deactivation regardless of
        // direction (was previously gated on `direction_sign == 1`).
        if inner.config.once && active_changed && !active {
            drop(inner);
            self.kill();
            return false;
        }

        // Self-reschedule signal: only Scrub::Number smoothing needs continuous
        // rAF frames; converged triggers stop the loop. Reduced-motion snaps to
        // raw, so no continuous rAF is needed in that posture either.
        !engine::reduced_motion_snaps_scrub()
            && matches!(inner.config.scrub, Scrub::Number(_))
            && (inner.scrub_target.get_value() - inner.scrub_current.get_value()).abs()
                > SCRUB_CONVERGENCE_EPS
    }
}

fn resolve_end_pixels(
    trigger_rect: Rect,
    viewport_size: f64,
    start_px: f64,
    end_pos: &crate::position::ScrollPosition,
) -> f64 {
    use crate::position::ScrollOffset;
    match &end_pos.scroller {
        ScrollOffset::Relative {
            pixels,
            percent_of_scroller,
        } => {
            if *percent_of_scroller {
                start_px + viewport_size * (*pixels / 100.0)
            } else {
                start_px + *pixels
            }
        }
        _ => resolve_start(trigger_rect, viewport_size, end_pos),
    }
}

fn raw_progress(scroll_pos: f64, start_px: f64, end_px: f64) -> f64 {
    if (end_px - start_px).abs() <= f64::EPSILON {
        return if scroll_pos >= end_px { 1.0 } else { 0.0 };
    }
    (scroll_pos - start_px) / (end_px - start_px)
}

fn phase_transition(prev_active: bool, active: bool, direction: i8) -> TogglePhase {
    match (prev_active, active, direction) {
        (false, true, 1) => TogglePhase::OnEnter,
        (true, false, 1) => TogglePhase::OnLeave,
        (false, true, -1) => TogglePhase::OnEnterBack,
        (true, false, -1) => TogglePhase::OnLeaveBack,
        _ => TogglePhase::OnEnter,
    }
}

fn dispatch_phase(config: &ScrollTriggerConfig, phase: TogglePhase, event: ScrollTriggerEvent) {
    let cb = match phase {
        TogglePhase::OnEnter => config.on_enter.as_ref(),
        TogglePhase::OnLeave => config.on_leave.as_ref(),
        TogglePhase::OnEnterBack => config.on_enter_back.as_ref(),
        TogglePhase::OnLeaveBack => config.on_leave_back.as_ref(),
    };
    if let Some(cb) = cb {
        cb(event);
    }
}

/// Advances `Scrub::Number` smoothing by one frame and returns the exposed
/// progress plus a flag that is `true` when the trigger converged on this call
/// (i.e. `|raw - next| < SCRUB_CONVERGENCE_EPS` after being non-converged).
/// The caller (`engine_update`) uses the flag to fire `on_scrub_complete`.
fn step_scrub(inner: &ScrollTriggerInner, raw: f64, now_ms: f64) -> (f64, bool) {
    match inner.config.scrub {
        Scrub::Bool(false) => (raw, false),
        Scrub::Bool(true) => {
            inner.scrub_current.set_value(raw);
            (raw, false)
        }
        Scrub::Number(t) => {
            inner.scrub_target.set_value(raw);
            let current = inner.scrub_current.get_value();
            let last = inner.scrub_last_ms.get_value();
            // Cap dt so a backgrounded tab (rAF paused, then resumed) doesn't
            // produce a huge dt → alpha → 1.0 → instant snap (GSAP-parity).
            let dt = match last {
                Some(prev) => (((now_ms - prev) / 1000.0).max(0.0)).min(SCRUB_DT_CAP_SECS),
                None => 0.0,
            };
            let next = if dt <= 0.0 || t <= 0.0 {
                raw
            } else {
                let alpha = 1.0 - (-dt / t).exp();
                current + (raw - current) * alpha
            };
            if (raw - next).abs() < SCRUB_CONVERGENCE_EPS {
                inner.scrub_current.set_value(raw);
                inner.scrub_last_ms.set_value(Some(now_ms));
                // Fire on_scrub_complete on the false → true convergence edge.
                let was_converged = inner.scrub_converged.get_value();
                inner.scrub_converged.set_value(true);
                (raw, !was_converged)
            } else {
                inner.scrub_current.set_value(next);
                inner.scrub_last_ms.set_value(Some(now_ms));
                inner.scrub_converged.set_value(false);
                (next, false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ScrollTriggerConfig;
    use crate::position::{ScrollOffset, ScrollPosition};
    use std::cell::Cell;
    use std::rc::Rc;

    /// Builds a `ScrollTriggerInner` with `scrub = Scrub::Bool(false)` and the
    /// given scrub state. Mirrors the literal used by the existing scrub tests
    /// so each test stays self-contained without duplicating the full struct
    /// field list.
    fn inner_with_scrub(
        scrub: Scrub,
        scrub_current: f64,
        scrub_target: f64,
        scrub_last_ms: Option<f64>,
        scrub_converged: bool,
    ) -> ScrollTriggerInner {
        ScrollTriggerInner {
            config: ScrollTriggerConfig::default().scrub(scrub),
            scroller: Scroller::viewport(),
            target_source: StoredValue::new_local(None),
            start_pixels: StoredValue::new_local(0.0),
            end_pixels: StoredValue::new_local(0.0),
            progress: RwSignal::new(0.0),
            direction: RwSignal::new(0),
            is_active: RwSignal::new(false),
            velocity: RwSignal::new(0.0),
            scrub_current: StoredValue::new_local(scrub_current),
            scrub_target: StoredValue::new_local(scrub_target),
            scrub_last_ms: StoredValue::new_local(scrub_last_ms),
            scrub_converged: StoredValue::new_local(scrub_converged),
            registration_id: StoredValue::new_local(None),
            enabled: StoredValue::new_local(true),
            killed: StoredValue::new_local(false),
            velocity_tracker: StoredValue::new_local(VelocityTracker::new()),
            prev_active: StoredValue::new_local(false),
            prev_progress: StoredValue::new_local(0.0),
        }
    }

    mod raw_progress {
        use super::*;

        #[test]
        fn raw_progress_clamps_at_boundaries() {
            assert!((raw_progress(100.0, 100.0, 200.0) - 0.0).abs() < 1e-9);
            assert!((raw_progress(150.0, 100.0, 200.0) - 0.5).abs() < 1e-9);
            assert!((raw_progress(250.0, 100.0, 200.0) - 1.5).abs() < 1e-9);
        }

        #[test]
        fn raw_progress_handles_zero_range() {
            assert_eq!(raw_progress(50.0, 100.0, 100.0), 0.0);
            assert_eq!(raw_progress(150.0, 100.0, 100.0), 1.0);
        }
    }

    mod phase_transitions {
        use super::*;

        #[test]
        fn phase_transition_maps_correctly() {
            assert_eq!(phase_transition(false, true, 1), TogglePhase::OnEnter);
            assert_eq!(phase_transition(true, false, 1), TogglePhase::OnLeave);
            assert_eq!(phase_transition(false, true, -1), TogglePhase::OnEnterBack);
            assert_eq!(phase_transition(true, false, -1), TogglePhase::OnLeaveBack);
        }
    }

    mod resolve_end_pixels {
        use super::*;

        #[test]
        fn resolve_end_pixels_absolute_uses_resolve_start() {
            let rect = Rect { start: 1000.0, size: 200.0 };
            let end_pos = ScrollPosition {
                trigger: crate::position::ScrollPoint::Bottom,
                scroller: ScrollOffset::Absolute(crate::position::ScrollPoint::Top),
            };
            assert_eq!(resolve_end_pixels(rect, 800.0, 1000.0, &end_pos), 1200.0);
        }

        #[test]
        fn resolve_end_pixels_relative_pixels_adds_to_start() {
            let rect = Rect { start: 1000.0, size: 200.0 };
            let end_pos = ScrollPosition {
                trigger: crate::position::ScrollPoint::Top,
                scroller: ScrollOffset::Relative {
                    pixels: 300.0,
                    percent_of_scroller: false,
                },
            };
            assert_eq!(resolve_end_pixels(rect, 800.0, 1000.0, &end_pos), 1300.0);
        }

        #[test]
        fn resolve_end_pixels_relative_percent_uses_viewport() {
            let rect = Rect { start: 1000.0, size: 200.0 };
            let end_pos = ScrollPosition {
                trigger: crate::position::ScrollPoint::Top,
                scroller: ScrollOffset::Relative {
                    pixels: 50.0,
                    percent_of_scroller: true,
                },
            };
            assert_eq!(resolve_end_pixels(rect, 800.0, 1000.0, &end_pos), 1400.0);
        }
    }

    mod scrub_smoothing {
        use super::*;

        #[test]
        fn scrub_bool_false_exposes_raw() {
            let inner = inner_with_scrub(Scrub::Bool(false), 0.0, 0.0, None, true);
            assert_eq!(step_scrub(&inner, 0.7, 1000.0).0, 0.7);
        }

        #[test]
        fn scrub_bool_true_sets_scrub_current_to_raw() {
            // Scrub::Bool(true) should set scrub_current = raw and return raw (1:1)
            let inner = inner_with_scrub(Scrub::Bool(true), 0.5, 0.5, None, true);
            let (val, converged) = step_scrub(&inner, 0.7, 1000.0);
            assert_eq!(val, 0.7);
            assert!(!converged); // Bool(true) never signals convergence
            assert_eq!(inner.scrub_current.get_value(), 0.7); // scrub_current was updated
        }

        #[test]
        fn scrub_number_eases_toward_target() {
            let inner = inner_with_scrub(Scrub::Number(0.3), 0.0, 0.0, Some(1000.0), true);
            let first = step_scrub(&inner, 1.0, 1050.0).0;
            assert!(first > 0.0 && first < 1.0);
            let second = step_scrub(&inner, 1.0, 1100.0).0;
            assert!(second > first && second < 1.0);
        }

        #[test]
        fn scrub_number_converges_to_target() {
            // After many steps with the same target, scrub should converge
            let inner = inner_with_scrub(Scrub::Number(0.15), 0.0, 0.0, Some(1000.0), true);
            let mut current = 0.0;
            // Start at t=1016 so the first frame has dt=0.016s (nonzero), which
            // eases current toward target instead of snapping dt=0 → next=raw=1.0
            // while scrub_converged is still `true` (which would suppress the
            // false→true convergence edge).
            let mut time = 1016.0;
            // Step toward target=1.0 for 120 frames at 16ms intervals
            for _ in 0..120 {
                let (val, converged) = step_scrub(&inner, 1.0, time);
                current = val;
                time += 16.0;
                if converged {
                    // Once converged, value should be exactly the target
                    assert!(
                        (current - 1.0).abs() < 1e-4,
                        "should converge to target, got {}",
                        current
                    );
                    return;
                }
            }
            // After 120 frames at t=0.15, should have converged
            panic!("did not converge after 120 frames, last value: {}", current);
        }

        #[test]
        fn scrub_number_reverses_direction() {
            // When target changes direction, scrub should follow without overshooting
            let inner = inner_with_scrub(Scrub::Number(0.3), 0.0, 0.0, Some(1000.0), true);
            // Step toward 1.0
            let (v1, _) = step_scrub(&inner, 1.0, 1050.0);
            assert!(v1 > 0.0 && v1 < 1.0);
            // Now step toward 0.0 (direction reversal)
            let (v2, _) = step_scrub(&inner, 0.0, 1100.0);
            assert!(v2 < v1, "reversal should decrease: v1={}, v2={}", v1, v2);
            assert!(v2 >= 0.0, "should not overshoot below 0.0: v2={}", v2);
        }

        #[test]
        fn scrub_number_dt_cap_prevents_instant_snap() {
            // When dt is very large (tab was hidden), the cap should prevent instant snap
            let inner = inner_with_scrub(Scrub::Number(0.15), 0.0, 0.0, Some(1000.0), true);
            // dt = 10 seconds (600 frames at 60fps — simulates tab hidden for 10s)
            let (val, _) = step_scrub(&inner, 1.0, 11000.0);
            // Without the cap, alpha = 1 - exp(-10/0.15) ≈ 1.0 → instant snap to 1.0
            // With the cap (0.1s), alpha = 1 - exp(-0.1/0.15) ≈ 0.487 → partial step
            assert!(val < 1.0, "dt cap should prevent instant snap, got {}", val);
            assert!(val > 0.0, "should still make progress, got {}", val);
        }
    }

    mod reduced_motion {
        use super::*;

        // GSAP-parity: documents that `prefers-reduced-motion` accommodation is a
        // wasm-only behavior. On host targets `reduced_motion_snaps_scrub()` always
        // returns `false`, so the smoothing-skip path never fires in host tests —
        // the real gating is exercised in browser wasm.
        #[test]
        fn reduced_motion_no_op_on_host() {
            engine::set_reduced_motion(crate::config::ReducedMotion::Respect);
            assert!(!engine::reduced_motion_snaps_scrub());
            engine::set_reduced_motion(crate::config::ReducedMotion::Ignore);
            assert!(!engine::reduced_motion_snaps_scrub());
        }
    }

    mod lifecycle {
        use super::*;

        #[test]
        fn kill_is_idempotent() {
            let trigger =
                ScrollTrigger::host_test_trigger(ScrollTriggerConfig::default(), 0.0, false, 0);
            trigger.kill();
            assert!(trigger.is_killed());
            // Second kill should not panic
            trigger.kill();
            assert!(trigger.is_killed());
        }

        #[test]
        fn kill_sets_killed_flag() {
            let trigger =
                ScrollTrigger::host_test_trigger(ScrollTriggerConfig::default(), 0.0, false, 0);
            assert!(!trigger.is_killed());
            trigger.kill();
            assert!(trigger.is_killed());
        }

        #[test]
        fn disabled_trigger_engine_update_returns_false() {
            let trigger = ScrollTrigger::host_test_trigger(
                ScrollTriggerConfig::default().scrub(Scrub::Bool(true)),
                0.0,
                false,
                0,
            );
            trigger.disable();
            // engine_update should return false (not needs_more_smoothing) when disabled
            let needs_more = trigger.engine_update(100.0, 0.0, 1000.0);
            assert!(!needs_more);
        }

        #[test]
        fn killed_trigger_engine_update_returns_false() {
            let trigger = ScrollTrigger::host_test_trigger(
                ScrollTriggerConfig::default().scrub(Scrub::Bool(true)),
                0.0,
                false,
                0,
            );
            trigger.kill();
            let needs_more = trigger.engine_update(100.0, 0.0, 1000.0);
            assert!(!needs_more);
        }

        #[test]
        fn enable_re_enables_trigger() {
            let trigger = ScrollTrigger::host_test_trigger(
                ScrollTriggerConfig::default().scrub(Scrub::Bool(true)),
                0.0,
                false,
                0,
            );
            trigger.disable();
            // engine_update returns false when disabled
            assert!(!trigger.engine_update(100.0, 0.0, 1000.0));
            trigger.enable();
            // After enable, engine_update should work (refresh re-runs, but on host
            // with no target, start/end are 0 so progress is always 1.0 or 0.0)
            // Just verify it doesn't panic and returns something
            let _ = trigger.engine_update(100.0, 0.0, 1000.0);
        }
    }

    mod engine_update {
        use super::*;

        // Helper: build a trigger with the given start/end pixels and previous
        // active/progress state, so `engine_update` calls exercise the active range
        // `[start_px, end_px]`.
        fn trigger_in_range(
            config: ScrollTriggerConfig,
            start_px: f64,
            end_px: f64,
            prev_active: bool,
            prev_progress: f64,
        ) -> ScrollTrigger {
            let trigger = ScrollTrigger::host_test_trigger(config, prev_progress, prev_active, 0);
            let inner = trigger.inner.write_value();
            inner.start_pixels.set_value(start_px);
            inner.end_pixels.set_value(end_px);
            inner.prev_active.set_value(prev_active);
            inner.prev_progress.set_value(prev_progress);
            drop(inner);
            trigger
        }

        #[test]
        fn on_enter_fires_on_forward_entry() {
            let entered = Rc::new(Cell::new(false));
            let entered_clone = entered.clone();
            let config =
                ScrollTriggerConfig::default().on_enter(move |_| entered_clone.set(true));
            let trigger = trigger_in_range(config, 0.0, 100.0, false, 0.0);

            // scroll_pos=50 → in range, active flips false→true, direction=1 (forward)
            trigger.engine_update(50.0, 0.0, 1000.0);

            assert!(entered.get(), "on_enter should have fired");
        }

        #[test]
        fn on_leave_fires_on_forward_exit() {
            let left = Rc::new(Cell::new(false));
            let left_clone = left.clone();
            let config = ScrollTriggerConfig::default().on_leave(move |_| left_clone.set(true));
            // prev_active=true so we start inside; scrolling past end exits forward.
            let trigger = trigger_in_range(config, 0.0, 100.0, true, 0.8);

            // scroll_pos=150 → past end → active false, progress clamped to 1.0 (>0.8) → dir=1
            trigger.engine_update(150.0, 0.0, 1000.0);

            assert!(left.get(), "on_leave should have fired");
        }

        #[test]
        fn on_enter_back_fires_on_backward_entry() {
            let entered = Rc::new(Cell::new(false));
            let entered_clone = entered.clone();
            let config =
                ScrollTriggerConfig::default().on_enter_back(move |_| entered_clone.set(true));
            // Start above the range (scroll_pos > end). prev_progress=1.0 so the
            // first move back into range has direction=-1.
            let trigger = trigger_in_range(config, 0.0, 100.0, false, 1.0);

            // scroll_pos=50 → in range, active false→true, progress 0.5 < 1.0 → dir=-1
            trigger.engine_update(50.0, 0.0, 1000.0);

            assert!(entered.get(), "on_enter_back should have fired");
        }

        #[test]
        fn on_leave_back_fires_on_backward_exit() {
            let left = Rc::new(Cell::new(false));
            let left_clone = left.clone();
            let config =
                ScrollTriggerConfig::default().on_leave_back(move |_| left_clone.set(true));
            // Start in range; scrolling before start exits backward.
            let trigger = trigger_in_range(config, 0.0, 100.0, true, 0.5);

            // scroll_pos=-50 → before start → active false, progress clamped to 0.0 (<0.5) → dir=-1
            trigger.engine_update(-50.0, 0.0, 1000.0);

            assert!(left.get(), "on_leave_back should have fired");
        }

        #[test]
        fn on_toggle_fires_on_active_change() {
            let toggled = Rc::new(Cell::new(false));
            let toggled_clone = toggled.clone();
            let config =
                ScrollTriggerConfig::default().on_toggle(move |_| toggled_clone.set(true));
            let trigger = trigger_in_range(config, 0.0, 100.0, false, 0.0);

            // Entering range flips active false→true → on_toggle fires.
            trigger.engine_update(50.0, 0.0, 1000.0);

            assert!(toggled.get(), "on_toggle should have fired");
        }

        #[test]
        fn on_update_fires_on_progress_change() {
            let updated = Rc::new(Cell::new(false));
            let updated_clone = updated.clone();
            let config =
                ScrollTriggerConfig::default().on_update(move |_| updated_clone.set(true));
            // prev_progress=0.0; moving to scroll_pos=50 in [0,100] → progress 0.5 (changed).
            let trigger = trigger_in_range(config, 0.0, 100.0, true, 0.0);

            trigger.engine_update(50.0, 0.0, 1000.0);

            assert!(updated.get(), "on_update should have fired");
        }

        #[test]
        fn on_update_does_not_fire_on_repeat() {
            let updated = Rc::new(Cell::new(0u32));
            let updated_clone = updated.clone();
            let config =
                ScrollTriggerConfig::default().on_update(move |_| updated_clone.set(
                    updated_clone.get() + 1,
                ));
            // prev_progress=0.5; same scroll_pos → same clamped progress → no fire.
            let trigger = trigger_in_range(config, 0.0, 100.0, true, 0.5);

            trigger.engine_update(50.0, 0.0, 1000.0);

            assert_eq!(
                updated.get(),
                0,
                "on_update should NOT fire when progress is unchanged"
            );
        }

        #[test]
        fn once_kills_on_forward_leave() {
            let config = ScrollTriggerConfig::default().once(true);
            let trigger = trigger_in_range(config, 0.0, 100.0, true, 0.8);

            // scroll past end → active false, direction forward → once kill.
            trigger.engine_update(150.0, 0.0, 1000.0);

            assert!(trigger.is_killed(), "once should kill on forward leave");
        }

        #[test]
        fn once_kills_on_backward_leave() {
            let config = ScrollTriggerConfig::default().once(true);
            let trigger = trigger_in_range(config, 0.0, 100.0, true, 0.5);

            // scroll before start → active false, direction backward → once kill
            // (GSAP-parity: `once` kills on ANY leave regardless of direction).
            trigger.engine_update(-50.0, 0.0, 1000.0);

            assert!(trigger.is_killed(), "once should kill on backward leave");
        }

        #[test]
        fn equality_guard_progress_prevents_signal_set() {
            // Two identical calls should leave the progress signal unchanged —
            // the equality guard suppresses spurious sets. We assert no panic and
            // that the signal value is stable across both calls.
            let trigger = trigger_in_range(
                ScrollTriggerConfig::default().scrub(Scrub::Bool(true)),
                0.0,
                100.0,
                true,
                0.5,
            );

            trigger.engine_update(50.0, 0.0, 1000.0);
            let after_first = trigger.progress().get_untracked();

            trigger.engine_update(50.0, 0.0, 1016.0);
            let after_second = trigger.progress().get_untracked();

            assert_eq!(after_first, after_second);
            assert!((after_first - 0.5).abs() < 1e-9);
        }

        #[test]
        fn equality_guard_is_active_prevents_signal_set() {
            // Two identical calls within the active range leave is_active unchanged.
            let trigger = trigger_in_range(
                ScrollTriggerConfig::default().scrub(Scrub::Bool(true)),
                0.0,
                100.0,
                true,
                0.5,
            );

            trigger.engine_update(50.0, 0.0, 1000.0);
            let after_first = trigger.is_active().get_untracked();

            trigger.engine_update(60.0, 0.0, 1016.0);
            let after_second = trigger.is_active().get_untracked();

            assert_eq!(after_first, after_second);
            assert!(after_first);
        }

        #[test]
        fn needs_more_smoothing_true_for_non_converged_scrub() {
            // Scrub::Number with scrub_current != target → returns true.
            let trigger = ScrollTrigger::host_test_trigger(
                ScrollTriggerConfig::default().scrub(Scrub::Number(0.3)),
                0.5,
                false,
                0,
            );
            {
                let inner = trigger.inner.write_value();
                inner.start_pixels.set_value(0.0);
                inner.end_pixels.set_value(100.0);
                inner.prev_active.set_value(false);
                inner.prev_progress.set_value(0.0);
                inner.scrub_current.set_value(0.5);
                inner.scrub_target.set_value(0.5);
                inner.scrub_last_ms.set_value(Some(1000.0));
                inner.scrub_converged.set_value(true);
                drop(inner);
            }

            // scroll_pos=100 → raw=1.0; scrub_current=0.5 → not converged → true.
            let needs_more = trigger.engine_update(100.0, 0.0, 1016.0);
            assert!(
                needs_more,
                "non-converged Scrub::Number should request more smoothing"
            );
        }

        #[test]
        fn needs_more_smoothing_false_for_converged_scrub() {
            // Scrub::Number already at target → returns false.
            let trigger = ScrollTrigger::host_test_trigger(
                ScrollTriggerConfig::default().scrub(Scrub::Number(0.3)),
                1.0,
                false,
                0,
            );
            {
                let inner = trigger.inner.write_value();
                inner.start_pixels.set_value(0.0);
                inner.end_pixels.set_value(100.0);
                inner.prev_active.set_value(false);
                inner.prev_progress.set_value(1.0);
                inner.scrub_current.set_value(1.0);
                inner.scrub_target.set_value(1.0);
                inner.scrub_last_ms.set_value(Some(1000.0));
                inner.scrub_converged.set_value(true);
                drop(inner);
            }

            // scroll_pos=100 → raw=1.0; scrub_current=1.0 → converged → false.
            let needs_more = trigger.engine_update(100.0, 0.0, 1016.0);
            assert!(
                !needs_more,
                "converged Scrub::Number should not request more smoothing"
            );
        }

        #[test]
        fn needs_more_smoothing_false_for_bool_scrub() {
            // Scrub::Bool(true) → never requests continuous rAF.
            let trigger = ScrollTrigger::host_test_trigger(
                ScrollTriggerConfig::default().scrub(Scrub::Bool(true)),
                0.0,
                false,
                0,
            );
            {
                let inner = trigger.inner.write_value();
                inner.start_pixels.set_value(0.0);
                inner.end_pixels.set_value(100.0);
                inner.prev_active.set_value(false);
                inner.prev_progress.set_value(0.0);
                drop(inner);
            }

            let needs_more = trigger.engine_update(50.0, 0.0, 1000.0);
            assert!(!needs_more, "Scrub::Bool should not request more smoothing");
        }

        #[test]
        fn needs_more_smoothing_false_for_disabled_scrub() {
            // Scrub::Bool(false) → never requests continuous rAF.
            let trigger = ScrollTrigger::host_test_trigger(
                ScrollTriggerConfig::default().scrub(Scrub::Bool(false)),
                0.0,
                false,
                0,
            );
            {
                let inner = trigger.inner.write_value();
                inner.start_pixels.set_value(0.0);
                inner.end_pixels.set_value(100.0);
                inner.prev_active.set_value(false);
                inner.prev_progress.set_value(0.0);
                drop(inner);
            }

            let needs_more = trigger.engine_update(50.0, 0.0, 1000.0);
            assert!(
                !needs_more,
                "Scrub::Bool(false) should not request more smoothing"
            );
        }
    }

    mod callbacks {
        use super::*;

        #[test]
        fn on_scrub_complete_fires_on_convergence() {
            // Scrub::Number, scrub_converged=false, scrub_current==raw (target
            // already equals current). step_scrub sees |raw - next| < eps and flips
            // scrub_converged false→true → on_scrub_complete fires.
            let fired = Rc::new(Cell::new(false));
            let fired_clone = fired.clone();
            let config = ScrollTriggerConfig::default()
                .scrub(Scrub::Number(0.3))
                .on_scrub_complete(move |_| fired_clone.set(true));
            let trigger = ScrollTrigger::host_test_trigger(config, 0.5, true, 1);
            {
                let inner = trigger.inner.write_value();
                inner.start_pixels.set_value(0.0);
                inner.end_pixels.set_value(100.0);
                inner.prev_active.set_value(true);
                inner.prev_progress.set_value(0.5);
                inner.scrub_current.set_value(0.5);
                inner.scrub_target.set_value(0.5);
                // scrub_last_ms = Some(now) → dt=0 → next=raw → |raw - next|=0 < eps.
                inner.scrub_last_ms.set_value(Some(1000.0));
                // Start non-converged so the false→true edge fires the callback.
                inner.scrub_converged.set_value(false);
                drop(inner);
            }

            // scroll_pos=50 → raw=0.5 (matches scrub_current) → converges on this call.
            trigger.engine_update(50.0, 0.0, 1000.0);

            assert!(fired.get(), "on_scrub_complete should fire on convergence");
        }

        #[test]
        fn on_scrub_complete_does_not_fire_on_initial_frame() {
            // scrub_converged starts true → no false→true transition → no fire,
            // even if the scrub value lands exactly on target on the first frame.
            let fired = Rc::new(Cell::new(false));
            let fired_clone = fired.clone();
            let config = ScrollTriggerConfig::default()
                .scrub(Scrub::Number(0.3))
                .on_scrub_complete(move |_| fired_clone.set(true));
            let trigger = ScrollTrigger::host_test_trigger(config, 0.5, true, 1);
            {
                let inner = trigger.inner.write_value();
                inner.start_pixels.set_value(0.0);
                inner.end_pixels.set_value(100.0);
                inner.prev_active.set_value(true);
                inner.prev_progress.set_value(0.5);
                inner.scrub_current.set_value(0.5);
                inner.scrub_target.set_value(0.5);
                inner.scrub_last_ms.set_value(Some(1000.0));
                // Already converged → no edge.
                inner.scrub_converged.set_value(true);
                drop(inner);
            }

            trigger.engine_update(50.0, 0.0, 1000.0);

            assert!(
                !fired.get(),
                "on_scrub_complete should NOT fire when already converged"
            );
        }

        #[test]
        fn on_refresh_fires_on_refresh_call() {
            let fired = Rc::new(Cell::new(false));
            let fired_clone = fired.clone();
            let config =
                ScrollTriggerConfig::default().on_refresh(move |_| fired_clone.set(true));
            let trigger = ScrollTrigger::host_test_trigger(config, 0.0, false, 0);

            // host has no target → start/end stay 0, but on_refresh still dispatches.
            trigger.refresh();

            assert!(fired.get(), "on_refresh should fire on refresh()");
        }
    }

    mod refresh {
        use super::*;

        #[test]
        fn refresh_with_no_target_sets_zero_pixels() {
            let trigger = ScrollTrigger::host_test_trigger(
                ScrollTriggerConfig::default(),
                0.0,
                false,
                0,
            );

            trigger.refresh();

            // host_test_trigger has no target → Rect is zero → start/end both 0.
            assert_eq!(trigger.start(), 0.0);
            assert_eq!(trigger.end(), 0.0);
        }

        #[test]
        fn refresh_equality_guards_prevent_signal_churn() {
            // Two consecutive refresh() calls with the same geometry should not
            // panic and should leave the signals stable.
            let trigger = ScrollTrigger::host_test_trigger(
                ScrollTriggerConfig::default(),
                0.0,
                false,
                0,
            );

            trigger.refresh();
            let progress_after_first = trigger.progress().get_untracked();
            let active_after_first = trigger.is_active().get_untracked();

            trigger.refresh();
            let progress_after_second = trigger.progress().get_untracked();
            let active_after_second = trigger.is_active().get_untracked();

            assert_eq!(progress_after_first, progress_after_second);
            assert_eq!(active_after_first, active_after_second);
        }
    }

    mod attach {
        use super::*;

        #[test]
        fn attach_resolver_updates_target_source() {
            let trigger = ScrollTrigger::host_test_trigger(
                ScrollTriggerConfig::default(),
                0.0,
                false,
                0,
            );

            // No DOM on host; the resolver simply returns None. The point of the
            // test is that attach_resolver stores the closure so target_source
            // becomes Some(...).
            trigger.attach_resolver(|| None);

            let inner = trigger.inner.read_value();
            assert!(
                inner.target_source.get_value().is_some(),
                "attach_resolver should set target_source to Some"
            );
        }

        #[test]
        fn reset_scrub_clock_clears_last_ms() {
            let trigger = ScrollTrigger::host_test_trigger(
                ScrollTriggerConfig::default(),
                0.0,
                false,
                0,
            );
            {
                let inner = trigger.inner.write_value();
                inner.scrub_last_ms.set_value(Some(1234.0));
                assert_eq!(inner.scrub_last_ms.get_value(), Some(1234.0));
                drop(inner);
            }

            trigger.reset_scrub_clock();

            assert_eq!(
                trigger.inner.read_value().scrub_last_ms.get_value(),
                None,
                "reset_scrub_clock should clear scrub_last_ms"
            );
        }
    }

    mod signals {
        use super::*;

        #[test]
        fn get_velocity_reads_per_trigger_tracker() {
            // Pushing samples via engine_update should populate the velocity
            // tracker; get_velocity should return a finite value after at least
            // two samples.
            let trigger = ScrollTrigger::host_test_trigger(
                ScrollTriggerConfig::default().scrub(Scrub::Bool(true)),
                0.0,
                false,
                0,
            );
            {
                let inner = trigger.inner.write_value();
                inner.start_pixels.set_value(0.0);
                inner.end_pixels.set_value(1000.0);
                inner.prev_active.set_value(false);
                inner.prev_progress.set_value(0.0);
                drop(inner);
            }

            // Two engine_update calls with distinct positions/times.
            trigger.engine_update(0.0, 0.0, 1000.0);
            trigger.engine_update(100.0, 0.0, 1100.0);

            let v = trigger.get_velocity();
            // 100px over 100ms = 1000 px/s (window is 100ms by default).
            assert!(
                v.is_finite(),
                "get_velocity should return a finite value, got {}",
                v
            );
        }

        #[test]
        fn progress_signal_reflects_engine_update() {
            // After engine_update moves into the active range, the progress()
            // signal should read the clamped value.
            let trigger = ScrollTrigger::host_test_trigger(
                ScrollTriggerConfig::default().scrub(Scrub::Bool(true)),
                0.0,
                false,
                0,
            );
            {
                let inner = trigger.inner.write_value();
                inner.start_pixels.set_value(0.0);
                inner.end_pixels.set_value(100.0);
                inner.prev_active.set_value(false);
                inner.prev_progress.set_value(0.0);
                drop(inner);
            }

            trigger.engine_update(50.0, 0.0, 1000.0);

            let p = trigger.progress().get_untracked();
            assert!((p - 0.5).abs() < 1e-9, "progress signal should be 0.5, got {}", p);
        }

        #[test]
        fn start_end_return_cached_pixels() {
            let trigger = ScrollTrigger::host_test_trigger(
                ScrollTriggerConfig::default(),
                0.0,
                false,
                0,
            );
            {
                let inner = trigger.inner.write_value();
                inner.start_pixels.set_value(42.0);
                inner.end_pixels.set_value(117.0);
                drop(inner);
            }

            assert_eq!(trigger.start(), 42.0);
            assert_eq!(trigger.end(), 117.0);
        }
    }
}