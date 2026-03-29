use std::borrow::Cow;
use std::rc::Rc;

use leptos::prelude::{GetValue, LocalStorage, RwSignal, Set, SetValue, StoredValue};
#[cfg(target_arch = "wasm32")]
use leptos_fluid_web::parse_js_f64;
use leptos_fluid_web::{
    animate_with_waapi, animation_cancel, animation_commit_styles, animation_pause, animation_play,
    animation_set_onfinish, computed_style, element_set_active_animation, html_style,
    keyframes_from_two, object_from_str_pairs, waapi_options,
};
use web_sys::wasm_bindgen::closure::Closure;
use web_sys::{Animation, CssStyleDeclaration, Element};

use crate::{FluidStyle, Transition};

type StyleProps = Vec<(Cow<'static, str>, String)>;

#[derive(Clone)]
pub(crate) struct ActiveAnimation {
    animation: Animation,
    keys: Rc<Vec<String>>,
    _on_finish: Rc<Closure<dyn FnMut()>>,
}

#[derive(Clone)]
struct TransitionRuntime {
    duration_ms: u32,
    delay_ms: u32,
    easing: String,
}

pub(crate) fn apply_style(element: &Element, style: &FluidStyle) {
    let Some(style_decl) = html_style(element) else {
        return;
    };
    style.apply_to(&style_decl);
}

fn apply_props(element: &Element, props: &StyleProps) {
    let Some(style_decl) = html_style(element) else {
        return;
    };
    for (key, value) in props {
        let _ = style_decl.set_property(key.as_ref(), value);
    }
}

fn apply_owned_props(element: &Element, props: &[(String, String)]) {
    let Some(style_decl) = html_style(element) else {
        return;
    };
    for (key, value) in props {
        let _ = style_decl.set_property(key, value);
    }
}

fn push_keyframe_prop(props: &mut Vec<(String, String)>, key: &str, value: &str) {
    if let Some((_, existing)) = props
        .iter_mut()
        .find(|(existing_key, _)| existing_key == key)
    {
        existing.clear();
        existing.push_str(value);
        return;
    }
    props.push((key.to_string(), value.to_string()));
}

fn keyframe_property_name(css_key: &str) -> String {
    if css_key.is_empty() || css_key.starts_with("--") || !css_key.contains('-') {
        return css_key.to_string();
    }

    let mut out = String::with_capacity(css_key.len());
    let mut uppercase_next = false;
    for ch in css_key.chars() {
        if ch == '-' {
            uppercase_next = true;
            continue;
        }
        if uppercase_next {
            out.extend(ch.to_uppercase());
            uppercase_next = false;
        } else {
            out.push(ch);
        }
    }
    out
}

fn normalize_transform_value(value: String) -> String {
    if value.trim().is_empty() || value.trim() == "none" {
        return "matrix(1, 0, 0, 1, 0, 0)".to_string();
    }
    value
}

fn read_computed_animation_value(computed: &CssStyleDeclaration, key: &str) -> String {
    if key == "border-color" {
        return computed
            .get_property_value("border-top-color")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| computed.get_property_value(key).ok())
            .unwrap_or_default();
    }
    computed.get_property_value(key).unwrap_or_default()
}

fn read_style_or_computed_value(
    style: &CssStyleDeclaration,
    computed: &CssStyleDeclaration,
    key: &str,
) -> String {
    let inline_value = style
        .get_property_value(key)
        .ok()
        .map(|value| value.trim().to_string())
        .unwrap_or_default();
    if !inline_value.is_empty() {
        return inline_value;
    }
    read_computed_animation_value(computed, key)
}

