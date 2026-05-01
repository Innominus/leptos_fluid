use js_sys::{Array, Number, Object, Reflect};
use web_sys::wasm_bindgen::{JsCast, JsValue};

pub fn object_set_string(object: &Object, key: &str, value: &str) {
    let _ = Reflect::set(object, &JsValue::from_str(key), &JsValue::from_str(value));
}

pub fn object_set_f64(object: &Object, key: &str, value: f64) {
    let _ = Reflect::set(object, &JsValue::from_str(key), &JsValue::from_f64(value));
}

pub fn js_number_to_string(value: f64) -> String {
    Number::from(value)
        .to_string_with_radix(10)
        .ok()
        .and_then(|value| value.as_string())
        .unwrap_or_default()
}

pub fn css_push_number(out: &mut String, value: f64) {
    out.push_str(&js_number_to_string(value));
}

pub fn css_push_px(out: &mut String, value: f64) {
    css_push_number(out, value);
    out.push_str("px");
}

pub fn css_px_string(value: f64) -> String {
    let mut out = js_number_to_string(value);
    out.push_str("px");
    out
}

pub fn safe_f64_ratio(numerator: f64, denominator: f64) -> f64 {
    if denominator.abs() <= f64::EPSILON {
        return 1.0;
    }
    let value = numerator / denominator;
    if value.is_finite() { value } else { 1.0 }
}

pub fn parse_js_f64(value: &str) -> Option<f64> {
    let parsed = js_sys::parse_float(value);
    if parsed.is_finite() {
        Some(parsed)
    } else {
        None
    }
}

pub fn object_from_str_pairs(pairs: &[(&str, &str)]) -> Object {
    let object = Object::new();
    for (key, value) in pairs {
        object_set_string(&object, key, value);
    }
    object
}

pub fn keyframes_from_two(from: &Object, to: &Object) -> Object {
    let keyframes = Array::new();
    keyframes.push(from);
    keyframes.push(to);
    keyframes.unchecked_into()
}

#[cfg(feature = "style")]
mod style {
    use web_sys::{CssStyleDeclaration, Element, HtmlElement, wasm_bindgen::JsCast};

    pub fn html_style(element: &Element) -> Option<CssStyleDeclaration> {
        element.dyn_ref::<HtmlElement>().map(|el| el.style())
    }

    pub fn computed_style(element: &Element) -> Option<CssStyleDeclaration> {
        let window = web_sys::window()?;
        window.get_computed_style(element).ok()?
    }

    pub fn restore_inline_property(style: &CssStyleDeclaration, property: &str, value: &str) {
        if value.is_empty() {
            let _ = style.remove_property(property);
        } else {
            let _ = style.set_property(property, value);
        }
    }
}

#[cfg(feature = "style")]
pub use style::{computed_style, html_style, restore_inline_property};

#[cfg(feature = "dom-query")]
mod dom_query {
    use web_sys::{Element, NodeList, wasm_bindgen::JsCast};

    pub fn node_list_to_elements(list: NodeList) -> Vec<Element> {
        let mut elements = Vec::new();
        let length = list.length();
        for index in 0..length {
            let Some(node) = list.get(index) else {
                continue;
            };
            let Ok(element) = node.dyn_into::<Element>() else {
                continue;
            };
            elements.push(element);
        }
        elements
    }
}

#[cfg(feature = "dom-query")]
pub use dom_query::node_list_to_elements;

#[cfg(feature = "resize-observer")]
mod resize_observer {
    use web_sys::Element;

    #[cfg(target_arch = "wasm32")]
    mod wasm {
        use std::cell::RefCell;
        use std::rc::Rc;

        use js_sys::Array;
        use web_sys::wasm_bindgen::JsCast;
        use web_sys::wasm_bindgen::closure::Closure;
        use web_sys::{Element, ResizeObserver, ResizeObserverEntry};

        pub(super) struct ResizeSubscription {
            id: u32,
            element: Element,
            callback: Rc<dyn Fn()>,
        }

