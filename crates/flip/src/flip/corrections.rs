use super::*;
use leptos_fluid_web::{css_push_number, css_push_px, parse_js_f64, safe_f64_ratio};
use std::cell::Cell;
use std::rc::Rc;
use web_sys::{CssStyleDeclaration, Element};

#[inline(never)]
pub(super) fn run_scale_correction_animations(
    root: &Element,
    selector: &str,
    initial_scale_x: f64,
    initial_scale_y: f64,
) -> Vec<ScaleCorrectionAnimation> {
    if (initial_scale_x - 1.0).abs() <= FLIP_DELTA_EPSILON
        && (initial_scale_y - 1.0).abs() <= FLIP_DELTA_EPSILON
    {
        return Vec::new();
    }

    let fallback_inv_scale_x = safe_f64_ratio(1.0, initial_scale_x);
    let fallback_inv_scale_y = safe_f64_ratio(1.0, initial_scale_y);
    let stop_signal = Rc::new(Cell::new(false));
    let root_rect = root.get_bounding_client_rect();
    let mut animations = Vec::new();
    let mut correction_targets = Vec::new();

    for element in query_elements_within(root, selector).into_iter() {
        let rect = element.get_bounding_client_rect();
        // `getBoundingClientRect` is in viewport space and already includes the
        // root's current FLIP scale. Convert back to root-local offsets.
        let offset_x = (rect.left() - root_rect.left()) * fallback_inv_scale_x;
        let offset_y = (rect.top() - root_rect.top()) * fallback_inv_scale_y;
        let inline_styles = capture_inline_styles(&element);
        if let Some(style) = html_style(&element) {
            let _ = style.set_property("transform-origin", "0 0");
            let _ = style.set_property("will-change", "transform");
        }
        apply_inline_scale_correction(
            &element,
            fallback_inv_scale_x,
            fallback_inv_scale_y,
            offset_x,
            offset_y,
        );
        correction_targets.push(ScaleCorrectionTarget {
            element: element.clone(),
            offset_x,
            offset_y,
        });

        animations.push(ScaleCorrectionAnimation {
            stop_signal: stop_signal.clone(),
            element,
            inline_styles: Rc::new(inline_styles),
        });
    }

    if !correction_targets.is_empty() {
        schedule_scale_correction_frame(
            root.clone(),
            Rc::new(correction_targets),
            stop_signal,
            fallback_inv_scale_x,
            fallback_inv_scale_y,
        );
    }

    animations
}

#[inline(never)]
fn schedule_scale_correction_frame(
    root: Element,
    correction_targets: Rc<Vec<ScaleCorrectionTarget>>,
    stop_signal: Rc<Cell<bool>>,
    fallback_inv_scale_x: f64,
    fallback_inv_scale_y: f64,
) {
    request_animation_frame(move || {
        if stop_signal.get() {
            return;
        }

        let (inv_scale_x, inv_scale_y) = current_inverse_scale(&root)
            .map(|(scale_x, scale_y)| (safe_f64_ratio(1.0, scale_x), safe_f64_ratio(1.0, scale_y)))
            .unwrap_or((fallback_inv_scale_x, fallback_inv_scale_y));

        for target in correction_targets.iter() {
            apply_inline_scale_correction(
                &target.element,
                inv_scale_x,
                inv_scale_y,
                target.offset_x,
                target.offset_y,
            );
        }

        schedule_scale_correction_frame(
            root,
            correction_targets,
            stop_signal,
            fallback_inv_scale_x,
            fallback_inv_scale_y,
        );
    });
}

#[inline(never)]
fn apply_inline_scale_correction(
    element: &Element,
    inv_scale_x: f64,
    inv_scale_y: f64,
    offset_x: f64,
    offset_y: f64,
) {
    let Some(style) = html_style(element) else {
        return;
    };
    // Counteract parent scale and keep the corrected child anchored in place.
    let translate_x = offset_x * (inv_scale_x - 1.0);
    let translate_y = offset_y * (inv_scale_y - 1.0);
    let mut transform = String::from("translate3d(");
    css_push_px(&mut transform, translate_x);
    transform.push_str(", ");
    css_push_px(&mut transform, translate_y);
    transform.push_str(", 0px) scale(");
    css_push_number(&mut transform, inv_scale_x);
    transform.push_str(", ");
    css_push_number(&mut transform, inv_scale_y);
    transform.push(')');
    let _ = style.set_property("transform", &transform);
}

#[inline(never)]
pub(super) fn run_border_radius_correction(
    element: &Element,
    initial_scale_x: f64,
    initial_scale_y: f64,
) -> Option<BorderRadiusCorrectionAnimation> {
    if (initial_scale_x - 1.0).abs() <= FLIP_DELTA_EPSILON
        && (initial_scale_y - 1.0).abs() <= FLIP_DELTA_EPSILON
    {
        return None;
    }

    let target = read_border_radius_target(element)?;
    let inline_styles = Rc::new(capture_border_radius_inline_styles(element));
    let stop_signal = Rc::new(Cell::new(false));
    let fallback_inv_scale_x = safe_f64_ratio(1.0, initial_scale_x);
    let fallback_inv_scale_y = safe_f64_ratio(1.0, initial_scale_y);

    apply_border_radius_correction(element, target, fallback_inv_scale_x, fallback_inv_scale_y);
    schedule_border_radius_correction_frame(
        element.clone(),
        target,
        stop_signal.clone(),
        fallback_inv_scale_x,
        fallback_inv_scale_y,
    );

    Some(BorderRadiusCorrectionAnimation {
        stop_signal,
        inline_styles,
    })
}

