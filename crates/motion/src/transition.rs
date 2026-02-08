use std::borrow::Cow;

use crate::spring_math::{clamp_bounce, duration_seconds};
const EASE_OUT_CUBIC: &str = "cubic-bezier(0.215, 0.61, 0.355, 1)";
const EASE_IN_OUT_CUBIC: &str = "cubic-bezier(0.645, 0.045, 0.355, 1)";
const SPRING_EASING: &str = EASE_OUT_CUBIC;
const DEFAULT_DURATION_MS: u32 = 200;
const SNAPPY_DURATION_MS: u32 = 150;
const SPRING_DURATION_MS: u32 = 500;
const SPRING_BOUNCE: f64 = 0.2;

#[derive(Clone, Debug, PartialEq)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Spring,
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
            Easing::Spring => SPRING_EASING,
            Easing::Custom(value) => value.as_ref(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    pub duration_ms: u32,
    pub bounce: f64,
    pub rest_delta: f64,
}

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

#[derive(Clone, Debug, PartialEq)]
pub struct Transition {
    pub duration_ms: u32,
    pub delay_ms: u32,
    pub easing: Easing,
    pub spring: Option<Spring>,
    pub excluded_properties: Vec<Cow<'static, str>>,
}

impl Default for Transition {
    fn default() -> Self {
        Self {
            duration_ms: DEFAULT_DURATION_MS,
            delay_ms: 0,
            easing: Easing::EaseOut,
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
        if let Some(spring) = &mut self.spring {
            spring.duration_ms = value;
        }
        self
    }

    pub fn delay_ms(mut self, value: u32) -> Self {
        self.delay_ms = value;
        self
    }

    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self.spring = None;
        self
    }

    pub fn spring() -> Self {
        Self::spring_with(SPRING_DURATION_MS, SPRING_BOUNCE)
    }

    pub fn spring_with(duration_ms: u32, bounce: f64) -> Self {
        let spring = Spring::new(duration_ms, bounce);
        Self {
            duration_ms,
            delay_ms: 0,
            easing: Easing::Spring,
            spring: Some(spring),
            excluded_properties: Vec::new(),
        }
    }

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
            spring: None,
            excluded_properties: Vec::new(),
        }
    }

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

    pub fn easing_string(&self) -> Cow<'_, str> {
        if let Some(spring) = self.spring {
            Cow::Owned(spring_easing(spring.duration_ms, spring.bounce))
        } else {
            Cow::Borrowed(self.easing.as_str())
        }
    }

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

fn spring_easing(duration_ms: u32, bounce: f64) -> String {
    let bounce = clamp_bounce(bounce);
    if bounce <= 0.0 {
        return SPRING_EASING.to_string();
    }

    let duration = duration_seconds(duration_ms);
    let zeta = (1.0 - 0.85 * bounce).clamp(0.05, 0.98);
    let omega_n = 4.0 / duration;
    let omega_d = omega_n * (1.0 - zeta * zeta).sqrt();
    let coeff = zeta / (1.0 - zeta * zeta).sqrt();

    let mut out = String::from("linear(");
    for index in 0..=7 {
        let t = index as f64 / 7.0;
        let value = if index == 0 {
            0.0
        } else if index == 7 {
            1.0
        } else {
            let expo = (-zeta * omega_n * t).exp();
            let value = 1.0 - expo * ((omega_d * t).cos() + coeff * (omega_d * t).sin());
            value.clamp(-0.1, 1.35)
        };

        if index > 0 {
            out.push_str(", ");
        }
        push_rounded_number(&mut out, value, 3);
        if index > 0 && index < 7 {
            out.push(' ');
            push_rounded_number(&mut out, t * 100.0, 1);
            out.push('%');
        }
    }
    out.push(')');
    out
}

fn push_rounded_number(out: &mut String, value: f64, decimals: u8) {
    let scale = match decimals {
        1 => 10_i64,
        2 => 100_i64,
        3 => 1000_i64,
        _ => 1_i64,
    };

    let mut scaled = (value * scale as f64).round();
    if !scaled.is_finite() {
        out.push('0');
        return;
    }
    if scaled == 0.0 {
        out.push('0');
        return;
    }

    let negative = scaled < 0.0;
    if negative {
        scaled = -scaled;
        out.push('-');
    }

    let scaled = scaled as u64;
    let int_part = scaled / scale as u64;
    let mut frac_part = scaled % scale as u64;
    out.push_str(&int_part.to_string());

    if decimals == 0 || frac_part == 0 {
        return;
    }

    let mut digits = [0_u8; 3];
    let mut index = decimals as usize;
    while index > 0 {
        index -= 1;
        digits[index] = (frac_part % 10) as u8;
        frac_part /= 10;
    }

    let mut end = decimals as usize;
    while end > 0 && digits[end - 1] == 0 {
        end -= 1;
    }
    if end == 0 {
        return;
    }

    out.push('.');
    for digit in &digits[..end] {
        out.push((b'0' + *digit) as char);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spring_transition_defaults() {
        let spring = Transition::spring();
        assert_eq!(spring.duration_ms, 500);
        assert_eq!(spring.delay_ms, 0);
        assert!(spring.spring.is_some());
        assert_eq!(spring.spring.unwrap().bounce, 0.2);
        let easing = spring.easing_string();
        assert!(easing.starts_with("linear("));
    }

    #[test]
    fn bounce_is_clamped() {
        let spring = Transition::spring_with(400, 1.8);
        assert_eq!(spring.spring.unwrap().bounce, 1.0);
    }

    #[test]
    fn zero_bounce_is_ease_out() {
        let spring = Transition::spring_with(400, 0.0);
        let easing = spring.easing_string();
        assert_eq!(easing.as_ref(), SPRING_EASING);
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