        #[derive(Default)]
        pub(super) struct SharedResizeObserverState {
            next_id: u32,
            subscriptions: Vec<ResizeSubscription>,
            observed_counts: Vec<(Element, usize)>,
        }

        pub(super) struct SharedResizeObserver {
            observer: ResizeObserver,
            state: Rc<RefCell<SharedResizeObserverState>>,
            _callback: Closure<dyn FnMut(Array, ResizeObserver)>,
        }

        thread_local! {
            static SHARED_RESIZE_OBSERVER: RefCell<Option<SharedResizeObserver>> = const { RefCell::new(None) };
        }

        impl SharedResizeObserver {
            fn new() -> Option<Self> {
                let state = Rc::new(RefCell::new(SharedResizeObserverState::default()));
                let callback_state = state.clone();
                let callback =
                    Closure::wrap(Box::new(move |entries: Array, _observer: ResizeObserver| {
                        for entry in entries.iter() {
                            let entry: ResizeObserverEntry = entry.unchecked_into();
                            let target = entry.target();
                            let callbacks = {
                                let state = callback_state.borrow();
                                state
                                    .subscriptions
                                    .iter()
                                    .filter(|subscription| subscription.element == target)
                                    .map(|subscription| subscription.callback.clone())
                                    .collect::<Vec<_>>()
                            };

                            for callback in callbacks {
                                callback();
                            }
                        }
                    })
                        as Box<dyn FnMut(Array, ResizeObserver)>);

                let observer = ResizeObserver::new(callback.as_ref().unchecked_ref()).ok()?;
                Some(Self {
                    observer,
                    state,
                    _callback: callback,
                })
            }
        }

        fn with_shared_resize_observer<R>(f: impl FnOnce(&SharedResizeObserver) -> R) -> Option<R> {
            SHARED_RESIZE_OBSERVER.with(|slot| {
                if slot.borrow().is_none() {
                    *slot.borrow_mut() = SharedResizeObserver::new();
                }

                let borrow = slot.borrow();
                let observer = borrow.as_ref()?;
                Some(f(observer))
            })
        }

        pub(super) fn remove_resize_subscription(subscription_id: u32) {
            SHARED_RESIZE_OBSERVER.with(|slot| {
                let borrow = slot.borrow();
                let Some(observer) = borrow.as_ref() else {
                    return;
                };

                let removed_element = {
                    let mut state = observer.state.borrow_mut();
                    let Some(index) = state
                        .subscriptions
                        .iter()
                        .position(|subscription| subscription.id == subscription_id)
                    else {
                        return;
                    };

                    let removed = state.subscriptions.remove(index);
                    if let Some((_, count)) = state
                        .observed_counts
                        .iter_mut()
                        .find(|(element, _)| *element == removed.element)
                    {
                        *count = count.saturating_sub(1);
                    }
                    removed.element
                };

                let should_unobserve = {
                    let mut state = observer.state.borrow_mut();
                    if let Some(index) = state
                        .observed_counts
                        .iter()
                        .position(|(element, count)| *element == removed_element && *count == 0)
                    {
                        state.observed_counts.remove(index);
                        true
                    } else {
                        false
                    }
                };

                if should_unobserve {
                    observer.observer.unobserve(&removed_element);
                }
            });
        }

        pub(super) fn observe_resize(element: &Element, callback: impl Fn() + 'static) -> u32 {
            let callback = Rc::new(callback);
            with_shared_resize_observer(|observer| {
                let mut state = observer.state.borrow_mut();
                let subscription_id = state.next_id;
                state.next_id = state.next_id.wrapping_add(1);
                state.subscriptions.push(ResizeSubscription {
                    id: subscription_id,
                    element: element.clone(),
                    callback,
                });

                let mut should_observe = false;
                if let Some((_, count)) = state
                    .observed_counts
                    .iter_mut()
                    .find(|(observed, _)| *observed == *element)
                {
                    *count += 1;
                } else {
                    state.observed_counts.push((element.clone(), 1));
                    should_observe = true;
                }
                drop(state);

                if should_observe {
                    observer.observer.observe(element);
                }

                subscription_id
            })
            .unwrap_or(0)
        }
    }

