//! Scroll source abstraction.
//!
//! MVP supports the viewport (`window`) only. The `Element(Element)` variant is
//! reserved for custom scroller elements (deferred per `technical.md`).

#[cfg(target_arch = "wasm32")]
use std::rc::Rc;

#[cfg(target_arch = "wasm32")]
use web_sys::wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use web_sys::wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use web_sys::wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::EventTarget;

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

    /// Returns the maximum scrollable position (`scrollHeight - innerHeight`),
    /// clamped to `>= 0`.
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
                let inner_height = window.inner_height().map(|v| v.as_f64().unwrap_or(0.0)).unwrap_or(0.0);
                (scroll_height - inner_height).max(0.0)
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn max_scroll(&self) -> f64 {
        0.0
    }

    /// Returns the viewport size along the scroll axis in pixels.
    #[cfg(target_arch = "wasm32")]
    pub fn viewport_size(&self) -> f64 {
        match self {
            Scroller::Viewport => web_sys::window()
                .map(|w| w.inner_height().map(|v| v.as_f64().unwrap_or(0.0)).unwrap_or(0.0))
                .unwrap_or(0.0),
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
            Scroller::Viewport => install_window_listener("scroll", callback),
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
            Scroller::Viewport => install_window_listener("resize", callback),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn on_resize(&self, _callback: impl Fn() + 'static) -> ScrollListenerHandle {
        ScrollListenerHandle::default()
    }
}

#[cfg(target_arch = "wasm32")]
fn install_window_listener(event: &'static str, callback: impl Fn() + 'static) -> ScrollListenerHandle {
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