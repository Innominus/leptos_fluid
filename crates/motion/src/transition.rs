use std::borrow::Cow;

#[cfg(feature = "spring")]
use crate::spring_math::clamp_bounce;
const EASE_OUT_CUBIC: &str = "cubic-bezier(0.215, 0.61, 0.355, 1)";
const EASE_IN_OUT_CUBIC: &str = "cubic-bezier(0.645, 0.045, 0.355, 1)";
const DEFAULT_DURATION_MS: u32 = 200;
const SNAPPY_DURATION_MS: u32 = 150;
#[cfg(feature = "spring")]
const SPRING_DURATION_MS: u32 = 500;
#[cfg(feature = "spring")]
const SPRING_BOUNCE: f64 = 0.2;

/// Easing presets used by `Transition`.
#[derive(Clone, Debug, PartialEq)]
pub enum Easing {
    /// Linear easing.
    Linear,
    /// CSS `ease-in`.
    EaseIn,
    /// Cubic-bezier ease-out tuned for UI movement.
    EaseOut,
    /// Cubic-bezier ease-in-out.
    EaseInOut,
    /// Caller-provided CSS easing string.
    Custom(Cow<'static, str>),
}

impl Easing {
    pub fn custom(value: impl Into<Cow<'static, str>>) -> Self {
        Easing::Custom(value.into())
    }

    pub fn as_str(&self) -> &str {
        match self {
            Easing::Linear => "linear",
            Easing::EaseIn => "ease-in",
            Easing::EaseOut => EASE_OUT_CUBIC,
            Easing::EaseInOut => EASE_IN_OUT_CUBIC,
            Easing::Custom(value) => value.as_ref(),
        }
    }
}

/// Spring configuration shared by transitions and spring values.
#[cfg(feature = "spring")]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    /// Intended settling duration in milliseconds.
    pub duration_ms: u32,
    /// Bounce amount in the inclusive range `[0.0, 1.0]`.
    pub bounce: f64,
    /// Solver threshold for ending motion.
    pub rest_delta: f64,
}

#[cfg(feature = "spring")]
impl Spring {
    pub fn new(duration_ms: u32, bounce: f64) -> Self {
        Self {
            duration_ms,
            bounce: clamp_bounce(bounce),
            rest_delta: 0.001,
        }
    }

    pub fn duration_ms(mut self, value: u32) -> Self {
        self.duration_ms = value;
        self
    }

    pub fn bounce(mut self, value: f64) -> Self {
        self.bounce = clamp_bounce(value);
        self
    }

    pub fn rest_delta(mut self, value: f64) -> Self {
        self.rest_delta = value.max(0.000_001);
        self
    }
}

/// Transition configuration for motion updates.
#[derive(Clone, Debug, PartialEq)]
pub struct Transition {
    /// Animation duration in milliseconds.
    pub duration_ms: u32,
    /// Animation delay in milliseconds.
    pub delay_ms: u32,
    /// Active easing preset.
    pub easing: Easing,
    /// Optional live spring configuration used by `Transition::spring*`.
    #[cfg(feature = "spring")]
    pub spring: Option<Spring>,
    /// Properties that should not animate (applied immediately).
    pub excluded_properties: Vec<Cow<'static, str>>,
}

impl Default for Transition {
    fn default() -> Self {
        Self {
            duration_ms: DEFAULT_DURATION_MS,
            delay_ms: 0,
            easing: Easing::EaseOut,
            #[cfg(feature = "spring")]
            spring: None,
            excluded_properties: Vec::new(),
        }
    }
}

impl Transition {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn duration_ms(mut self, value: u32) -> Self {
        self.duration_ms = value;
        self.sync_spring_duration(value);
        self
    }