#[inline(never)]
fn split_animation_props(
    style: &FluidStyle,
    transition: &Transition,
) -> (StyleProps, StyleProps, TransitionRuntime) {
    let mut animated = Vec::new();
    let mut immediate = Vec::new();
    let has_excluded = !transition.excluded_properties.is_empty();

    let mut runtime = TransitionRuntime {
        duration_ms: transition.duration_ms,
        delay_ms: transition.delay_ms,
        easing: transition.easing_string().to_string(),
    };

    for (key, value) in style.to_props() {
        if key.as_ref() == "transition" {
            if let Some(parsed) = parse_transition_override(&value) {
                runtime = parsed;
            }
            continue;
        }
        if has_excluded
            && transition
                .excluded_properties
                .iter()
                .any(|excluded| excluded.as_ref() == key.as_ref())
        {
            immediate.push((key, value));
        } else {
            animated.push((key, value));
        }
    }

    (animated, immediate, runtime)
}

fn parse_transition_override(value: &str) -> Option<TransitionRuntime> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if value == "none" {
        return Some(TransitionRuntime {
            duration_ms: 0,
            delay_ms: 0,
            easing: "linear".to_string(),
        });
    }

    let rest = value.strip_prefix("all ")?;
    let (duration_raw, rest) = rest.split_once(' ')?;
    let duration_ms = parse_time_token(duration_raw)?;
    let (easing_raw, delay_ms) = if let Some((easing, delay_raw)) = rest.rsplit_once(' ') {
        if let Some(delay_ms) = parse_time_token(delay_raw) {
            (easing.trim(), delay_ms)
        } else {
            (rest.trim(), 0)
        }
    } else {
        (rest.trim(), 0)
    };
    if easing_raw.is_empty() {
        return None;
    }

    Some(TransitionRuntime {
        duration_ms,
        delay_ms,
        easing: easing_raw.to_string(),
    })
}

fn parse_time_token(token: &str) -> Option<u32> {
    let token = token.trim();
    if let Some(raw) = token.strip_suffix("ms") {
        return parse_f64_token(raw.trim()).map(|value| value.max(0.0).round() as u32);
    }
    if let Some(raw) = token.strip_suffix('s') {
        return parse_f64_token(raw.trim()).map(|value| {
            let ms = value.max(0.0) * 1000.0;
            ms.round() as u32
        });
    }
    None
}

fn parse_f64_token(token: &str) -> Option<f64> {
    #[cfg(target_arch = "wasm32")]
    {
        parse_js_f64(token)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        token.parse::<f64>().ok()
    }
}

#[inline(never)]
fn freeze_computed_values(
    element: &Element,
    keys: &[String],
    prefer_inline: bool,
) -> Vec<(String, String)> {
    if keys.is_empty() {
        return Vec::new();
    }
    let Some(style_decl) = html_style(element) else {
        return Vec::new();
    };
    let Some(computed) = computed_style(element) else {
        return Vec::new();
    };

    let mut frozen = Vec::with_capacity(keys.len());
    for key in keys {
        let mut value = if prefer_inline {
            read_style_or_computed_value(&style_decl, &computed, key)
        } else {
            read_computed_animation_value(&computed, key)
        };
        if key == "transform" {
            value = normalize_transform_value(value);
        }
        if value.trim().is_empty() {
            continue;
        }
        let _ = style_decl.set_property(key, value.trim());
        frozen.push((key.clone(), value));
    }
    frozen
}

fn snapshot_value(snapshot: &[(String, String)], key: &str) -> Option<String> {
    snapshot
        .iter()
        .find(|(snapshot_key, _)| snapshot_key == key)
        .map(|(_, value)| value.clone())
}

#[inline(never)]
pub(crate) fn cancel_active_animation(
    element: &Element,
    active_animation: StoredValue<Option<ActiveAnimation>, LocalStorage>,
) -> Vec<(String, String)> {
    let Some(active) = active_animation.get_value() else {
        return Vec::new();
    };
    let committed = animation_commit_styles(&active.animation);
    let frozen = freeze_computed_values(element, active.keys.as_ref(), committed);
    animation_set_onfinish(&active.animation, None);
    animation_cancel(&active.animation);
    element_set_active_animation(element, None);
    active_animation.set_value(None);
    frozen
}

pub(crate) fn pause_active_animation(
    active_animation: StoredValue<Option<ActiveAnimation>, LocalStorage>,
) -> bool {
    let Some(active) = active_animation.get_value() else {
        return false;
    };
    animation_pause(&active.animation)
}

