//! Shared scroll engine — a thread-local singleton that batches scroll/resize
//! updates via `requestAnimationFrame`.
//!
//! One scroll listener + one rAF drives all registered triggers on the same
//! scroller (viewport in MVP). Mirrors the `SHARED_RESIZE_OBSERVER` pattern in
//! `crates/web/src/lib.rs:148`.

#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;

use js_sys::Date;

#[cfg(target_arch = "wasm32")]
use leptos::prelude::request_animation_frame;

use crate::callbacks::VelocityTracker;
use crate::scroller::{Scroller, ScrollListenerHandle};
#[cfg(target_arch = "wasm32")]
use crate::scroller::{inner_height, inner_width, install_window_listener, refresh_100vh};
use crate::trigger::ScrollTrigger;

#[cfg(all(target_arch = "wasm32", feature = "resize-observer"))]
use leptos_fluid_web::{ResizeObserverHandle, observe_resize};

#[cfg(target_arch = "wasm32")]
use web_sys::wasm_bindgen::{closure::Closure, JsCast, JsValue};

#[cfg(target_arch = "wasm32")]
thread_local! {
    static SHARED_ENGINE: RefCell<Option<SharedScrollEngine>> = const { RefCell::new(None) };
    static ENGINE_OUT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static PENDING_REGISTERS: RefCell<Vec<ScrollTrigger>> = const { RefCell::new(Vec::new()) };
    /// Side-channel flag set by `schedule_resize` when it is called while the
    /// engine is taken OUT of the `SHARED_ENGINE` slot (i.e. from inside
    /// `tick`/`refresh_all`). The rAF closures drain this after restoring the
    /// engine so the resize isn't silently dropped. GSAP-parity: a resize
    /// during a tick must still be honored.
    static PENDING_RESIZE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// Side-channel flag set by `schedule_smoothing_tick` when it is called
    /// while the engine is taken OUT of the `SHARED_ENGINE` slot (i.e. from
    /// inside `tick`). `schedule_tick` consults this to avoid double-scheduling
    /// a rAF for the same frame. Cleared in the rAF prologue alongside
    /// `raf_scheduled`.
    static SMOOTHING_RAF_PENDING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    // GSAP-parity: engine-global `prefers-reduced-motion` posture. `Ignore` by
    // default; callers opt in via `set_reduced_motion(Respect)`. The cached
    // `ACTIVE` bool mirrors the current MQ match and is updated by the
    // `MediaQueryList` change listener (and re-checked on visibility regain).
    static REDUCED_MOTION_MODE: std::cell::Cell<crate::config::ReducedMotion> =
        const { std::cell::Cell::new(crate::config::ReducedMotion::Ignore) };
    static REDUCED_MOTION_ACTIVE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
struct RegisteredTrigger {
    id: u32,
    trigger: ScrollTrigger,
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
struct SharedScrollEngine {
    _scroll_handle: ScrollListenerHandle,
    _resize_handle: ScrollListenerHandle,
    #[cfg(all(target_arch = "wasm32", feature = "resize-observer"))]
    _resize_observer_handle: Option<ResizeObserverHandle>,
    /// Handle for the `orientationchange` listener (re-measures `100vh` +
    /// refreshes on rotation).
    _orientation_handle: ScrollListenerHandle,
    /// Handle for the `visibilitychange` listener (resets scrub clocks and
    /// re-measures on tab regain so a backgrounded rAF loop doesn't snap).
    _visibility_handle: ScrollListenerHandle,
    /// Handle for the 250ms `setInterval` safety-net that catches dropped
    /// scroll events (Chrome at high velocity) and programmatic `scrollTo`
    /// that doesn't fire a detectable event. Mirrors GSAP's `_syncInterval`.
    _sync_interval_handle: IntervalHandle,
    /// Handle for the `document.fonts.ready` callback: fires a resize refresh
    /// once web fonts load so trigger geometry re-measures after layout shift.
    _fonts_ready_handle: FontFaceReadyHandle,
    /// Handle for the `matchMedia("(prefers-reduced-motion: reduce)")` change
    /// listener. Lazily installed by `set_reduced_motion(Respect)`; keeps the
    /// change `Closure` alive so `REDUCED_MOTION_ACTIVE` tracks OS toggles.
    _reduced_motion_mq_handle: MediaQueryListHandle,
    triggers: Vec<RegisteredTrigger>,
    next_id: u32,
    scroll_pending: bool,
    raf_scheduled: bool,
    last_resize_ms: f64,
    velocity_tracker: VelocityTracker,
    /// Last scroll position observed by the sync-interval poll. If the scroll
    /// position changes without a `scroll` event firing, the interval queues
    /// a rAF tick to catch up.
    last_polled_scroll: f64,
    /// Baseline `innerWidth` at engine init, used by the mobile address-bar
    /// resize suppression. If a resize event has the same width and a small
    /// height delta (< 25% of innerHeight), it's likely the address bar
    /// toggling — suppressed unless the `100vh` sentinel says otherwise.
    base_width: f64,
    /// Baseline `innerHeight` at engine init, for the address-bar suppression.
    base_height: f64,
}

/// Opaque handle to a `setInterval` that drops the closure on `Drop`.
/// Keeps the interval alive for the lifetime of the engine. On non-wasm
/// targets this is a no-op zero-sized placeholder.
struct IntervalHandle {
    #[cfg(target_arch = "wasm32")]
    closure: Option<Rc<Closure<dyn FnMut()>>>,
    #[cfg(target_arch = "wasm32")]
    id: i32,
}

/// Opaque handle to the `document.fonts.ready` promise callback. Keeps the
/// `Closure` alive so JS can call it when web fonts finish loading. On non-wasm
/// targets this is a no-op zero-sized placeholder.
struct FontFaceReadyHandle {
    #[cfg(target_arch = "wasm32")]
    _closure: Option<Closure<dyn FnMut(JsValue)>>,
}

/// Opaque handle to a `matchMedia` listener for `prefers-reduced-motion`.
/// Keeps the change `Closure` alive for the engine's lifetime so the cached
/// `REDUCED_MOTION_ACTIVE` bool tracks OS/browser reduced-motion toggles. On
/// non-wasm targets this is a no-op zero-sized placeholder.
struct MediaQueryListHandle {
    #[cfg(target_arch = "wasm32")]
    _mql: Option<web_sys::MediaQueryList>,
    #[cfg(target_arch = "wasm32")]
    _closure: Option<Closure<dyn FnMut(JsValue)>>,
}

impl Default for IntervalHandle {
    fn default() -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            closure: None,
            #[cfg(target_arch = "wasm32")]
            id: 0,
        }
    }
}

