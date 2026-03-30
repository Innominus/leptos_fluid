use self::corrections::{
    read_border_radius_target, run_border_radius_correction, run_scale_correction_animations,
};
use leptos::prelude::*;
use leptos_fluid_web::{
    animate_with_waapi, animation_cancel, animation_set_onfinish, computed_style, css_push_number,
    css_push_px, html_style, keyframes_from_two, node_list_to_elements, object_from_str_pairs,
    restore_inline_property, safe_f64_ratio, waapi_options,
};
use std::cell::Cell;
use std::rc::Rc;
use web_sys::{Animation, Element, wasm_bindgen::prelude::Closure};

mod builders;
mod corrections;
mod group;
mod options;
mod single;
mod target;

pub use builders::{FlipBuilder, FlipGroupBuilder, ReadyFlipBuilder, ReadyFlipGroupBuilder};
pub use group::FlipGroup;
pub use options::{Easing, FlipOptions, ScaleMode};
pub use single::Flip;
pub use target::FlipTarget;

pub(crate) use target::FlipTargetSource;

const LINEAR: &str = "linear(\n    0, 0.009, 0.035 2.1%, 0.141, 0.281 6.7%, 0.723 12.9%, 0.938 16.7%, 1.017,\n    1.077, 1.121, 1.149 24.3%, 1.159, 1.163, 1.161, 1.154 29.9%, 1.129 32.8%,\n    1.051 39.6%, 1.017 43.1%, 0.991, 0.977 51%, 0.974 53.8%, 0.975 57.1%,\n    0.997 69.8%, 1.003 76.9%, 1.004 83.8%, 1\n)";

const EASE_IN_OUT: &str = "cubic-bezier(0.77, 0, 0.175, 1)";
pub(crate) const FLIP_DELTA_EPSILON: f64 = 0.1;

#[derive(Debug, Clone, Copy)]
pub struct FlipValues {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
    border_radius: Option<BorderRadiusTarget>,
}

#[derive(Clone)]
pub(crate) struct InlineStyles {
    transform: String,
    transform_origin: String,
    will_change: String,
    transition: String,
}

#[derive(Clone)]
pub(crate) struct FlipAnimation {
    animation: Option<Animation>,
    element: Element,
    inline_styles: InlineStyles,
    scale_corrections: Rc<Vec<ScaleCorrectionAnimation>>,
    border_radius_correction: Option<BorderRadiusCorrectionAnimation>,
}

#[derive(Clone)]
pub(crate) struct ScaleCorrectionAnimation {
    stop_signal: Rc<Cell<bool>>,
    element: Element,
    inline_styles: Rc<InlineStyles>,
}

#[derive(Clone)]
pub(crate) struct ScaleCorrectionTarget {
    element: Element,
    offset_x: f64,
    offset_y: f64,
}

#[derive(Clone)]
pub(crate) struct BorderRadiusInlineStyles {
    top_left: String,
    top_right: String,
    bottom_right: String,
    bottom_left: String,
}

