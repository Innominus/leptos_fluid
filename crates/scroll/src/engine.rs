//! Shared scroll engine — a thread-local singleton that batches scroll/resize
//! updates via `requestAnimationFrame`.
//!
//! One scroll listener + one rAF drives all registered triggers on the same
//! scroller (viewport in MVP). Mirrors the `SHARED_RESIZE_OBSERVER` pattern in
//! `crates/web/src/lib.rs:148`.

#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;

use js_sys::Date;

#[cfg(target_arch = "wasm32")]
use leptos::prelude::request_animation_frame;

use crate::callbacks::VelocityTracker;
use crate::scroller::{ScrollListenerHandle, Scroller};
use crate::trigger::ScrollTrigger;

#[cfg(target_arch = "wasm32")]
use leptos_fluid_web::{ResizeObserverHandle, observe_resize};

#[cfg(target_arch = "wasm32")]
thread_local! {
    static SHARED_ENGINE: RefCell<Option<SharedScrollEngine>> = const { RefCell::new(None) };
}

#[allow(dead_code)]
struct RegisteredTrigger {
    id: u32,
    trigger: ScrollTrigger,
}

#[allow(dead_code)]
struct SharedScrollEngine {
    _scroll_handle: ScrollListenerHandle,
    _resize_handle: ScrollListenerHandle,
    #[cfg(target_arch = "wasm32")]
    _resize_observer_handle: Option<ResizeObserverHandle>,
    triggers: Vec<RegisteredTrigger>,
    next_id: u32,
    scroll_pending: bool,
    raf_scheduled: bool,
    last_resize_ms: f64,
    velocity_tracker: VelocityTracker,
}

impl SharedScrollEngine {
    #[cfg(target_arch = "wasm32")]
    fn new() -> Option<Self> {
        let scroller = Scroller::viewport();
        let scroll_handle = scroller.on_scroll(|| schedule_tick());
        let resize_handle = scroller.on_resize(|| schedule_resize());

        let resize_observer_handle = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.document_element())
            .map(|doc_el| observe_resize(&doc_el, || schedule_resize()));

        Some(Self {
            _scroll_handle: scroll_handle,
            _resize_handle: resize_handle,
            _resize_observer_handle: resize_observer_handle,
            triggers: Vec::new(),
            next_id: 1,
            scroll_pending: false,
            raf_scheduled: false,
            last_resize_ms: 0.0,
            velocity_tracker: VelocityTracker::new(),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[allow(dead_code)]
    fn new() -> Option<Self> {
        Some(Self {
            _scroll_handle: ScrollListenerHandle::default(),
            _resize_handle: ScrollListenerHandle::default(),
            triggers: Vec::new(),
            next_id: 1,
            scroll_pending: false,
            raf_scheduled: false,
            last_resize_ms: 0.0,
            velocity_tracker: VelocityTracker::new(),
        })
    }

    #[allow(dead_code)]
    fn register(&mut self, trigger: ScrollTrigger) -> u32 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        self.triggers.push(RegisteredTrigger { id, trigger });
        id
    }

    #[allow(dead_code)]
    fn unregister(&mut self, id: u32) {
        self.triggers.retain(|registered| registered.id != id);
    }

    #[allow(dead_code)]
    fn tick(&mut self) {
        self.raf_scheduled = false;
        self.scroll_pending = false;
        let now = now_ms();
        let scroller = Scroller::viewport();
        let scroll_pos = scroller.scroll_position();
        self.velocity_tracker.push(now, scroll_pos);
        let velocity = self.velocity_tracker.velocity();

        let triggers = std::mem::take(&mut self.triggers);
        for registered in &triggers {
            registered.trigger.engine_update(scroll_pos, velocity, now);
        }
        self.triggers = triggers
            .into_iter()
            .filter(|registered| !registered.trigger.is_killed())
            .collect();
    }

    #[allow(dead_code)]
    fn refresh_all(&mut self) {
        let triggers = std::mem::take(&mut self.triggers);
        for registered in &triggers {
            registered.trigger.refresh();
        }
        self.triggers = triggers;
    }
}

#[allow(dead_code)]
fn now_ms() -> f64 {
    Date::now()
}

#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
fn schedule_tick() {
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
        request_animation_frame(move || {
            SHARED_ENGINE.with(|slot| {
                if let Some(engine) = slot.borrow_mut().as_mut() {
                    engine.tick();
                }
            });
        });
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn _schedule_tick() {}

#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
fn schedule_resize() {
    SHARED_ENGINE.with(|slot| {
        let mut borrow = slot.borrow_mut();
        let Some(engine) = borrow.as_mut() else {
            return;
        };
        let now = now_ms();
        if engine.last_resize_ms != 0.0 && now - engine.last_resize_ms < 200.0 {
            request_animation_frame(move || {
                schedule_resize();
            });
            return;
        }
        engine.last_resize_ms = now;
        request_animation_frame(move || {
            SHARED_ENGINE.with(|slot| {
                if let Some(engine) = slot.borrow_mut().as_mut() {
                    engine.refresh_all();
                }
            });
        });
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn _schedule_resize() {}

pub(crate) fn register(trigger: ScrollTrigger) -> u32 {
    #[cfg(target_arch = "wasm32")]
    {
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