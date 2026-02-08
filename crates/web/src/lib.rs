use js_sys::Number;
use js_sys::{Array, Function, Object, Reflect};
use web_sys::wasm_bindgen::JsCast;
use web_sys::wasm_bindgen::JsValue;
use web_sys::{Animation, CssStyleDeclaration, Element, HtmlElement, NodeList};

const ACTIVE_ANIMATION_KEY: &str = "__fluidActiveAnimation";

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

pub fn object_set_string(object: &Object, key: &str, value: &str) {
    let _ = Reflect::set(object, &JsValue::from_str(key), &JsValue::from_str(value));
}

pub fn object_set_f64(object: &Object, key: &str, value: f64) {
    let _ = Reflect::set(object, &JsValue::from_str(key), &JsValue::from_f64(value));
}

pub fn js_number_to_string(value: f64) -> String {
    Number::from(value)
        .to_string(10)
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
    let value = Reflect::get(element.as_ref(), &JsValue::from_str(ACTIVE_ANIMATION_KEY)).ok()?;
    if value.is_null() || value.is_undefined() {
        return None;
    }
    value.dyn_into::<Animation>().ok()
}