#[derive(Clone)]
pub(crate) struct BorderRadiusCorrectionAnimation {
    stop_signal: Rc<Cell<bool>>,
    inline_styles: Rc<BorderRadiusInlineStyles>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct BorderRadiusTarget {
    top_left: RadiusPair,
    top_right: RadiusPair,
    bottom_right: RadiusPair,
    bottom_left: RadiusPair,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RadiusPair {
    x: f64,
    y: f64,
}

#[derive(Clone)]
pub(crate) struct FlipItem {
    key: String,
    element: Element,
    values: FlipValues,
}

pub(crate) fn capture_inline_styles(element: &Element) -> InlineStyles {
    let Some(style) = html_style(element) else {
        return InlineStyles {
            transform: String::new(),
            transform_origin: String::new(),
            will_change: String::new(),
            transition: String::new(),
        };
    };
    InlineStyles {
        transform: style.get_property_value("transform").unwrap_or_default(),
        transform_origin: style
            .get_property_value("transform-origin")
            .unwrap_or_default(),
        will_change: style.get_property_value("will-change").unwrap_or_default(),
        transition: style.get_property_value("transition").unwrap_or_default(),
    }
}

pub(crate) fn capture_border_radius_inline_styles(element: &Element) -> BorderRadiusInlineStyles {
    let Some(style) = html_style(element) else {
        return BorderRadiusInlineStyles {
            top_left: String::new(),
            top_right: String::new(),
            bottom_right: String::new(),
            bottom_left: String::new(),
        };
    };
    BorderRadiusInlineStyles {
        top_left: style
            .get_property_value("border-top-left-radius")
            .unwrap_or_default(),
        top_right: style
            .get_property_value("border-top-right-radius")
            .unwrap_or_default(),
        bottom_right: style
            .get_property_value("border-bottom-right-radius")
            .unwrap_or_default(),
        bottom_left: style
            .get_property_value("border-bottom-left-radius")
            .unwrap_or_default(),
    }
}

pub(crate) fn apply_inline_transform(element: &Element, transform: &str) {
    let Some(style) = html_style(element) else {
        return;
    };
    let _ = style.set_property("transform-origin", "0 0");
    let _ = style.set_property("transform", transform);
    let _ = style.set_property("will-change", "transform");
}

pub(crate) fn restore_inline_styles(element: &Element, inline_styles: &InlineStyles) {
    let Some(style) = html_style(element) else {
        return;
    };
    restore_inline_property(&style, "transform", &inline_styles.transform);
    restore_inline_property(&style, "transform-origin", &inline_styles.transform_origin);
    restore_inline_property(&style, "will-change", &inline_styles.will_change);
    restore_inline_property(&style, "transition", &inline_styles.transition);
}

pub(crate) fn restore_border_radius_inline_styles(
    element: &Element,
    inline_styles: &BorderRadiusInlineStyles,
) {
    let Some(style) = html_style(element) else {
        return;
    };
    restore_inline_property(&style, "border-top-left-radius", &inline_styles.top_left);
    restore_inline_property(&style, "border-top-right-radius", &inline_styles.top_right);
    restore_inline_property(
        &style,
        "border-bottom-right-radius",
        &inline_styles.bottom_right,
    );
    restore_inline_property(
        &style,
        "border-bottom-left-radius",
        &inline_styles.bottom_left,
    );
}

pub(crate) fn apply_computed_transform(element: &Element) {
    let Some(computed) = computed_style(element) else {
        return;
    };
    let Ok(transform) = computed.get_property_value("transform") else {
        return;
    };
    let Ok(origin) = computed.get_property_value("transform-origin") else {
        return;
    };
    let Some(style) = html_style(element) else {
        return;
    };
    let transform_value = if transform.trim().is_empty() {
        "none"
    } else {
        transform.trim()
    };
    let _ = style.set_property("transform", transform_value);
    if !origin.trim().is_empty() {
        let _ = style.set_property("transform-origin", origin.trim());
    }
    let _ = style.set_property("will-change", "transform");
}

pub(crate) fn query_elements(selector: &str) -> Vec<Element> {
    let Ok(list) = document().query_selector_all(selector) else {
        return Vec::new();
    };
    node_list_to_elements(list)
}

pub(crate) fn query_elements_within(root: &Element, selector: &str) -> Vec<Element> {
    let Ok(list) = root.query_selector_all(selector) else {
        return Vec::new();
    };
    node_list_to_elements(list)
}

pub(crate) fn snapshot_elements(elements: Vec<Element>) -> Vec<FlipItem> {
    elements
        .into_iter()
        .enumerate()
        .map(|(index, element)| {
            let key = element_key(&element).unwrap_or_else(|| {
                let mut value = String::from("__flip-index-");
                value.push_str(&index.to_string());
                value
            });
            let (_, values) = single::Flip::rect(element.clone());
            FlipItem {
                key,
                element,
                values,
            }
        })
        .collect()
}

pub(crate) fn element_key(element: &Element) -> Option<String> {
    element
        .get_attribute("data-flip-id")
        .or_else(|| element.get_attribute("id"))
        .filter(|value| !value.is_empty())
}

#[inline(never)]
pub(crate) fn stop_group_animations(
    animations_store: StoredValue<Vec<FlipAnimation>, LocalStorage>,
) -> Vec<(String, InlineStyles)> {
    let active_animations = animations_store.get_value();
    let mut carried_inline = Vec::new();

    for animation in active_animations {
        stop_flip_animation_state(&animation);
        if let Some(key) = element_key(&animation.element) {
            carried_inline.push((key, animation.inline_styles.clone()));
        }
    }

    animations_store.set_value(Vec::new());
    carried_inline
}

#[inline(never)]
pub(crate) fn stop_flip_animation_state(animation: &FlipAnimation) {
    apply_computed_transform(&animation.element);
    if let Some(active) = animation.animation.as_ref() {
        animation_cancel(active);
    }
    if let Some(correction) = animation.border_radius_correction.as_ref() {
        correction.stop_signal.set(true);
        restore_border_radius_inline_styles(&animation.element, correction.inline_styles.as_ref());
    }
    for correction in animation.scale_corrections.iter() {
        correction.stop_signal.set(true);
        restore_inline_styles(&correction.element, correction.inline_styles.as_ref());
    }
}

pub(crate) fn find_inline_by_key<'a>(
    entries: &'a [(String, InlineStyles)],
    key: &str,
) -> Option<&'a InlineStyles> {
    entries
        .iter()
        .find_map(|(entry_key, value)| (entry_key == key).then_some(value))
}

