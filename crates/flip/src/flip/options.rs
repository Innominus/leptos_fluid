use super::{EASE_IN_OUT, LINEAR};

const DEFAULT_DURATION_MS: usize = 240;

/// Runtime options for both `Flip` and `FlipGroup`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlipOptions {
    /// Animation duration in milliseconds.
    pub duration: usize,
    /// Initial delay in milliseconds.
    pub delay: usize,
    /// Per-item delay increment in milliseconds (group mode).
    pub stagger: usize,
    /// Easing curve used by WAAPI.
    pub easing: Easing,
    /// Whether to animate only position or position+size.
    pub scale_mode: ScaleMode,
    /// Optional descendant selector used for inverse-scale correction.
    pub scale_correction_selector: Option<&'static str>,
}

impl Default for FlipOptions {
    fn default() -> Self {
        Self {
            duration: DEFAULT_DURATION_MS,
            delay: 0,
            stagger: 0,
            easing: Easing::EaseInOut,
            scale_mode: ScaleMode::PositionOnly,
            scale_correction_selector: None,
        }
    }
}

impl FlipOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn duration_ms(mut self, duration: usize) -> Self {
        self.duration = duration;
        self
    }

    pub fn delay_ms(mut self, delay: usize) -> Self {
        self.delay = delay;
        self
    }

    pub fn stagger_ms(mut self, stagger: usize) -> Self {
        self.stagger = stagger;
        self
    }

    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    pub fn scale_mode(mut self, scale_mode: ScaleMode) -> Self {
        self.scale_mode = scale_mode;
        self
    }

    pub fn scale_correction_selector(mut self, selector: &'static str) -> Self {
        self.scale_correction_selector = Some(selector);
        self
    }

    pub(crate) fn with_stagger_index(self, index: usize) -> Self {
        let stagger_delay = self.stagger.saturating_mul(index);
        Self {
            delay: self.delay.saturating_add(stagger_delay),
            ..self
        }
    }
}

/// Easing presets for FLIP animations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Easing {
    /// Cubic-bezier ease-in-out curve.
    #[default]
    EaseInOut,
    /// Smooth custom `linear(...)` curve tuned for FLIP movement.
    Linear,
    /// Caller-provided CSS easing string.
    Custom(&'static str),
}

impl Easing {
    pub(crate) fn get_easing_fn(&self) -> &'static str {
        match self {
            Easing::Linear => LINEAR,
            Easing::EaseInOut => EASE_IN_OUT,
            Easing::Custom(val) => val,
        }
    }
}

/// Controls whether size deltas participate in FLIP inversion.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScaleMode {
    /// Animate position changes only.
    #[default]
    PositionOnly,
    /// Animate both position and size via scale transforms.
    PositionAndScale,
}

impl ScaleMode {
    pub(crate) fn uses_scale(&self) -> bool {
        matches!(self, ScaleMode::PositionAndScale)
    }
}
