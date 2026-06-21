//! Pure start/end position resolution for scroll triggers.
//!
//! This module is fully host-testable: it takes simple value types (`Rect`) and
//! produces concrete pixel offsets without any DOM access. The browser layer
//! (Phase 3+) is responsible for turning an element into a `Rect` and a scroller
//! into a size, then calling [`resolve_start`] / [`resolve_end`].

/// One-dimensional layout rectangle: `start` is the top or left edge, `size` is
/// the extent along the scroll axis.
#[cfg_attr(test, derive(Debug))]
#[derive(Clone, Copy, PartialEq)]
pub struct Rect {
    /// Offset of the leading edge from the scroll origin (top/left).
    pub start: f64,
    /// Extent along the scroll axis (height for vertical, width for horizontal).
    pub size: f64,
}

/// A single anchor point on an element or scroller.
///
/// `Percent` is in `0.0..=1.0` of the element's own size, measured from the
/// top/left edge. `Pixels` is an absolute pixel offset from the top/left edge.
/// GSAP treats percentages and pixels as relative to the top/left of the
/// element/viewport, and this module follows that convention.
#[cfg_attr(test, derive(Debug))]
#[derive(Clone, Copy, PartialEq)]
pub enum ScrollPoint {
    Top,
    Bottom,
    Left,
    Right,
    Center,
    /// Fraction of the element size in `0.0..=1.0` measured from the leading edge.
    Percent(f64),
    /// Absolute pixel offset from the leading edge of the element/viewport.
    Pixels(f64),
}

/// Scroller-side anchor: either an absolute point on the viewport, or a
/// relative delta applied to the resolved start position (the `"+=N"` /
/// `"-=N"` form).
#[cfg_attr(test, derive(Debug))]
#[derive(Clone, Copy, PartialEq)]
pub enum ScrollOffset {
    Absolute(ScrollPoint),
    Relative {
        /// Raw pixel (or percent value when `percent_of_scroller` is true).
        pixels: f64,
        /// When true, `pixels` is interpreted as a percentage of the scroller
        /// size (`"+=80%"` -> `scroller_size * (80.0 / 100.0)`).
        percent_of_scroller: bool,
    },
}

/// One side of a start/end pair: a trigger point on the trigger element and a
/// scroller point on the scroller viewport. The trigger fires when the
/// scroller's scroll position aligns the two points.
#[cfg_attr(test, derive(Debug))]
#[derive(Clone, Copy, PartialEq)]
pub struct ScrollPosition {
    /// The anchor point on the trigger element (e.g. `Top`, `Bottom`, `Center`).
    pub trigger: ScrollPoint,
    /// The anchor point on the scroller viewport, or a `Relative` delta for `end`.
    pub scroller: ScrollOffset,
}

impl ScrollPoint {
    /// Resolves the point to an absolute pixel position along the scroll axis
    /// given the owning element's `Rect`.
    pub fn resolve(&self, rect: Rect) -> f64 {
        match self {
            ScrollPoint::Top | ScrollPoint::Left => rect.start,
            ScrollPoint::Bottom | ScrollPoint::Right => rect.start + rect.size,
            ScrollPoint::Center => rect.start + rect.size * 0.5,
            ScrollPoint::Percent(p) => rect.start + rect.size * p,
            ScrollPoint::Pixels(px) => rect.start + px,
        }
    }
}

impl ScrollOffset {
    /// Resolves the offset to a pixel position on the scroller viewport of the
    /// given size.
    pub fn resolve(&self, scroller_size: f64) -> f64 {
        match self {
            ScrollOffset::Absolute(point) => match point {
                ScrollPoint::Top | ScrollPoint::Left => 0.0,
                ScrollPoint::Bottom | ScrollPoint::Right => scroller_size,
                ScrollPoint::Center => scroller_size * 0.5,
                ScrollPoint::Percent(p) => scroller_size * p,
                ScrollPoint::Pixels(px) => *px,
            },
            ScrollOffset::Relative {
                pixels,
                percent_of_scroller,
            } => {
                if *percent_of_scroller {
                    scroller_size * (*pixels / 100.0)
                } else {
                    *pixels
                }
            }
        }
    }
}