impl Default for FontFaceReadyHandle {
    fn default() -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            _closure: None,
        }
    }
}

impl Default for MediaQueryListHandle {
    fn default() -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            _mql: None,
            #[cfg(target_arch = "wasm32")]
            _closure: None,
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl Drop for IntervalHandle {
    fn drop(&mut self) {
        if let Some(window) = web_sys::window() {
            let _ = window.clear_interval_with_handle(self.id);
        }
        // Closure dropped here, freeing the JS callback.
        self.closure.take();
    }
}

#[cfg(target_arch = "wasm32")]
impl Drop for MediaQueryListHandle {
    fn drop(&mut self) {
        // Detach the JS-side `change` listener before the `Closure` drops so
        // there's no window where both old and new listeners fire on a
        // re-install. Mirrors `IntervalHandle::Drop`.
        if let (Some(mql), Some(closure)) = (self._mql.take(), self._closure.take()) {
            let _ = mql.remove_event_listener_with_callback(
                "change",
                closure.as_ref().unchecked_ref(),
            );
        }
    }
}

impl SharedScrollEngine {
    #[cfg(target_arch = "wasm32")]
    fn new() -> Option<Self> {
        let scroller = Scroller::viewport();
        let scroll_handle = scroller.on_scroll(|| schedule_tick());
        let resize_handle = scroller.on_resize(|| schedule_resize());

        #[cfg(feature = "resize-observer")]
        let resize_observer_handle = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.document_element())
            .map(|doc_el| observe_resize(&doc_el, || schedule_resize()));

        // Measure the stable 100vh sentinel now so viewport_size() is correct
        // from the first tick.
        refresh_100vh();

