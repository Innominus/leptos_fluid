//! Scroll source abstraction.
//!
//! MVP supports the viewport (`window`) only. The `Element(Element)` variant is
//! reserved for custom scroller elements (deferred per `technical.md`).

#[cfg(target_arch = "wasm32")]
use std::rc::Rc;

#[cfg(target_arch = "wasm32")]
use web_sys::wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use web_sys::wasm_bindgen::{JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
use web_sys::{EventTarget, HtmlElement};

// Thread-local cache of the `100vh` sentinel measurement.
//
// CSS `100vh` is stable across mobile address-bar show/hide, unlike
// `window.innerHeight` which fluctuates. We measure a `<div style="height:
// 100vh">` once and cache the result, then use it for all viewport-size math
// so that `end: "100%"` and other viewport-relative positions don't jump when
// the address bar toggles. Re-measured by `refresh_100vh()` on resize/orientation
// change (called from the engine's `refresh_all`).
//
// `0.0` = not yet measured → `viewport_size()` falls back to `inner_height()`.
#[cfg(target_arch = "wasm32")]
thread_local! {
    static STABLE_100VH: std::cell::Cell<f64> = const { std::cell::Cell::new(0.0) };
}

/// Re-measures the `100vh` sentinel and updates the cache. Call on resize,
/// orientation change, and at the start of `refresh_all`.
#[cfg(target_arch = "wasm32")]
pub fn refresh_100vh() {
    let measured = measure_100vh();
    if measured > 0.0 {
        STABLE_100VH.with(|c| c.set(measured));
    }
}

/// Creates a hidden `<div style="height:100vh;position:absolute">`, appends it
/// to `<body>`, measures `offsetHeight`, and removes it. Returns the stable
/// viewport height in CSS pixels, or `0.0` if the DOM isn't ready.
#[cfg(target_arch = "wasm32")]
fn measure_100vh() -> f64 {
    let Some(window) = web_sys::window() else {
        return 0.0;
    };
    let Some(document) = window.document() else {
        return 0.0;
    };
    let Some(body) = document.body() else {
        return 0.0;
    };
    let Ok(div) = document.create_element("div") else {
        return 0.0;
    };
    // `100vh` in CSS is the "large viewport" (or stable viewport) height —
    // it does NOT change when the mobile address bar shows/hides, unlike
    // `window.innerHeight`. `position:absolute` + `visibility:hidden` ensures
    // the sentinel doesn't affect layout or flash.
    div.set_attribute("style", "height:100vh;position:absolute;visibility:hidden;pointer-events:none;")
        .ok();
    body.append_child(&div).ok();
    // `offset_height` is on `HtmlElement`, not `Element` — cast by reference.
    let height = div
        .dyn_ref::<HtmlElement>()
        .map(|h| h.offset_height() as f64)
        .unwrap_or(0.0);
    body.remove_child(&div).ok();
    height
}

/// Returns the stable `100vh` viewport height, measuring on first call.
/// Falls back to `window.innerHeight` if the sentinel can't be measured
/// (DOM not ready, no window, etc.).
#[cfg(target_arch = "wasm32")]
fn stable_viewport_height() -> f64 {
    let cached = STABLE_100VH.with(|c| c.get());
    if cached > 0.0 {
        return cached;
    }
    // First call — measure and cache.
    let measured = measure_100vh();
    if measured > 0.0 {
        STABLE_100VH.with(|c| c.set(measured));
        measured
    } else {
        // DOM not ready — fall back to innerHeight.
        web_sys::window()
            .map(|w| w.inner_height().map(|v| v.as_f64().unwrap_or(0.0)).unwrap_or(0.0))
            .unwrap_or(0.0)
    }
}

/// Returns the current `window.innerWidth` in CSS pixels. Used by the engine's
/// mobile address-bar resize suppression to detect orientation changes (width
/// changes → real resize, not just address bar).
#[cfg(target_arch = "wasm32")]
pub fn inner_width() -> f64 {
    web_sys::window()
        .map(|w| w.inner_width().map(|v| v.as_f64().unwrap_or(0.0)).unwrap_or(0.0))
        .unwrap_or(0.0)
}

/// Returns the current `window.innerHeight` in CSS pixels (the *dynamic* height
/// that includes address-bar fluctuations). Used as a baseline for the resize
/// suppression delta check.
#[cfg(target_arch = "wasm32")]
pub fn inner_height() -> f64 {
    web_sys::window()
        .map(|w| w.inner_height().map(|v| v.as_f64().unwrap_or(0.0)).unwrap_or(0.0))
        .unwrap_or(0.0)
}

/// The scroll source a trigger is attached to.
#[cfg_attr(test, derive(Debug))]
#[derive(Clone)]
pub enum Scroller {
    /// The browser viewport (`window`). The only supported source in MVP.
    Viewport,
    // TODO(Phase 5+): `Element(Element)` for custom scroller elements.
}

impl Scroller {
    /// Constructs a viewport scroller.
    pub fn viewport() -> Self {
        Scroller::Viewport
    }

    /// Returns the current scroll position along the scroll axis in pixels.
    #[cfg(target_arch = "wasm32")]
    pub fn scroll_position(&self) -> f64 {
        match self {
            Scroller::Viewport => web_sys::window()
                .map(|w| w.scroll_y().unwrap_or(0.0))
                .unwrap_or(0.0),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn scroll_position(&self) -> f64 {
        0.0
    }

    /// Returns the maximum scrollable position (`scrollHeight - viewport_height`),
    /// clamped to `>= 0`. Uses the stable `100vh` sentinel for the viewport
    /// height so this doesn't fluctuate with the mobile address bar.
    #[cfg(target_arch = "wasm32")]
    pub fn max_scroll(&self) -> f64 {
        match self {
            Scroller::Viewport => {
                let Some(window) = web_sys::window() else {
                    return 0.0;
                };
                let Some(document) = window.document() else {
                    return 0.0;
                };
                let Some(doc_el) = document.document_element() else {
                    return 0.0;
                };
                let scroll_height = doc_el.scroll_height() as f64;
                let viewport_height = stable_viewport_height();
                (scroll_height - viewport_height).max(0.0)
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn max_scroll(&self) -> f64 {
        0.0
    }

    /// Returns the viewport size along the scroll axis in pixels.
    ///
    /// Uses the `100vh` sentinel measurement (stable across mobile address-bar
    /// show/hide) when available, falling back to `window.innerHeight` on first
    /// call before the sentinel has been measured.
    #[cfg(target_arch = "wasm32")]
    pub fn viewport_size(&self) -> f64 {
        match self {
            Scroller::Viewport => stable_viewport_height(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn viewport_size(&self) -> f64 {
        0.0
    }

    /// Installs a `"scroll"` listener on the scroller and returns a handle whose
    /// `disconnect()` detaches it.
    #[cfg(target_arch = "wasm32")]
    pub fn on_scroll(&self, callback: impl Fn() + 'static) -> ScrollListenerHandle {
        match self {
            Scroller::Viewport => install_window_listener("scroll", Box::new(callback)),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn on_scroll(&self, _callback: impl Fn() + 'static) -> ScrollListenerHandle {
        ScrollListenerHandle::default()
    }

    /// Installs a `"resize"` listener on the scroller and returns a handle whose
    /// `disconnect()` detaches it.
    #[cfg(target_arch = "wasm32")]
    pub fn on_resize(&self, callback: impl Fn() + 'static) -> ScrollListenerHandle {
        match self {
            Scroller::Viewport => install_window_listener("resize", Box::new(callback)),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn on_resize(&self, _callback: impl Fn() + 'static) -> ScrollListenerHandle {
        ScrollListenerHandle::default()
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn install_window_listener(
    event: &'static str,
    callback: Box<dyn Fn() + 'static>,
) -> ScrollListenerHandle {
    let Some(window) = web_sys::window() else {
        return ScrollListenerHandle::default();
    };
    let target: EventTarget = window.into();
    let closure = Closure::wrap(Box::new(move |_: JsValue| {
        callback();
    }) as Box<dyn FnMut(JsValue)>);
    let _ = target.add_event_listener_with_callback(event, closure.as_ref().unchecked_ref());
    ScrollListenerHandle {
        target: Some(target),
        closure: Some(Rc::new(closure)),
        event,
    }
}

/// Handle to an installed scroll/resize listener. Dropping does not detach;
/// call `disconnect()` explicitly.
#[derive(Default)]
pub struct ScrollListenerHandle {
    #[cfg(target_arch = "wasm32")]
    target: Option<EventTarget>,
    #[cfg(target_arch = "wasm32")]
    closure: Option<Rc<Closure<dyn FnMut(JsValue)>>>,
    #[cfg(target_arch = "wasm32")]
    event: &'static str,
}

impl ScrollListenerHandle {
    /// Detaches the listener. Idempotent.
    pub fn disconnect(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            use std::ops::Deref;
            let Some(target) = self.target.take() else {
                return;
            };
            let Some(closure) = self.closure.take() else {
                return;
            };
            let _ = target.remove_event_listener_with_callback(
                &self.event,
                closure.deref().as_ref().unchecked_ref(),
            );
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // No-op: nothing was ever attached on host targets.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_constructor_returns_viewport_variant() {
        match Scroller::viewport() {
            Scroller::Viewport => {}
        }
    }

    #[test]
    fn host_defaults_are_zero() {
        let scroller = Scroller::viewport();
        assert_eq!(scroller.scroll_position(), 0.0);
        assert_eq!(scroller.max_scroll(), 0.0);
        assert_eq!(scroller.viewport_size(), 0.0);
    }

    #[test]
    fn host_listener_handle_is_noop() {
        let mut handle = Scroller::viewport().on_scroll(|| {});
        handle.disconnect();
        handle.disconnect();
    }
}