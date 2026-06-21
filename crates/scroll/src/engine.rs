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
use web_sys::wasm_bindgen::{closure::Closure, JsCast};

#[cfg(target_arch = "wasm32")]
thread_local! {
    static SHARED_ENGINE: RefCell<Option<SharedScrollEngine>> = const { RefCell::new(None) };
    static ENGINE_OUT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static PENDING_REGISTERS: RefCell<Vec<ScrollTrigger>> = const { RefCell::new(Vec::new()) };
    /// Side-channel flag set by `schedule_smoothing_tick` when it is called
    /// while the engine is taken OUT of the `SHARED_ENGINE` slot (i.e. from
    /// inside `tick`). `schedule_tick` consults this to avoid double-scheduling
    /// a rAF for the same frame. Cleared in the rAF prologue alongside
    /// `raf_scheduled`.
    static SMOOTHING_RAF_PENDING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
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
    /// Handle for the 250ms `setInterval` safety-net that catches dropped
    /// scroll events (Chrome at high velocity) and programmatic `scrollTo`
    /// that doesn't fire a detectable event. Mirrors GSAP's `_syncInterval`.
    _sync_interval_handle: IntervalHandle,
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
        let orientation_handle = install_window_listener("orientationchange", || {
            refresh_100vh();
            schedule_resize();
        });

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

        Some(Self {
            _scroll_handle: scroll_handle,
            _resize_handle: resize_handle,
            #[cfg(all(target_arch = "wasm32", feature = "resize-observer"))]
            _resize_observer_handle: resize_observer_handle,
            _orientation_handle: orientation_handle,
            _sync_interval_handle: sync_interval_handle,
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
            _sync_interval_handle: IntervalHandle::default(),
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

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    fn tick(&mut self) {
        let now = now_ms();
        let scroller = Scroller::viewport();
        let scroll_pos = scroller.scroll_position();
        self.velocity_tracker.push(now, scroll_pos);
        let velocity = self.velocity_tracker.velocity();

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
        });
    });
}

#[cfg(target_arch = "wasm32")]
fn schedule_resize() {
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