#[inline(never)]
fn schedule_border_radius_correction_frame(
    element: Element,
    target: BorderRadiusTarget,
    stop_signal: Rc<Cell<bool>>,
    fallback_inv_scale_x: f64,
    fallback_inv_scale_y: f64,
) {
    request_animation_frame(move || {
        if stop_signal.get() {
            return;
        }

        let (inv_scale_x, inv_scale_y) = current_inverse_scale(&element)
            .map(|(scale_x, scale_y)| (safe_f64_ratio(1.0, scale_x), safe_f64_ratio(1.0, scale_y)))
            .unwrap_or((fallback_inv_scale_x, fallback_inv_scale_y));

        apply_border_radius_correction(&element, target, inv_scale_x, inv_scale_y);

        schedule_border_radius_correction_frame(
            element,
            target,
            stop_signal,
            fallback_inv_scale_x,
            fallback_inv_scale_y,
        );
    });
}

#[inline(never)]
fn apply_border_radius_correction(
    element: &Element,
    target: BorderRadiusTarget,
    inv_scale_x: f64,
    inv_scale_y: f64,
) {
    let Some(style) = html_style(element) else {
        return;
    };

    set_corner_radius(
        &style,
        "border-top-left-radius",
        target.top_left,
        inv_scale_x,
        inv_scale_y,
    );
    set_corner_radius(
        &style,
        "border-top-right-radius",
        target.top_right,
        inv_scale_x,
        inv_scale_y,
    );
    set_corner_radius(
        &style,
        "border-bottom-right-radius",
        target.bottom_right,
        inv_scale_x,
        inv_scale_y,
    );
    set_corner_radius(
        &style,
        "border-bottom-left-radius",
        target.bottom_left,
        inv_scale_x,
        inv_scale_y,
    );
}

#[inline(never)]
fn set_corner_radius(
    style: &CssStyleDeclaration,
    property: &str,
    base: RadiusPair,
    inv_scale_x: f64,
    inv_scale_y: f64,
) {
    let radius_x = base.x * inv_scale_x;
    let radius_y = base.y * inv_scale_y;

    let value = if (radius_x - radius_y).abs() <= FLIP_DELTA_EPSILON {
        let mut out = String::new();
        css_push_px(&mut out, radius_x);
        out
    } else {
        let mut out = String::new();
        css_push_px(&mut out, radius_x);
        out.push(' ');
        css_push_px(&mut out, radius_y);
        out
    };
    let _ = style.set_property(property, &value);
}

#[inline(never)]
fn read_border_radius_target(element: &Element) -> Option<BorderRadiusTarget> {
    let computed = computed_style(element)?;

    Some(BorderRadiusTarget {
        top_left: parse_radius_pair(&computed.get_property_value("border-top-left-radius").ok()?)?,
        top_right: parse_radius_pair(
            &computed
                .get_property_value("border-top-right-radius")
                .ok()?,
        )?,
        bottom_right: parse_radius_pair(
            &computed
                .get_property_value("border-bottom-right-radius")
                .ok()?,
        )?,
        bottom_left: parse_radius_pair(
            &computed
                .get_property_value("border-bottom-left-radius")
                .ok()?,
        )?,
    })
}

#[inline(never)]
fn parse_radius_pair(value: &str) -> Option<RadiusPair> {
    let mut parts = value.split_whitespace();
    let first = parse_px(parts.next()?)?;
    let second = parts.next().and_then(parse_px).unwrap_or(first);
    Some(RadiusPair {
        x: first,
        y: second,
    })
}

#[inline(never)]
fn parse_px(value: &str) -> Option<f64> {
    value
        .trim()
        .strip_suffix("px")
        .and_then(|raw| parse_js_f64(raw.trim()))
}

#[inline(never)]
fn current_inverse_scale(element: &Element) -> Option<(f64, f64)> {
    let computed = computed_style(element)?;
    let Ok(transform) = computed.get_property_value("transform") else {
        return None;
    };
    parse_transform_scale(&transform)
}

#[inline(never)]
fn parse_transform_scale(transform: &str) -> Option<(f64, f64)> {
    let transform = transform.trim();
    if transform.is_empty() || transform == "none" {
        return Some((1.0, 1.0));
    }

    if let Some(values) = transform
        .strip_prefix("matrix(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let mut matrix = [0.0; 6];
        let mut count = 0usize;
        for value in values.split(',') {
            if count >= matrix.len() {
                break;
            }
            matrix[count] = parse_js_f64(value.trim())?;
            count += 1;
        }
        if count == matrix.len() {
            return Some((matrix[0], matrix[3]));
        }
    }

    if let Some(values) = transform
        .strip_prefix("matrix3d(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let mut matrix = [0.0; 16];
        let mut count = 0usize;
        for value in values.split(',') {
            if count >= matrix.len() {
                break;
            }
            matrix[count] = parse_js_f64(value.trim())?;
            count += 1;
        }
        if count == matrix.len() {
            return Some((matrix[0], matrix[5]));
        }
    }

    None
}
