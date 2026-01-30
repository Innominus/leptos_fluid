use std::borrow::Cow;

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
            bounce: bounce.clamp(0.0, 1.0),
            rest_delta: 0.001,
        }
    }

    pub fn duration_ms(mut self, value: u32) -> Self {
        self.duration_ms = value;
        self
    }

    pub fn bounce(mut self, value: f64) -> Self {
        self.bounce = value.clamp(0.0, 1.0);
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
}

impl Default for Transition {
    fn default() -> Self {
        Self {
            duration_ms: DEFAULT_DURATION_MS,
            delay_ms: 0,
            easing: Easing::EaseOut,
            spring: None,
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
        }
    }

    pub fn bounce(mut self, bounce: f64) -> Self {
        let value = bounce.clamp(0.0, 1.0);
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
        }
    }

    pub fn easing_string(&self) -> Cow<'_, str> {
        if let Some(spring) = self.spring {
            Cow::Owned(spring_easing(spring.duration_ms, spring.bounce))
        } else {
            Cow::Borrowed(self.easing.as_str())
        }
    }
}

fn spring_easing(duration_ms: u32, bounce: f64) -> String {
    let bounce = bounce.clamp(0.0, 1.0);
    if bounce <= 0.0 {
        return SPRING_EASING.to_string();
    }

    let duration = (duration_ms as f64 / 1000.0).max(0.12);
    let zeta = (1.0 - 0.85 * bounce).clamp(0.05, 0.98);
    let omega_n = 4.0 / duration;
    let omega_d = omega_n * (1.0 - zeta * zeta).sqrt();
    let coeff = zeta / (1.0 - zeta * zeta).sqrt();

    let mut points = Vec::with_capacity(8);
    for i in 0..=7 {
        let t = i as f64 / 7.0;
        let expo = (-zeta * omega_n * t).exp();
        let value = 1.0 - expo * ((omega_d * t).cos() + coeff * (omega_d * t).sin());
        points.push((t, value.clamp(-0.1, 1.35)));
    }

    if let Some(first) = points.first_mut() {
        first.1 = 0.0;
    }
    if let Some(last) = points.last_mut() {
        last.1 = 1.0;
    }

    let mut out = String::from("linear(");
    for (index, (t, value)) in points.iter().enumerate() {
        if index == 0 {
            out.push_str(&format!("{:.3}", value));
        } else if index + 1 == points.len() {
            out.push_str(&format!(", {:.3}", value));
        } else {
            out.push_str(&format!(", {:.3} {:.1}%", value, t * 100.0));
        }
    }
    out.push(')');
    out
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
}