        // orientationchange: re-measure 100vh (CSS 100vh can change on
        // rotation) and refresh all triggers.
        let orientation_handle = install_window_listener("orientationchange", Box::new(|| {
            refresh_100vh();
            schedule_resize();
        }));

        // visibilitychange: GSAP-parity — when the tab becomes visible again,
        // reset scrub clocks (so the first tick back doesn't use a huge stale
        // dt and snap) and schedule a resize if the viewport dimensions changed
        // while hidden (e.g. devtools docked/un-docked).
        let visibility_handle = install_window_listener("visibilitychange", Box::new(|| {
            // If the engine is OUT (inside a tick), skip — the rAF closure will
            // handle any pending work.
            if ENGINE_OUT.with(|out| out.get()) {
                return;
            }
            let visible = web_sys::window()
                .and_then(|w| w.document())
                .map(|d| !d.hidden())
                .unwrap_or(false);
            if visible {
                let width = inner_width();
                let height = inner_height();
                let dims_changed = SHARED_ENGINE.with(|slot| {
                    slot.borrow()
                        .as_ref()
                        .map(|engine| {
                            (width - engine.base_width).abs() > 1.0
                                || (height - engine.base_height).abs() > 1.0
                        })
                        .unwrap_or(false)
                });
                reset_scrub_clocks();
                // GSAP-parity: re-check the reduced-motion MQ on visibility
                // regain in case the OS-level setting changed while the tab
                // was hidden (the change listener doesn't fire for a hidden
                // tab). No-op when mode is `Ignore`.
                refresh_reduced_motion_active();
                if dims_changed {
                    schedule_resize();
                }
            }
        }));

        // 250ms setInterval safety net: if the scroll position changed without
        // a `scroll` event firing (Chrome drops events at high velocity, or
        // programmatic `scrollTo` without `behavior: "smooth"`), queue a rAF
        // tick to catch up. Mirrors GSAP's `_syncInterval` (which uses 250ms
        // and a 34ms threshold). The interval is lightweight — one
        // `scroll_position()` read + comparison per 250ms.
        let sync_closure = Rc::new(Closure::wrap(Box::new(sync_interval_tick) as Box<dyn FnMut()>));
        let sync_id = web_sys::window()
            .map(|w| {
                w.set_interval_with_callback_and_timeout_and_arguments_0(
                    (&*sync_closure).as_ref().unchecked_ref(),
                    250,
                )
                .unwrap_or(0)
            })
            .unwrap_or(0);
        let sync_interval_handle = IntervalHandle {
            closure: Some(sync_closure),
            id: sync_id,
        };

        let base_w = inner_width();
        let base_h = inner_height();

        // fonts.ready: GSAP-parity — re-measure triggers once web fonts load,
        // since font loading can shift element geometry (line heights, widths).
        let fonts_ready_handle = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.fonts().ready().ok())
            .and_then(|promise| {
                // Keep the closure alive for the promise's lifetime; it fires
                // once when fonts settle, scheduling a resize refresh.
                let closure = Closure::<dyn FnMut(JsValue)>::new(|_v| {
                    schedule_resize();
                });
                let _ = promise.then(&closure);
                Some(FontFaceReadyHandle { _closure: Some(closure) })
            })
            .unwrap_or_default();