    pub struct ResizeObserverHandle {
        subscription_id: Option<u32>,
    }

    impl ResizeObserverHandle {
        pub fn disconnect(&mut self) {
            #[cfg(target_arch = "wasm32")]
            {
                let Some(subscription_id) = self.subscription_id.take() else {
                    return;
                };
                wasm::remove_resize_subscription(subscription_id);
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                self.subscription_id = None;
            }
        }
    }

    pub fn observe_resize<F>(element: &Element, callback: F) -> ResizeObserverHandle
    where
        F: Fn() + 'static,
    {
        #[cfg(target_arch = "wasm32")]
        let subscription_id = Some(wasm::observe_resize(element, callback));

        #[cfg(not(target_arch = "wasm32"))]
        let subscription_id = {
            let _ = element;
            let _ = callback;
            None
        };

        ResizeObserverHandle { subscription_id }
    }
}

#[cfg(feature = "resize-observer")]
pub use resize_observer::{ResizeObserverHandle, observe_resize};

#[cfg(feature = "waapi")]
mod waapi {
    use js_sys::{Function, Object, Reflect};
    use web_sys::wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Animation, Element};

    use crate::{object_set_f64, object_set_string};

    const ACTIVE_ANIMATION_KEY: &str = "__fluidActiveAnimation";

    pub fn waapi_options(duration_ms: u32, delay_ms: u32, easing: &str, fill: &str) -> Object {
        let options = Object::new();
        object_set_f64(&options, "duration", duration_ms as f64);
        object_set_f64(&options, "delay", delay_ms as f64);
        object_set_string(&options, "easing", easing);
        object_set_string(&options, "fill", fill);
        options
    }

    pub fn animate_with_waapi(
        element: &Element,
        keyframes: &Object,
        options: &Object,
    ) -> Option<Animation> {
        let animate_fn = Reflect::get(element.as_ref(), &JsValue::from_str("animate"))
            .ok()?
            .dyn_into::<Function>()
            .ok()?;
        let animation = animate_fn
            .call2(element.as_ref(), keyframes.as_ref(), options.as_ref())
            .ok()?;
        animation.dyn_into::<Animation>().ok()
    }

    pub fn animation_cancel(animation: &Animation) {
        animation.cancel();
    }

    pub fn animation_commit_styles(animation: &Animation) -> bool {
        let Some(commit_fn) = Reflect::get(animation.as_ref(), &JsValue::from_str("commitStyles"))
            .ok()
            .and_then(|value| value.dyn_into::<Function>().ok())
        else {
            return false;
        };
        commit_fn.call0(animation.as_ref()).is_ok()
    }

    pub fn animation_set_onfinish(animation: &Animation, callback: Option<&JsValue>) {
        let callback = callback.and_then(|value| value.dyn_ref::<Function>());
        animation.set_onfinish(callback);
    }

    pub fn animation_pause(animation: &Animation) -> bool {
        animation.pause().is_ok()
    }

    pub fn animation_play(animation: &Animation) -> bool {
        animation.play().is_ok()
    }

    pub fn element_set_active_animation(element: &Element, animation: Option<&Animation>) {
        let key = JsValue::from_str(ACTIVE_ANIMATION_KEY);
        let value = animation
            .map(|value| JsValue::from(value.clone()))
            .unwrap_or(JsValue::NULL);
        let _ = Reflect::set(element.as_ref(), &key, &value);
    }

    pub fn element_get_active_animation(element: &Element) -> Option<Animation> {
        let value =
            Reflect::get(element.as_ref(), &JsValue::from_str(ACTIVE_ANIMATION_KEY)).ok()?;
        if value.is_null() || value.is_undefined() {
            return None;
        }
        value.dyn_into::<Animation>().ok()
    }
}

#[cfg(feature = "waapi")]
pub use waapi::{
    animate_with_waapi, animation_cancel, animation_commit_styles, animation_pause, animation_play,
    animation_set_onfinish, element_get_active_animation, element_set_active_animation,
    waapi_options,
};