/// Parses a single point token.
///
/// Keywords (`top`, `bottom`, `left`, `right`, `center`) are accepted as-is for
/// both axes; the caller is responsible for choosing axis-appropriate values.
/// `Percent` is `0.0..=1.0`; `Pixels` is an absolute offset. A bare numeric
/// string with no suffix is treated as pixels (GSAP allows this). `horizontal`
/// is currently unused but kept in the signature for forward compatibility with
/// axis-aware validation.
pub fn parse_point(s: &str, _horizontal: bool) -> Option<ScrollPoint> {
    let trimmed = s.trim();
    match trimmed.to_ascii_lowercase().as_str() {
        "top" => return Some(ScrollPoint::Top),
        "bottom" => return Some(ScrollPoint::Bottom),
        "left" => return Some(ScrollPoint::Left),
        "right" => return Some(ScrollPoint::Right),
        "center" => return Some(ScrollPoint::Center),
        _ => {}
    }

    if let Some(rest) = trimmed.strip_suffix('%') {
        let value: f64 = rest.trim().parse().ok()?;
        return Some(ScrollPoint::Percent(value / 100.0));
    }

    if let Some(rest) = trimmed.strip_suffix("px") {
        let value: f64 = rest.trim().parse().ok()?;
        return Some(ScrollPoint::Pixels(value));
    }

    if let Ok(value) = trimmed.parse::<f64>() {
        return Some(ScrollPoint::Pixels(value));
    }

    None
}

/// Parses a scroller-side token which may be `Absolute` (a point) or
/// `Relative` (the `"+=N"` / `"-=N"` form).
pub fn parse_offset(s: &str) -> Option<ScrollOffset> {
    let trimmed = s.trim();

    if let Some(rest) = trimmed.strip_prefix("+=") {
        return parse_relative_body(rest, 1.0);
    }
    if let Some(rest) = trimmed.strip_prefix("-=") {
        return parse_relative_body(rest, -1.0);
    }

    parse_point(trimmed, false).map(ScrollOffset::Absolute)
}

fn parse_relative_body(body: &str, sign: f64) -> Option<ScrollOffset> {
    let body = body.trim();
    if let Some(rest) = body.strip_suffix('%') {
        let value: f64 = rest.trim().parse().ok()?;
        return Some(ScrollOffset::Relative {
            pixels: sign * value,
            percent_of_scroller: true,
        });
    }
    if let Some(rest) = body.strip_suffix("px") {
        let value: f64 = rest.trim().parse().ok()?;
        return Some(ScrollOffset::Relative {
            pixels: sign * value,
            percent_of_scroller: false,
        });
    }
    let value: f64 = body.parse().ok()?;
    Some(ScrollOffset::Relative {
        pixels: sign * value,
        percent_of_scroller: false,
    })
}

/// Parses a full `"trigger scroller"` pair like `"top center"` or
/// `"bottom 80%"`.
///
/// A single-token string is interpreted as the trigger point with the scroller
/// defaulting to the same keyword (GSAP: `"top"` means `"top top"`). A lone
/// relative token (`"+=300"`) is only meaningful for `end` (relative to start);
/// for `start` it is an error because there is no anchor trigger point.
pub fn parse_position(s: &str, horizontal: bool) -> Option<ScrollPosition> {
    let trimmed = s.trim();
    let mut parts = trimmed.split_whitespace();

    let first = parts.next()?;
    let second = parts.next();

    if second.is_none() {
        if let Some(offset) = parse_offset(first) {
            if matches!(offset, ScrollOffset::Relative { .. }) {
                return None;
            }
        }
        let trigger = parse_point(first, horizontal)?;
        return Some(ScrollPosition {
            trigger,
            scroller: ScrollOffset::Absolute(trigger),
        });
    }

    let trigger = parse_point(first, horizontal)?;
    let scroller = parse_offset(second?)?;
    Some(ScrollPosition { trigger, scroller })
}

/// Parses a start/end pair where `end` may be a lone relative token. When `end`
/// is relative, its trigger point is ignored and only the `Relative` offset is
/// used by the caller (applied to the resolved start pixels). The returned
/// `ScrollPosition` for `end` still carries a `trigger`; the engine is
/// responsible for honoring the relative form.
pub fn parse_start_end(
    start: &str,
    end: &str,
    horizontal: bool,
) -> Option<(ScrollPosition, ScrollPosition)> {
    let start_pos = parse_position(start, horizontal)?;

    let end_trimmed = end.trim();
    if let Some(rest) = end_trimmed
        .strip_prefix("+=")
        .or_else(|| end_trimmed.strip_prefix("-="))
    {
        let _ = rest;
        if let Some(ScrollOffset::Relative {
            pixels,
            percent_of_scroller,
        }) = parse_offset(end_trimmed)
        {
            return Some((
                start_pos,
                ScrollPosition {
                    trigger: ScrollPoint::Top,
                    scroller: ScrollOffset::Relative {
                        pixels,
                        percent_of_scroller,
                    },
                },
            ));
        }
    }

    let end_pos = parse_position(end, horizontal)?;
    Some((start_pos, end_pos))
}