        Some(Self {
            _scroll_handle: scroll_handle,
            _resize_handle: resize_handle,
            #[cfg(all(target_arch = "wasm32", feature = "resize-observer"))]
            _resize_observer_handle: resize_observer_handle,
            _orientation_handle: orientation_handle,
            _visibility_handle: visibility_handle,
            _sync_interval_handle: sync_interval_handle,
            _fonts_ready_handle: fonts_ready_handle,
            // Reduced-motion MQ listener is NOT installed by default (mode is
            // `Ignore`). `set_reduced_motion(Respect)` installs it lazily and
            // stores the handle here.
            _reduced_motion_mq_handle: MediaQueryListHandle::default(),
            triggers: Vec::new(),
            next_id: 1,
            scroll_pending: false,
            raf_scheduled: false,
            last_resize_ms: 0.0,
            velocity_tracker: VelocityTracker::new(),
            last_polled_scroll: 0.0,
            base_width: base_w,
            base_height: base_h,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[allow(dead_code)]
    fn new() -> Option<Self> {
        Some(Self {
            _scroll_handle: ScrollListenerHandle::default(),
            _resize_handle: ScrollListenerHandle::default(),
            _orientation_handle: ScrollListenerHandle::default(),
            _visibility_handle: ScrollListenerHandle::default(),
            _sync_interval_handle: IntervalHandle::default(),
            _fonts_ready_handle: FontFaceReadyHandle::default(),
            _reduced_motion_mq_handle: MediaQueryListHandle::default(),
            triggers: Vec::new(),
            next_id: 1,
            scroll_pending: false,
            raf_scheduled: false,
            last_resize_ms: 0.0,
            velocity_tracker: VelocityTracker::new(),
            last_polled_scroll: 0.0,
            base_width: 0.0,
            base_height: 0.0,
        })
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    fn register(&mut self, trigger: ScrollTrigger) -> u32 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        self.triggers.push(RegisteredTrigger { id, trigger });
        id
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    fn unregister(&mut self, id: u32) {
        self.triggers.retain(|registered| registered.id != id);
    }

    /// Resets every registered trigger's scrub clock to `None` so the next
    /// `step_scrub` call uses `dt = 0` instead of a stale timestamp. Called on
    /// visibility regain (tab hidden → visible) to avoid a huge first dt.
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    fn reset_scrub_clocks(&mut self) {
        for registered in &self.triggers {
            registered.trigger.reset_scrub_clock();
        }
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    fn tick(&mut self) {
        let now = now_ms();
        let scroller = Scroller::viewport();
        let scroll_pos = scroller.scroll_position();
        self.velocity_tracker.push(now, scroll_pos);
        // GSAP-parity: use velocity_now(now) so a paused rAF loop (tab hidden)
        // doesn't report a stale velocity on the first tick back.
        let velocity = self.velocity_tracker.velocity_now(now);

        let triggers = std::mem::take(&mut self.triggers);
        let mut needs_more = false;
        for registered in &triggers {
            if registered.trigger.engine_update(scroll_pos, velocity, now) {
                needs_more = true;
            }
        }
        self.triggers = triggers
            .into_iter()
            .filter(|registered| !registered.trigger.is_killed())
            .collect();

        // Self-reschedule the rAF loop if any Scrub::Number trigger hasn't
        // converged. This keeps the smoothing advancing every frame (not just
        // on scroll events) and stops once all triggers reach their targets,
        // so there's no CPU usage at rest. The next scroll event restarts the
        // loop via `schedule_tick`. We use `schedule_smoothing_tick` (not
        // `schedule_tick`) so we don't spuriously mark `scroll_pending = true`
        // when there's no new scroll — just continued smoothing.
        if needs_more {
            #[cfg(target_arch = "wasm32")]
            schedule_smoothing_tick();
        }
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    fn refresh_all(&mut self) {
        // Re-measure the 100vh sentinel at the start of every refresh — CSS
        // `100vh` can change on orientation rotation, and the sentinel should
        // reflect the current stable viewport height.
        #[cfg(target_arch = "wasm32")]
        refresh_100vh();

        let triggers = std::mem::take(&mut self.triggers);
        for registered in &triggers {
            registered.trigger.refresh();
        }
        self.triggers = triggers
            .into_iter()
            .filter(|registered| !registered.trigger.is_killed())
            .collect();

        // Update the baseline dimensions after a refresh so the mobile
        // address-bar suppression has an accurate reference for the *next*
        // resize event.
        #[cfg(target_arch = "wasm32")]
        {
            self.base_width = inner_width();
            self.base_height = inner_height();
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn now_ms() -> f64 {
    Date::now()
}

#[cfg(target_arch = "wasm32")]
fn schedule_tick() {
    // A smoothing rAF may have been scheduled while the engine was OUT of the
    // slot (from inside `tick`). If so, that rAF will fire and process any
    // pending scroll — we only need to mark `scroll_pending` so the smoothing
    // tick knows to treat the frame as scroll-driven. We must NOT schedule a
    // second rAF or set `raf_scheduled` (the smoothing path already owns the
    // flag via `SMOOTHING_RAF_PENDING`).
    if SMOOTHING_RAF_PENDING.with(|p| p.get()) {
        SHARED_ENGINE.with(|slot| {
            if let Some(engine) = slot.borrow_mut().as_mut() {
                engine.scroll_pending = true;
            }
        });
        return;
    }
    SHARED_ENGINE.with(|slot| {
        let mut borrow = slot.borrow_mut();
        let Some(engine) = borrow.as_mut() else {
            return;
        };
        engine.scroll_pending = true;
        if engine.raf_scheduled {
            return;
        }
        engine.raf_scheduled = true;
    });
    request_animation_frame(move || {
        SHARED_ENGINE.with(|slot| {
            // Take the engine OUT of the slot so callbacks invoked during
            // `tick` can re-enter `register`/`unregister` (which hit the slot)
            // without a `RefCell` double-borrow panic. `register` calls during
            // the loop are queued in `PENDING_REGISTERS`; `unregister` is a
            // no-op (the `killed` flag + tick's `!is_killed()` filter drops it).
            let mut engine_opt = slot.borrow_mut().take();
            ENGINE_OUT.with(|out| out.set(true));
            if let Some(engine) = engine_opt.as_mut() {
                engine.raf_scheduled = false;
                SMOOTHING_RAF_PENDING.with(|p| p.set(false));
                engine.scroll_pending = false;
                engine.tick();
                PENDING_REGISTERS.with(|pending| {
                    for trigger in pending.borrow_mut().drain(..) {
                        engine.register(trigger);
                    }
                });
            }
            ENGINE_OUT.with(|out| out.set(false));
            *slot.borrow_mut() = engine_opt;
            // GSAP-parity: a resize queued while the engine was OUT must not be
            // dropped. Re-enter schedule_resize now that ENGINE_OUT is false.
            if PENDING_RESIZE.with(|p| p.replace(false)) {
                schedule_resize();
            }
        });
    });
}

#[cfg(target_arch = "wasm32")]
fn schedule_smoothing_tick() {
    // Self-reschedule variant of `schedule_tick` used when `tick` detects that
    // one or more `Scrub::Number` triggers haven't converged to their targets.
    // It schedules the next rAF frame WITHOUT marking `scroll_pending = true`
    // (there's no new scroll event — we're continuing smoothing across frames).
    //
    // The only caller is `tick`, which runs inside the rAF callback where the
    // engine has been taken OUT of the `SHARED_ENGINE` slot (see `schedule_tick`).
    // The `SHARED_ENGINE.with` dedup block below therefore finds `None` and
    // returns early. The `raf_scheduled` flag was already reset to `false` in
    // the rAF prologue (engine.rs:179/220) BEFORE `tick` ran, so we set it back
    // to `true` here unconditionally to prevent `schedule_tick` from
    // double-scheduling a second rAF for the same frame (which would cause
    // `tick` to run twice, zeroing velocity via the `dt == 0.0` guard and
    // doubling `set_immediate` churn on subscribers).
    SHARED_ENGINE.with(|slot| {
        if let Some(engine) = slot.borrow_mut().as_mut() {
            if engine.raf_scheduled {
                return;
            }
            engine.raf_scheduled = true;
        }
    });
    // Engine is OUT of the slot during `tick` (the common caller). Set the
    // flag via a side channel so `schedule_tick`'s dedup sees it even when
    // the slot is `None`. `SMOOTHING_RAF_PENDING` is cleared in the rAF
    // prologue alongside `raf_scheduled`.
    SMOOTHING_RAF_PENDING.with(|p| p.set(true));
    request_animation_frame(move || {
        SHARED_ENGINE.with(|slot| {
            // Take the engine OUT of the slot so callbacks invoked during
            // `tick` can re-enter `register`/`unregister` (which hit the slot)
            // without a `RefCell` double-borrow panic. `register` calls during
            // the loop are queued in `PENDING_REGISTERS`; `unregister` is a
            // no-op (the `killed` flag + tick's `!is_killed()` filter drops it).
            let mut engine_opt = slot.borrow_mut().take();
            ENGINE_OUT.with(|out| out.set(true));
            if let Some(engine) = engine_opt.as_mut() {
                engine.raf_scheduled = false;
                SMOOTHING_RAF_PENDING.with(|p| p.set(false));
                engine.tick();
                PENDING_REGISTERS.with(|pending| {
                    for trigger in pending.borrow_mut().drain(..) {
                        engine.register(trigger);
                    }
                });
            }
            ENGINE_OUT.with(|out| out.set(false));
            *slot.borrow_mut() = engine_opt;
            // GSAP-parity: a resize queued while the engine was OUT must not be
            // dropped. Re-enter schedule_resize now that ENGINE_OUT is false.
            if PENDING_RESIZE.with(|p| p.replace(false)) {
                schedule_resize();
            }
        });
    });
}

#[cfg(target_arch = "wasm32")]
fn schedule_resize() {
    // If the engine is taken OUT of the slot (inside tick/refresh_all), the
    // `SHARED_ENGINE.with` borrow below would find `None` and silently drop
    // the resize. Queue it via a side-channel; the rAF closure restores the
    // engine and re-enters `schedule_resize` with ENGINE_OUT = false.
    if ENGINE_OUT.with(|out| out.get()) {
        PENDING_RESIZE.with(|p| p.set(true));
        return;
    }
    // Mobile address-bar suppression: on touch devices, the iOS Safari / Chrome
    // Android address bar collapses/expands during scroll, firing `resize`
    // events with a small height delta (typically 10-15% of innerHeight) and
    // no width change. These are not real resizes — the layout hasn't changed
    // in a way that affects trigger geometry (the 100vh sentinel is stable).
    // Suppress them to avoid spurious `refresh_all` + progress churn.
    //
    // Heuristic (mirrors GSAP's `_onResize`): if the width is unchanged AND
    // the height delta is < 25% of innerHeight, skip the refresh. The 25%
    // threshold is generous enough to catch address-bar toggles but not a
    // real orientation change (which swaps width/height → width changes).
    //
    // We apply this unconditionally (not gated on a touch-only flag) because
    // detecting touch reliably in wasm is fragile, and the suppression is
    // safe for desktop too (desktop resizes almost always change width, and
    // a height-only resize on desktop is rare and usually small).
    SHARED_ENGINE.with(|slot| {
        let mut borrow = slot.borrow_mut();
        let Some(engine) = borrow.as_mut() else {
            return;
        };
        let now = now_ms();
        // Debounce: coalesce rapid resize bursts into a single refresh after
        // 200ms of quiet. This is a trailing-edge debounce — each event within
        // 200ms of the last resets the timer.
        if engine.last_resize_ms != 0.0 && now - engine.last_resize_ms < 200.0 {
            request_animation_frame(move || {
                schedule_resize();
            });
            return;
        }
        // Mobile address-bar suppression check.
        let current_width = inner_width();
        let current_height = inner_height();
        let width_changed = (current_width - engine.base_width).abs() > 1.0;
        let height_delta = (current_height - engine.base_height).abs();
        let height_threshold = engine.base_height * 0.25;
        let is_address_bar_noise = !width_changed && height_delta < height_threshold;
        if is_address_bar_noise {
            // Skip this resize — it's just the address bar. Update the baseline
            // so subsequent toggles are measured against the latest height.
            engine.base_height = current_height;
            return;
        }
        engine.last_resize_ms = now;
    });
    request_animation_frame(move || {
        SHARED_ENGINE.with(|slot| {
            let mut engine_opt = slot.borrow_mut().take();
            ENGINE_OUT.with(|out| out.set(true));
            if let Some(engine) = engine_opt.as_mut() {
                engine.refresh_all();
                PENDING_REGISTERS.with(|pending| {
                    for trigger in pending.borrow_mut().drain(..) {
                        engine.register(trigger);
                    }
                });
            }
            ENGINE_OUT.with(|out| out.set(false));
            *slot.borrow_mut() = engine_opt;
            // Drain any resize queued during refresh_all (e.g. from on_refresh).
            if PENDING_RESIZE.with(|p| p.replace(false)) {
                schedule_resize();
            }
        });
    });
}

/// 250ms safety-net poll. Catches:
/// - Chrome dropping `scroll` events at high scroll velocity (the browser
///   throttles event firing but `scrollY` keeps changing).
/// - Programmatic `window.scrollTo()` that doesn't fire a detectable `scroll`
///   event (e.g. instant jump without `behavior: "smooth"`).
///
/// If the scroll position changed since the last poll without a `scroll` event
/// having scheduled a tick, queue one. This mirrors GSAP's `_syncInterval`
/// (250ms, 34ms threshold).
#[cfg(target_arch = "wasm32")]
fn sync_interval_tick() {
    SHARED_ENGINE.with(|slot| {
        let mut borrow = slot.borrow_mut();
        let Some(engine) = borrow.as_mut() else {
            return;
        };
        let current = Scroller::viewport().scroll_position();
        if (current - engine.last_polled_scroll).abs() > 0.5 {
            // Scroll moved without a scroll event — catch up.
            engine.last_polled_scroll = current;
            drop(borrow);
            schedule_tick();
        } else {
            engine.last_polled_scroll = current;
        }
    });
}

pub(crate) fn register(trigger: ScrollTrigger) -> u32 {
    #[cfg(target_arch = "wasm32")]
    {
        if ENGINE_OUT.with(|out| out.get()) {
            // Engine is taken out of the slot for tick/refresh_all; queue
            // for merge after the loop completes.
            PENDING_REGISTERS.with(|pending| {
                pending.borrow_mut().push(trigger);
            });
            return 0;
        }
        SHARED_ENGINE.with(|slot| {
            if slot.borrow().is_none() {
                *slot.borrow_mut() = SharedScrollEngine::new();
                // F1: a pre-engine-init `set_reduced_motion(Respect)` installs
                // the MQ listener but the handle is discarded (slot was None).
                // Adopt the listener now that the engine exists so the change
                // `Closure` lives for the engine's lifetime.
                if REDUCED_MOTION_MODE.with(|m| m.get())
                    == crate::config::ReducedMotion::Respect
                {
                    install_reduced_motion_listener();
                }
            }
            slot.borrow_mut()
                .as_mut()
                .map(|engine| engine.register(trigger))
                .unwrap_or(0)
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = trigger;
        0
    }
}

pub(crate) fn unregister(id: u32) {
    #[cfg(target_arch = "wasm32")]
    {
        SHARED_ENGINE.with(|slot| {
            if let Some(engine) = slot.borrow_mut().as_mut() {
                engine.unregister(id);
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = id;
    }
}

/// Resets all registered triggers' scrub clocks. Called by the
/// `visibilitychange` listener on tab regain (see `SharedScrollEngine::new`).
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub(crate) fn reset_scrub_clocks() {
    #[cfg(target_arch = "wasm32")]
    {
        // If the engine is OUT (inside a tick), skip — the rAF closure will
        // handle it. (The visibilitychange listener already guards this, but
        // be defensive in case this is called from elsewhere.)
        if ENGINE_OUT.with(|out| out.get()) {
            return;
        }
        SHARED_ENGINE.with(|slot| {
            if let Some(engine) = slot.borrow_mut().as_mut() {
                engine.reset_scrub_clocks();
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // No-op on non-wasm: there are no registered triggers in this path.
    }
}

/// Sets the engine-global `prefers-reduced-motion` posture. When set to
/// `Respect`, the engine installs a `matchMedia` listener and snaps
/// `Scrub::Number` smoothing to raw progress while
/// `(prefers-reduced-motion: reduce)` matches. When set to `Ignore` (the
/// default), the cached active flag is cleared and smoothing runs
/// unconditionally. The MQ listener (if previously installed) is left in place
/// — it only updates `REDUCED_MOTION_ACTIVE`, which is only consulted when the
/// mode is `Respect`, so it's harmless when the mode is `Ignore`.
#[cfg(target_arch = "wasm32")]
pub fn set_reduced_motion(mode: crate::config::ReducedMotion) {
    REDUCED_MOTION_MODE.with(|m| m.set(mode));
    match mode {
        crate::config::ReducedMotion::Respect => {
            install_reduced_motion_listener();
        }
        crate::config::ReducedMotion::Ignore => {
            REDUCED_MOTION_ACTIVE.with(|a| a.set(false));
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn set_reduced_motion(_mode: crate::config::ReducedMotion) {}

/// Returns `true` when the engine should snap `Scrub::Number` to raw progress:
/// the mode is `Respect` AND the `(prefers-reduced-motion: reduce)` media query
/// currently matches. Called from `trigger::engine_update` to gate `step_scrub`.
#[cfg(target_arch = "wasm32")]
pub(crate) fn reduced_motion_snaps_scrub() -> bool {
    REDUCED_MOTION_MODE.with(|m| m.get()) == crate::config::ReducedMotion::Respect
        && REDUCED_MOTION_ACTIVE.with(|a| a.get())
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn reduced_motion_snaps_scrub() -> bool {
    false
}

/// Re-reads `matchMedia("(prefers-reduced-motion: reduce)").matches` and
/// updates `REDUCED_MOTION_ACTIVE`. Called on visibility regain (the change
/// listener doesn't fire while the tab is hidden) and by
/// `install_reduced_motion_listener` for the initial read. No-op when there's
/// no `window` (e.g. SSR / non-browser wasm).
#[cfg(target_arch = "wasm32")]
fn refresh_reduced_motion_active() {
    let active = web_sys::window()
        .and_then(|w| w.match_media("(prefers-reduced-motion: reduce)").ok().flatten())
        .map(|mql| mql.matches())
        .unwrap_or(false);
    REDUCED_MOTION_ACTIVE.with(|a| a.set(active));
}

/// Installs (or refreshes) the `MediaQueryList` change listener for
/// `prefers-reduced-motion: reduce` and stores its handle on the engine so the
/// `Closure` lives as long as the engine. Reads the current `matches` value
/// once on install. Subsequent OS/browser toggles update
/// `REDUCED_MOTION_ACTIVE` via the change callback. Idempotent: re-installing
/// replaces the prior handle (the old `Closure` drops, detaching the prior
/// listener).
#[cfg(target_arch = "wasm32")]
fn install_reduced_motion_listener() {
    // Engine is taken out of the slot for tick/refresh_all; the listener install
    // would silently no-op (slot is None). Defer by returning — the F1 fix in
    // `register()` will adopt the listener on the next register, or the user can
    // re-call `set_reduced_motion` outside a callback.
    if ENGINE_OUT.with(|out| out.get()) {
        return;
    }
    // Initial read so the active flag is correct before any change event fires.
    refresh_reduced_motion_active();
    let Some(window) = web_sys::window() else {
        return;
    };
    let mql = match window.match_media("(prefers-reduced-motion: reduce)") {
        Ok(Some(mql)) => mql,
        _ => return,
    };
    // GSAP-parity: mirror the MQ's `matches` into the cached active bool on
    // every change so `reduced_motion_snaps_scrub()` reflects the current OS
    // posture without re-querying `matchMedia` per tick.
    let closure = Closure::wrap(Box::new(|event: JsValue| {
        let active = event
            .dyn_ref::<web_sys::MediaQueryListEvent>()
            .map(|e| e.matches())
            .unwrap_or(false);
        REDUCED_MOTION_ACTIVE.with(|a| a.set(active));
    }) as Box<dyn FnMut(JsValue)>);
    let _ = mql.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref());
    let handle = MediaQueryListHandle {
        _mql: Some(mql),
        _closure: Some(closure),
    };
    SHARED_ENGINE.with(|slot| {
        if let Some(engine) = slot.borrow_mut().as_mut() {
            engine._reduced_motion_mq_handle = handle;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    // GSAP-parity: documents that `prefers-reduced-motion` accommodation is a
    // wasm-only behavior. On host targets `reduced_motion_snaps_scrub()` always
    // returns `false`, so the smoothing-skip path never fires in host tests —
    // the real gating is exercised in browser wasm.
    #[test]
    fn reduced_motion_snaps_scrub_returns_false_on_host() {
        // On non-wasm targets, reduced_motion_snaps_scrub() always returns false
        // regardless of the mode set. This documents the contract.
        set_reduced_motion(crate::config::ReducedMotion::Respect);
        assert!(!reduced_motion_snaps_scrub());
        set_reduced_motion(crate::config::ReducedMotion::Ignore);
        assert!(!reduced_motion_snaps_scrub());
    }

    #[test]
    fn reduced_motion_set_is_callable_on_host() {
        // set_reduced_motion is a no-op on host (the thread-local only exists
        // in wasm), but it must remain callable so library code that calls it
        // unconditionally compiles + runs in host tests.
        set_reduced_motion(crate::config::ReducedMotion::Respect);
        set_reduced_motion(crate::config::ReducedMotion::Ignore);
    }
}