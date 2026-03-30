use crate::Spring;
use crate::spring_math::{clamp_bounce, duration_seconds};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct SpringState {
    pub value: f64,
    pub velocity: f64,
}

#[inline]
pub(crate) fn normalized_dt(raw_seconds: f64) -> f64 {
    if raw_seconds <= 0.0 {
        return 0.016;
    }

    raw_seconds.min(0.05)
}

#[inline]
pub(crate) fn spring_constants(duration_ms: u32, bounce: f64) -> (f64, f64) {
    let duration = duration_seconds(duration_ms);
    let damping_ratio = (1.0 - clamp_bounce(bounce)).clamp(0.05, 1.0);
    let angular = 4.0 / (duration * damping_ratio);
    let stiffness = angular * angular;
    let damping = 2.0 * damping_ratio * angular;
    (stiffness, damping)
}

#[inline]
pub(crate) fn step_spring(state: &mut SpringState, target: f64, spring: Spring, dt: f64) -> bool {
    let (stiffness, damping) = spring_constants(spring.duration_ms, spring.bounce);
    let acceleration = -stiffness * (state.value - target) - damping * state.velocity;
    state.velocity += acceleration * dt;
    state.value += state.velocity * dt;

    if (state.value - target).abs() < spring.rest_delta && state.velocity.abs() < spring.rest_delta
    {
        state.value = target;
        state.velocity = 0.0;
        return true;
    }

    false
}