/// Clamps `value` to `[min, max]`.
pub fn clamp_value(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

/// If `s` is wrapped as `"clamp(...)"`, returns the inner content and `true`.
/// Otherwise returns `(s, false)`. Recursive clamp is not supported for MVP.
pub fn strip_clamp(s: &str) -> (&str, bool) {
    let trimmed = s.trim();
    if let Some(inner) = trimmed
        .strip_prefix("clamp(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return (inner.trim(), true);
    }
    (s, false)
}

/// Resolves the scroll position at which the trigger point meets the scroller
/// point. The core GSAP formula: `scroll = trigger_doc_pos - scroller_viewport_pos`.
pub fn resolve_start(trigger_rect: Rect, scroller_size: f64, position: &ScrollPosition) -> f64 {
    let trigger_point_pixels = position.trigger.resolve(trigger_rect);
    let scroller_point_pixels = position.scroller.resolve(scroller_size);
    trigger_point_pixels - scroller_point_pixels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_keywords() {
        assert_eq!(parse_point("top", false), Some(ScrollPoint::Top));
        assert_eq!(parse_point("BOTTOM", false), Some(ScrollPoint::Bottom));
        assert_eq!(parse_point("left", false), Some(ScrollPoint::Left));
        assert_eq!(parse_point("right", false), Some(ScrollPoint::Right));
        assert_eq!(parse_point("Center", false), Some(ScrollPoint::Center));
    }

    #[test]
    fn parses_percent() {
        assert_eq!(parse_point("80%", false), Some(ScrollPoint::Percent(0.8)));
        assert_eq!(parse_point("0%", false), Some(ScrollPoint::Percent(0.0)));
        assert_eq!(parse_point("100%", false), Some(ScrollPoint::Percent(1.0)));
    }

    #[test]
    fn parses_pixels() {
        assert_eq!(parse_point("100px", false), Some(ScrollPoint::Pixels(100.0)));
        assert_eq!(parse_point("100", false), Some(ScrollPoint::Pixels(100.0)));
        assert_eq!(
            parse_point("-50px", false),
            Some(ScrollPoint::Pixels(-50.0))
        );
    }

    #[test]
    fn rejects_garbage() {
        assert_eq!(parse_point("banana", false), None);
        assert_eq!(parse_point("", false), None);
        assert_eq!(parse_point("12px20", false), None);
    }

    #[test]
    fn parses_relative_pixels() {
        assert_eq!(
            parse_offset("+=300"),
            Some(ScrollOffset::Relative {
                pixels: 300.0,
                percent_of_scroller: false
            })
        );
        assert_eq!(
            parse_offset("-=300"),
            Some(ScrollOffset::Relative {
                pixels: -300.0,
                percent_of_scroller: false
            })
        );
        assert_eq!(
            parse_offset("+=300px"),
            Some(ScrollOffset::Relative {
                pixels: 300.0,
                percent_of_scroller: false
            })
        );
        assert_eq!(
            parse_offset("-=300px"),
            Some(ScrollOffset::Relative {
                pixels: -300.0,
                percent_of_scroller: false
            })
        );
    }

    #[test]
    fn parses_relative_percent() {
        assert_eq!(
            parse_offset("+=80%"),
            Some(ScrollOffset::Relative {
                pixels: 80.0,
                percent_of_scroller: true
            })
        );
        assert_eq!(
            parse_offset("-=20%"),
            Some(ScrollOffset::Relative {
                pixels: -20.0,
                percent_of_scroller: true
            })
        );
    }

    #[test]
    fn parses_absolute_offset() {
        assert_eq!(
            parse_offset("80%"),
            Some(ScrollOffset::Absolute(ScrollPoint::Percent(0.8)))
        );
        assert_eq!(
            parse_offset("top"),
            Some(ScrollOffset::Absolute(ScrollPoint::Top))
        );
    }

    #[test]
    fn strips_clamp_wrapper() {
        assert_eq!(strip_clamp("clamp(top center)"), ("top center", true));
        assert_eq!(strip_clamp("top center"), ("top center", false));
        assert_eq!(strip_clamp("clamp( top center )"), ("top center", true));
        assert_eq!(strip_clamp("not clamp("), ("not clamp(", false));
    }

    #[test]
    fn parses_position_top_center() {
        let pos = parse_position("top center", false).unwrap();
        assert_eq!(pos.trigger, ScrollPoint::Top);
        assert_eq!(pos.scroller, ScrollOffset::Absolute(ScrollPoint::Center));
    }

    #[test]
    fn parses_position_bottom_80percent() {
        let pos = parse_position("bottom 80%", false).unwrap();
        assert_eq!(pos.trigger, ScrollPoint::Bottom);
        assert_eq!(
            pos.scroller,
            ScrollOffset::Absolute(ScrollPoint::Percent(0.8))
        );
    }

    #[test]
    fn parses_single_token_defaults_scroller_to_same() {
        let pos = parse_position("top", false).unwrap();
        assert_eq!(pos.trigger, ScrollPoint::Top);
        assert_eq!(pos.scroller, ScrollOffset::Absolute(ScrollPoint::Top));
    }

    #[test]
    fn parses_single_bottom_token() {
        let pos = parse_position("bottom", false).unwrap();
        assert_eq!(pos.trigger, ScrollPoint::Bottom);
        assert_eq!(pos.scroller, ScrollOffset::Absolute(ScrollPoint::Bottom));
    }

    #[test]
    fn rejects_lone_relative_for_start() {
        assert_eq!(parse_position("+=300", false), None);
    }

    #[test]
    fn parses_start_end_with_relative_end() {
        let (start, end) = parse_start_end("top center", "+=300", false).unwrap();
        assert_eq!(start.trigger, ScrollPoint::Top);
        assert_eq!(start.scroller, ScrollOffset::Absolute(ScrollPoint::Center));
        assert_eq!(
            end.scroller,
            ScrollOffset::Relative {
                pixels: 300.0,
                percent_of_scroller: false
            }
        );
    }

    #[test]
    fn parses_start_end_with_absolute_end() {
        let (start, end) = parse_start_end("top center", "bottom top", false).unwrap();
        assert_eq!(start.trigger, ScrollPoint::Top);
        assert_eq!(end.trigger, ScrollPoint::Bottom);
        assert_eq!(end.scroller, ScrollOffset::Absolute(ScrollPoint::Top));
    }

    #[test]
    fn clamp_value_clamps() {
        assert_eq!(clamp_value(5.0, 0.0, 10.0), 5.0);
        assert_eq!(clamp_value(-1.0, 0.0, 10.0), 0.0);
        assert_eq!(clamp_value(15.0, 0.0, 10.0), 10.0);
    }

    #[test]
    fn resolves_start_top_vs_top() {
        let rect = Rect { start: 1000.0, size: 200.0 };
        let pos = ScrollPosition {
            trigger: ScrollPoint::Top,
            scroller: ScrollOffset::Absolute(ScrollPoint::Top),
        };
        assert_eq!(resolve_start(rect, 800.0, &pos), 1000.0);
    }

    #[test]
    fn resolves_start_bottom_vs_center() {
        let rect = Rect { start: 1000.0, size: 200.0 };
        let pos = ScrollPosition {
            trigger: ScrollPoint::Bottom,
            scroller: ScrollOffset::Absolute(ScrollPoint::Center),
        };
        assert_eq!(resolve_start(rect, 800.0, &pos), 1200.0 - 400.0);
    }

    #[test]
    fn resolves_start_center_vs_80percent() {
        let rect = Rect { start: 1000.0, size: 200.0 };
        let pos = ScrollPosition {
            trigger: ScrollPoint::Center,
            scroller: ScrollOffset::Absolute(ScrollPoint::Percent(0.8)),
        };
        assert_eq!(resolve_start(rect, 800.0, &pos), 1100.0 - 640.0);
    }

    #[test]
    fn resolves_start_percent_trigger() {
        let rect = Rect { start: 1000.0, size: 200.0 };
        let pos = ScrollPosition {
            trigger: ScrollPoint::Percent(0.25),
            scroller: ScrollOffset::Absolute(ScrollPoint::Top),
        };
        assert_eq!(resolve_start(rect, 800.0, &pos), 1050.0);
    }

    #[test]
    fn resolves_start_pixels_trigger() {
        let rect = Rect { start: 1000.0, size: 200.0 };
        let pos = ScrollPosition {
            trigger: ScrollPoint::Pixels(50.0),
            scroller: ScrollOffset::Absolute(ScrollPoint::Top),
        };
        assert_eq!(resolve_start(rect, 800.0, &pos), 1050.0);
    }

    #[test]
    fn resolves_start_relative_end_pixels() {
        let rect = Rect { start: 1000.0, size: 200.0 };
        let pos = ScrollPosition {
            trigger: ScrollPoint::Top,
            scroller: ScrollOffset::Relative {
                pixels: 300.0,
                percent_of_scroller: false,
            },
        };
        assert_eq!(resolve_start(rect, 800.0, &pos), 1000.0 - 300.0);
    }

    #[test]
    fn resolves_start_relative_end_percent_of_scroller() {
        let rect = Rect { start: 1000.0, size: 200.0 };
        let pos = ScrollPosition {
            trigger: ScrollPoint::Top,
            scroller: ScrollOffset::Relative {
                pixels: 80.0,
                percent_of_scroller: true,
            },
        };
        assert_eq!(resolve_start(rect, 800.0, &pos), 1000.0 - (800.0 * 0.8));
    }
}