pub(crate) fn resume_active_animation(
    active_animation: StoredValue<Option<ActiveAnimation>, LocalStorage>,
) -> bool {
    let Some(active) = active_animation.get_value() else {
        return false;
    };
    animation_play(&active.animation)
}

pub(crate) fn set_immediate(
    element: &Element,
    style: &FluidStyle,
    active_animation: StoredValue<Option<ActiveAnimation>, LocalStorage>,
    animation_generation: StoredValue<u32, LocalStorage>,
    is_animating: Option<RwSignal<bool>>,
) {
    let generation = animation_generation.get_value().wrapping_add(1);
    animation_generation.set_value(generation);

    cancel_active_animation(element, active_animation);
    if !style.is_empty() {
        apply_style(element, style);
    }
    if let Some(signal) = is_animating {
        signal.set(false);
    }
}

#[inline(never)]
pub(crate) fn animate_to(
    element: &Element,
    to: &FluidStyle,
    transition: &Transition,
    active_animation: StoredValue<Option<ActiveAnimation>, LocalStorage>,
    animation_generation: StoredValue<u32, LocalStorage>,
    is_animating: Option<RwSignal<bool>>,
) {
    let generation = animation_generation.get_value().wrapping_add(1);
    animation_generation.set_value(generation);

    let (animated_props, immediate_props, runtime) = split_animation_props(to, transition);
    let mut final_props = Vec::with_capacity(immediate_props.len() + animated_props.len());
    for (key, value) in immediate_props.iter().chain(animated_props.iter()) {
        final_props.push((key.as_ref().to_string(), value.clone()));
    }

    let snapshot = cancel_active_animation(element, active_animation);
    apply_props(element, &immediate_props);

    if animated_props.is_empty() {
        active_animation.set_value(None);
        if let Some(signal) = is_animating {
            signal.set(false);
        }
        return;
    }

    if runtime.duration_ms == 0 && runtime.delay_ms == 0 {
        apply_props(element, &animated_props);
        active_animation.set_value(None);
        if let Some(signal) = is_animating {
            signal.set(false);
        }
        return;
    }

    let computed = computed_style(element);
    if computed.is_none() && snapshot.is_empty() {
        apply_props(element, &animated_props);
        active_animation.set_value(None);
        if let Some(signal) = is_animating {
            signal.set(false);
        }
        return;
    }

    let mut from_props = Vec::with_capacity(animated_props.len());
    let mut to_props = Vec::with_capacity(animated_props.len());
    let mut animated_keys = Vec::with_capacity(animated_props.len());
    for (css_key, to_value) in &animated_props {
        let css_key = css_key.as_ref();
        let mut from_value = snapshot_value(&snapshot, css_key).unwrap_or_else(|| {
            computed
                .as_ref()
                .map(|style| read_computed_animation_value(style, css_key))
                .unwrap_or_default()
        });
        let mut to_value = to_value.clone();
        if css_key == "transform" {
            from_value = normalize_transform_value(from_value);
            to_value = normalize_transform_value(to_value);
        }
        let keyframe_key = keyframe_property_name(css_key);
        push_keyframe_prop(&mut from_props, &keyframe_key, &from_value);
        push_keyframe_prop(&mut to_props, &keyframe_key, &to_value);
        animated_keys.push(css_key.to_string());
    }

    let mut frame_from_entries = Vec::with_capacity(from_props.len());
    for (key, value) in &from_props {
        frame_from_entries.push((key.as_str(), value.as_str()));
    }
    let frame_from = object_from_str_pairs(&frame_from_entries);

    let mut frame_to_entries = Vec::with_capacity(to_props.len());
    for (key, value) in &to_props {
        frame_to_entries.push((key.as_str(), value.as_str()));
    }
    let frame_to = object_from_str_pairs(&frame_to_entries);
    let keyframes = keyframes_from_two(&frame_from, &frame_to);

    let animation_options = waapi_options(
        runtime.duration_ms.max(1),
        runtime.delay_ms,
        &runtime.easing,
        "both",
    );
    let Some(animation) = animate_with_waapi(element, &keyframes, &animation_options) else {
        apply_props(element, &animated_props);
        active_animation.set_value(None);
        if let Some(signal) = is_animating {
            signal.set(false);
        }
        return;
    };

    if let Some(signal) = is_animating {
        signal.set(true);
    }

    let inner_element = element.clone();
    let inner_final_props = Rc::new(final_props);
    let on_finish = Rc::new(Closure::wrap(Box::new(move || {
        if animation_generation.get_value() != generation {
            return;
        }
        apply_owned_props(&inner_element, inner_final_props.as_ref());
        element_set_active_animation(&inner_element, None);
        active_animation.set_value(None);
        if let Some(signal) = is_animating {
            signal.set(false);
        }
    }) as Box<dyn FnMut()>));
    animation_set_onfinish(&animation, Some(on_finish.as_ref().as_ref()));
    element_set_active_animation(element, Some(&animation));

    active_animation.set_value(Some(ActiveAnimation {
        animation,
        keys: Rc::new(animated_keys),
        _on_finish: on_finish,
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FluidStyle;

    fn has_prop(props: &StyleProps, key: &str, value: &str) -> bool {
        props
            .iter()
            .any(|(prop_key, prop_value)| prop_key.as_ref() == key && prop_value == value)
    }

    #[test]
    fn parse_time_token_supports_ms_and_seconds() {
        assert_eq!(parse_time_token("180ms"), Some(180));
        assert_eq!(parse_time_token("0.25s"), Some(250));
        assert_eq!(parse_time_token("2s"), Some(2000));
        assert_eq!(parse_time_token("bogus"), None);
    }

    #[test]
    fn parse_transition_override_reads_runtime_fields() {
        let parsed = parse_transition_override("all 320ms ease-in 40ms").unwrap();
        assert_eq!(parsed.duration_ms, 320);
        assert_eq!(parsed.delay_ms, 40);
        assert_eq!(parsed.easing, "ease-in");

        let parsed_without_delay = parse_transition_override("all 140ms ease-out").unwrap();
        assert_eq!(parsed_without_delay.duration_ms, 140);
        assert_eq!(parsed_without_delay.delay_ms, 0);
        assert_eq!(parsed_without_delay.easing, "ease-out");

        let none_transition = parse_transition_override("none").unwrap();
        assert_eq!(none_transition.duration_ms, 0);
        assert_eq!(none_transition.delay_ms, 0);
        assert_eq!(none_transition.easing, "linear");

        assert!(parse_transition_override("opacity 100ms linear").is_none());
    }

    #[test]
    fn split_animation_props_honors_exclusions_and_style_override() {
        let style = FluidStyle::new()
            .with("opacity", "0.84")
            .with("width", "120px")
            .with("transition", "all 460ms linear 30ms");
        let transition = Transition::new()
            .duration_ms(120)
            .exclude_properties(&["width"]);

        let (animated, immediate, runtime) = split_animation_props(&style, &transition);

        assert!(has_prop(&animated, "opacity", "0.84"));
        assert!(has_prop(&immediate, "width", "120px"));
        assert_eq!(runtime.duration_ms, 460);
        assert_eq!(runtime.delay_ms, 30);
        assert_eq!(runtime.easing, "linear");
    }

    #[test]
    fn keyframe_property_name_camel_cases_css_keys() {
        assert_eq!(
            keyframe_property_name("background-color"),
            "backgroundColor"
        );
        assert_eq!(keyframe_property_name("opacity"), "opacity");
        assert_eq!(keyframe_property_name("--fluid-token"), "--fluid-token");
    }

    #[test]
    fn normalize_transform_rewrites_none_to_identity() {
        assert_eq!(
            normalize_transform_value("none".to_string()),
            "matrix(1, 0, 0, 1, 0, 0)"
        );
        assert_eq!(
            normalize_transform_value(" ".to_string()),
            "matrix(1, 0, 0, 1, 0, 0)"
        );
        assert_eq!(
            normalize_transform_value("translate3d(10px, 0px, 0px)".to_string()),
            "translate3d(10px, 0px, 0px)"
        );
    }
}
