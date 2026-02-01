#[inline]
pub(crate) fn clamp_bounce(bounce: f64) -> f64 {
    bounce.clamp(0.0, 1.0)
}

#[inline]
pub(crate) fn duration_seconds(duration_ms: u32) -> f64 {
    (duration_ms as f64 / 1000.0).max(0.12)
}