pub(crate) fn find_values_by_key<'a>(
    entries: &'a [(String, FlipValues)],
    key: &str,
) -> Option<&'a FlipValues> {
    entries
        .iter()
        .find_map(|(entry_key, value)| (entry_key == key).then_some(value))
}

pub(crate) fn has_flip_delta_with_size(
    from: &FlipValues,
    to: &FlipValues,
    include_size_delta: bool,
) -> bool {
    let has_position_delta = (from.left - to.left).abs() > FLIP_DELTA_EPSILON
        || (from.top - to.top).abs() > FLIP_DELTA_EPSILON;
    if has_position_delta {
        return true;
    }
    include_size_delta
        && ((from.width - to.width).abs() > FLIP_DELTA_EPSILON
            || (from.height - to.height).abs() > FLIP_DELTA_EPSILON)
}

#[inline(never)]
pub(crate) fn run_flip_animation(
    element: Element,
    from: FlipValues,
    to: FlipValues,
    options: FlipOptions,
    on_finish: Rc<dyn Fn()>,
) -> FlipAnimation {
    let dx = from.left - to.left;
    let dy = from.top - to.top;
    let use_scale = options.scale_mode.uses_scale();
    let scale_x = if use_scale {
        safe_f64_ratio(from.width, to.width)
    } else {
        1.0
    };
    let scale_y = if use_scale {
        safe_f64_ratio(from.height, to.height)
    } else {
        1.0
    };

    let transform_from = if use_scale {
        build_translate_scale_transform(dx, dy, scale_x, scale_y)
    } else {
        build_translate_transform(dx, dy)
    };
    let transform_to = if use_scale {
        "translate(0px, 0px) scale(1, 1)"
    } else {
        "translate(0px, 0px)"
    };

    let inline_styles = capture_inline_styles(&element);
    apply_inline_transform(&element, &transform_from);

    let frame_from = object_from_str_pairs(&[("transform", transform_from.as_str())]);
    let frame_to = object_from_str_pairs(&[("transform", transform_to)]);
    let keyframes = keyframes_from_two(&frame_from, &frame_to);

    let animation_options = waapi_options(
        options.duration.max(1) as u32,
        options.delay as u32,
        options.easing.get_easing_fn(),
        "backwards",
    );
    let animation = animate_with_waapi(&element, &keyframes, &animation_options);

    let border_radius_correction = if use_scale {
        run_border_radius_correction(
            &element,
            from.border_radius,
            to.border_radius,
            scale_x,
            scale_y,
        )
    } else {
        None
    };

    let scale_corrections = if use_scale {
        options
            .scale_correction_selector
            .map(|selector| run_scale_correction_animations(&element, selector, scale_x, scale_y))
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let scale_corrections = Rc::new(scale_corrections);

    let inner_element = element.clone();
    let inner_inline_styles = inline_styles.clone();
    let border_radius_correction_for_finish = border_radius_correction.clone();
    let scale_corrections_for_finish = scale_corrections.clone();
    let on_complete: Rc<dyn Fn()> = Rc::new(move || {
        if let Some(correction) = &border_radius_correction_for_finish {
            correction.stop_signal.set(true);
            restore_border_radius_inline_styles(&inner_element, correction.inline_styles.as_ref());
        }
        for correction in scale_corrections_for_finish.iter() {
            correction.stop_signal.set(true);
            restore_inline_styles(&correction.element, correction.inline_styles.as_ref());
        }
        restore_inline_styles(&inner_element, &inner_inline_styles);
        on_finish();
    });
    if let Some(animation) = animation.as_ref() {
        let on_complete = Closure::wrap(Box::new(move || on_complete()) as Box<dyn FnMut()>);
        animation_set_onfinish(animation, Some(on_complete.as_ref()));
        on_complete.forget();
    } else {
        request_animation_frame(move || on_complete());
    }

    FlipAnimation {
        animation,
        element,
        inline_styles,
        scale_corrections,
        border_radius_correction,
    }
}

pub(crate) fn build_translate_transform(dx: f64, dy: f64) -> String {
    let mut out = String::from("translate(");
    css_push_px(&mut out, dx);
    out.push_str(", ");
    css_push_px(&mut out, dy);
    out.push(')');
    out
}

pub(crate) fn build_translate_scale_transform(
    dx: f64,
    dy: f64,
    scale_x: f64,
    scale_y: f64,
) -> String {
    let mut out = build_translate_transform(dx, dy);
    out.push_str(" scale(");
    css_push_number(&mut out, scale_x);
    out.push_str(", ");
    css_push_number(&mut out, scale_y);
    out.push(')');
    out
}

#[cfg(test)]
mod tests {
    use super::{Easing, FlipOptions, FlipValues, ScaleMode, has_flip_delta_with_size};

    #[test]
    fn default_options_match_builder_defaults() {
        let options = FlipOptions::new();

        assert_eq!(options, FlipOptions::default());
        assert_eq!(options.duration, 240);
        assert_eq!(options.delay, 0);
        assert_eq!(options.stagger, 0);
        assert_eq!(options.easing, Easing::EaseInOut);
        assert_eq!(options.scale_mode, ScaleMode::PositionOnly);
        assert_eq!(options.scale_correction_selector, None);
    }

    #[test]
    fn options_builder_updates_fields() {
        let options = FlipOptions::new()
            .duration_ms(640)
            .delay_ms(40)
            .stagger_ms(12)
            .scale_mode(ScaleMode::PositionAndScale);

        assert_eq!(options.duration, 640);
        assert_eq!(options.delay, 40);
        assert_eq!(options.stagger, 12);
        assert!(options.scale_mode.uses_scale());
    }

    #[test]
    fn delta_detection_respects_scale_mode() {
        let from = FlipValues {
            left: 0.0,
            top: 0.0,
            width: 120.0,
            height: 40.0,
            border_radius: None,
        };
        let to = FlipValues {
            left: 0.0,
            top: 0.0,
            width: 220.0,
            height: 40.0,
            border_radius: None,
        };

        assert!(!has_flip_delta_with_size(&from, &to, false));
        assert!(has_flip_delta_with_size(&from, &to, true));
    }
}
