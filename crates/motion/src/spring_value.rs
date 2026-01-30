use leptos::prelude::*;

use crate::Spring;

use js_sys::Date;

#[derive(Clone)]
pub struct SpringValue {
    value: RwSignal<f64>,
    target: RwSignal<f64>,
    velocity: RwSignal<f64>,
    spring: StoredValue<Spring>,
    running: RwSignal<bool>,
    last_time: StoredValue<Option<f64>>,
}

impl SpringValue {
    pub fn new(initial: f64, spring: Spring) -> Self {
        Self {
            value: RwSignal::new(initial),
            target: RwSignal::new(initial),
            velocity: RwSignal::new(0.0),
            spring: StoredValue::new(spring),
            running: RwSignal::new(false),
            last_time: StoredValue::new(None),
        }
    }

    pub fn signal(&self) -> Signal<f64> {
        self.value.into()
    }

    pub fn get(&self) -> f64 {
        self.value.get()
    }

    pub fn set(&self, target: f64) {
        self.target.set(target);
        self.start();
    }

    pub fn set_immediate(&self, value: f64) {
        self.value.set(value);
        self.target.set(value);
        self.velocity.set(0.0);
        self.running.set(false);
        self.last_time.set_value(None);
    }

    fn start(&self) {
        if self.running.get_untracked() {
            return;
        }
        self.running.set(true);
        self.last_time.set_value(None);
        schedule_step(
            self.value,
            self.target,
            self.velocity,
            self.spring,
            self.running,
            self.last_time,
        );
    }
}

fn schedule_step(
    value: RwSignal<f64>,
    target: RwSignal<f64>,
    velocity: RwSignal<f64>,
    spring: StoredValue<Spring>,
    running: RwSignal<bool>,
    last_time: StoredValue<Option<f64>>,
) {
    request_animation_frame(move || {
        if !running.get_untracked() {
            return;
        }

        let now = Date::now();
        let last = last_time.get_value().unwrap_or(now);
        let mut dt = (now - last) / 1000.0;
        if dt <= 0.0 {
            dt = 0.016;
        }
        if dt > 0.05 {
            dt = 0.05;
        }
        last_time.set_value(Some(now));

        let spring_cfg = spring.get_value();
        let target_value = target.get_untracked();
        let mut current_value = value.get_untracked();
        let mut current_velocity = velocity.get_untracked();

        let (stiffness, damping) = spring_constants(spring_cfg.duration_ms, spring_cfg.bounce);
        let acceleration = -stiffness * (current_value - target_value) - damping * current_velocity;
        current_velocity += acceleration * dt;
        current_value += current_velocity * dt;

        if (current_value - target_value).abs() < spring_cfg.rest_delta
            && current_velocity.abs() < spring_cfg.rest_delta
        {
            value.set(target_value);
            velocity.set(0.0);
            running.set(false);
            last_time.set_value(None);
            return;
        }

        value.set(current_value);
        velocity.set(current_velocity);

        schedule_step(value, target, velocity, spring, running, last_time);
    });
}

fn spring_constants(duration_ms: u32, bounce: f64) -> (f64, f64) {
    let duration = (duration_ms as f64 / 1000.0).max(0.12);
    let damping_ratio = (1.0 - bounce).clamp(0.05, 1.0);
    let angular = 4.0 / (duration * damping_ratio);
    let stiffness = angular * angular;
    let damping = 2.0 * damping_ratio * angular;
    (stiffness, damping)
}

pub fn use_spring(initial: f64, spring: Spring) -> SpringValue {
    SpringValue::new(initial, spring)
}