    pub fn delay_ms(mut self, value: u32) -> Self {
        self.delay_ms = value;
        self
    }

    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self.clear_spring();
        self
    }

    /// Creates a live rAF spring transition.
    ///
    /// Unsupported properties are applied immediately and do not interpolate in
    /// spring mode. Use tween transitions for colors, filters, shadows, and
    /// other text-valued CSS properties.
    #[cfg(feature = "spring")]
    pub fn spring() -> Self {
        Self::spring_with(SPRING_DURATION_MS, SPRING_BOUNCE)
    }

    /// Creates a live rAF spring transition with a custom duration and bounce.
    ///
    /// Unsupported properties are applied immediately and do not interpolate in
    /// spring mode. Use tween transitions for colors, filters, shadows, and
    /// other text-valued CSS properties.
    #[cfg(feature = "spring")]
    pub fn spring_with(duration_ms: u32, bounce: f64) -> Self {
        let spring = Spring::new(duration_ms, bounce);
        Self {
            duration_ms,
            delay_ms: 0,
            easing: Easing::EaseOut,
            spring: Some(spring),
            excluded_properties: Vec::new(),
        }
    }

    /// Adjusts the bounce amount of the live spring transition.
    ///
    /// Unsupported properties are applied immediately and do not interpolate in
    /// spring mode. Use tween transitions for colors, filters, shadows, and
    /// other text-valued CSS properties.
    #[cfg(feature = "spring")]
    pub fn bounce(mut self, bounce: f64) -> Self {
        let value = clamp_bounce(bounce);
        match &mut self.spring {
            Some(spring) => spring.bounce = value,
            None => self.spring = Some(Spring::new(self.duration_ms, value)),
        }
        self
    }

    pub fn snappy() -> Self {
        Self {
            duration_ms: SNAPPY_DURATION_MS,
            delay_ms: 0,
            easing: Easing::EaseOut,
            #[cfg(feature = "spring")]
            spring: None,
            excluded_properties: Vec::new(),
        }
    }

    /// Excludes properties from animation so they apply immediately.
    ///
    /// When using a spring transition, unsupported properties are already
    /// applied immediately and do not interpolate in spring mode.
    pub fn exclude_properties(mut self, props: &[&'static str]) -> Self {
        let mut list = Vec::new();
        for prop in props {
            push_unique_property(&mut list, Cow::Borrowed(prop));
        }
        self.excluded_properties = list;
        self
    }

    pub fn add_excluded_properties(mut self, props: &[&'static str]) -> Self {
        for prop in props {
            push_unique_property(&mut self.excluded_properties, Cow::Borrowed(prop));
        }
        self
    }

    pub fn without_properties(self, props: &[&'static str]) -> Self {
        self.exclude_properties(props)
    }

    #[cfg(feature = "spring")]
    pub(crate) fn spring_config(&self) -> Option<Spring> {
        self.spring
    }

    pub fn easing_string(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.easing.as_str())
    }

    #[cfg(feature = "spring")]
    fn sync_spring_duration(&mut self, value: u32) {
        if let Some(spring) = &mut self.spring {
            spring.duration_ms = value;
        }
    }

    #[cfg(not(feature = "spring"))]
    fn sync_spring_duration(&mut self, _value: u32) {}

    #[cfg(feature = "spring")]
    fn clear_spring(&mut self) {
        self.spring = None;
    }

    #[cfg(not(feature = "spring"))]
    fn clear_spring(&mut self) {}

    #[cfg(any(test, feature = "bench"))]
    pub(crate) fn transition_css(&self) -> String {
        if self.duration_ms == 0 && self.delay_ms == 0 {
            return "none".to_string();
        }

        let easing = self.easing_string();
        let mut out = String::from("all ");
        out.push_str(&self.duration_ms.to_string());
        out.push_str("ms ");
        out.push_str(easing.as_ref());
        out.push(' ');
        out.push_str(&self.delay_ms.to_string());
        out.push_str("ms");

        for prop in &self.excluded_properties {
            if prop.is_empty() {
                continue;
            }
            out.push_str(", ");
            out.push_str(prop.as_ref());
            out.push_str(" 0ms linear 0ms");
        }

        out
    }

    #[cfg(feature = "bench")]
    pub fn transition_css_public(&self) -> String {
        self.transition_css()
    }
}

fn push_unique_property(list: &mut Vec<Cow<'static, str>>, prop: Cow<'static, str>) {
    if list
        .iter()
        .any(|existing| existing.as_ref() == prop.as_ref())
    {
        return;
    }
    list.push(prop);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "spring")]
    #[test]
    fn spring_transition_defaults() {
        let spring = Transition::spring();
        assert_eq!(spring.duration_ms, 500);
        assert_eq!(spring.delay_ms, 0);
        assert!(spring.spring.is_some());
        assert_eq!(spring.spring.unwrap().bounce, 0.2);
        assert_eq!(spring.easing.as_str(), EASE_OUT_CUBIC);
    }

    #[cfg(feature = "spring")]
    #[test]
    fn bounce_is_clamped() {
        let spring = Transition::spring_with(400, 1.8);
        assert_eq!(spring.spring.unwrap().bounce, 1.0);
    }

    #[test]
    fn default_transition_is_all() {
        let transition = Transition::new().duration_ms(300).delay_ms(50);
        let css = transition.transition_css();
        assert!(css.starts_with("all 300ms"));
        assert!(css.ends_with("50ms"));
    }

    #[test]
    fn excluded_properties_disable_specific_animation() {
        let transition = Transition::new()
            .duration_ms(300)
            .exclude_properties(&["height", "width"]);
        let css = transition.transition_css();
        assert!(css.contains("all 300ms"));
        assert!(css.contains(", height 0ms linear 0ms"));
        assert!(css.contains(", width 0ms linear 0ms"));
    }
}
