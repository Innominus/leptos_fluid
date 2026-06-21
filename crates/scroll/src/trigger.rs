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

        inner.direction.set(direction_sign);
        inner.velocity.set(velocity);
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

        let exposed_progress = step_scrub(&inner, clamped, now_ms);
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

        if inner.config.once && active_changed && !active && direction_sign == 1 {
            drop(inner);
            self.kill();
            return false;
        }

        // Self-reschedule signal: only Scrub::Number smoothing needs continuous
        // rAF frames; converged triggers stop the loop.
        matches!(inner.config.scrub, Scrub::Number(_))
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

fn step_scrub(inner: &ScrollTriggerInner, raw: f64, now_ms: f64) -> f64 {
    match inner.config.scrub {
        Scrub::Bool(false) => raw,
        Scrub::Bool(true) => {
            inner.scrub_current.set_value(raw);
            raw
        }
        Scrub::Number(t) => {
            inner.scrub_target.set_value(raw);
            let current = inner.scrub_current.get_value();
            let last = inner.scrub_last_ms.get_value();
            let dt = match last {
                Some(prev) => ((now_ms - prev) / 1000.0).max(0.0),
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
                raw
            } else {
                inner.scrub_current.set_value(next);
                inner.scrub_last_ms.set_value(Some(now_ms));
                next
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::{ScrollOffset, ScrollPosition};

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

    #[test]
    fn phase_transition_maps_correctly() {
        assert_eq!(phase_transition(false, true, 1), TogglePhase::OnEnter);
        assert_eq!(phase_transition(true, false, 1), TogglePhase::OnLeave);
        assert_eq!(phase_transition(false, true, -1), TogglePhase::OnEnterBack);
        assert_eq!(phase_transition(true, false, -1), TogglePhase::OnLeaveBack);
    }

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

    #[test]
    fn scrub_bool_false_exposes_raw() {
        use crate::config::ScrollTriggerConfig;
        let cfg = ScrollTriggerConfig::default().scrub(Scrub::Bool(false));
        let inner = ScrollTriggerInner {
            config: cfg,
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
            registration_id: StoredValue::new_local(None),
            enabled: StoredValue::new_local(true),
            killed: StoredValue::new_local(false),
            velocity_tracker: StoredValue::new_local(VelocityTracker::new()),
            prev_active: StoredValue::new_local(false),
            prev_progress: StoredValue::new_local(0.0),
        };
        assert_eq!(step_scrub(&inner, 0.7, 1000.0), 0.7);
    }

    #[test]
    fn scrub_number_eases_toward_target() {
        use crate::config::ScrollTriggerConfig;
        let cfg = ScrollTriggerConfig::default().scrub(Scrub::Number(0.3));
        let inner = ScrollTriggerInner {
            config: cfg,
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
            scrub_last_ms: StoredValue::new_local(Some(1000.0)),
            registration_id: StoredValue::new_local(None),
            enabled: StoredValue::new_local(true),
            killed: StoredValue::new_local(false),
            velocity_tracker: StoredValue::new_local(VelocityTracker::new()),
            prev_active: StoredValue::new_local(false),
            prev_progress: StoredValue::new_local(0.0),
        };
        let first = step_scrub(&inner, 1.0, 1050.0);
        assert!(first > 0.0 && first < 1.0);
        let second = step_scrub(&inner, 1.0, 1100.0);
        assert!(second > first && second < 1.0);
    }
}