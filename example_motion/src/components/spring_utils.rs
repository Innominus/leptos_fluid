pub fn lerp(from: f64, to: f64, progress: f64) -> f64 {
    let progress = progress.clamp(0.0, 1.0);
    from + (to - from) * progress
}
