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
        // Keep correction transforms synced to root scale every frame until stop.
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
            // Stopped by the parent FLIP cleanup path.
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
    from_target: Option<BorderRadiusTarget>,
    to_target: Option<BorderRadiusTarget>,
    initial_scale_x: f64,
    initial_scale_y: f64,
) -> Option<BorderRadiusCorrectionAnimation> {
    if (initial_scale_x - 1.0).abs() <= FLIP_DELTA_EPSILON
        && (initial_scale_y - 1.0).abs() <= FLIP_DELTA_EPSILON
    {
        return None;
    }

    let target = to_target.or_else(|| read_border_radius_target(element))?;
    let source = from_target.unwrap_or(target);
    let inline_styles = Rc::new(capture_border_radius_inline_styles(element));
    let stop_signal = Rc::new(Cell::new(false));
    let fallback_inv_scale_x = safe_f64_ratio(1.0, initial_scale_x);
    let fallback_inv_scale_y = safe_f64_ratio(1.0, initial_scale_y);

    apply_border_radius_correction(element, source, fallback_inv_scale_x, fallback_inv_scale_y);
    // Border radius must be corrected continuously while scale is interpolating.
    schedule_border_radius_correction_frame(
        element.clone(),
        source,
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
    source_target: BorderRadiusTarget,
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

        let progress_x = progress_from_inverse_scale(inv_scale_x, fallback_inv_scale_x);
        let progress_y = progress_from_inverse_scale(inv_scale_y, fallback_inv_scale_y);
        let current_target =
            interpolate_border_radius_target(source_target, target, progress_x, progress_y);

        apply_border_radius_correction(&element, current_target, inv_scale_x, inv_scale_y);

        schedule_border_radius_correction_frame(
            element,
            source_target,
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
pub(super) fn read_border_radius_target(element: &Element) -> Option<BorderRadiusTarget> {
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
fn progress_from_inverse_scale(current_inv_scale: f64, initial_inv_scale: f64) -> f64 {
    let denominator = 1.0 - initial_inv_scale;
    if denominator.abs() <= f64::EPSILON {
        return 1.0;
    }

    let progress = (current_inv_scale - initial_inv_scale) / denominator;
    if progress.is_finite() {
        progress.clamp(0.0, 1.0)
    } else {
        1.0
    }
}

#[inline(never)]
fn interpolate_border_radius_target(
    from: BorderRadiusTarget,
    to: BorderRadiusTarget,
    progress_x: f64,
    progress_y: f64,
) -> BorderRadiusTarget {
    BorderRadiusTarget {
        top_left: interpolate_radius_pair(from.top_left, to.top_left, progress_x, progress_y),
        top_right: interpolate_radius_pair(from.top_right, to.top_right, progress_x, progress_y),
        bottom_right: interpolate_radius_pair(
            from.bottom_right,
            to.bottom_right,
            progress_x,
            progress_y,
        ),
        bottom_left: interpolate_radius_pair(
            from.bottom_left,
            to.bottom_left,
            progress_x,
            progress_y,
        ),
    }
}

#[inline(never)]
fn interpolate_radius_pair(
    from: RadiusPair,
    to: RadiusPair,
    progress_x: f64,
    progress_y: f64,
) -> RadiusPair {
    RadiusPair {
        x: lerp(from.x, to.x, progress_x),
        y: lerp(from.y, to.y, progress_y),
    }
}

#[inline(never)]
fn lerp(from: f64, to: f64, progress: f64) -> f64 {
    from + (to - from) * progress
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
        // matrix(a, b, c, d, tx, ty) => scaleX = a, scaleY = d
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
        // matrix3d uses m11 and m22 for scaleX/scaleY respectively.
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
