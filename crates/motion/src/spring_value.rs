use leptos::prelude::*;

use crate::Spring;
use crate::spring_solver::{SpringState, normalized_dt, step_spring};
use crate::timing::now_ms;

/// Spring-smoothed scalar value for continuously retargeted interactions.
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

        let now = now_ms();
        let last = last_time.get_value().unwrap_or(now);
        let dt = normalized_dt((now - last) / 1000.0);
        last_time.set_value(Some(now));

        let spring_cfg = spring.get_value();
        let target_value = target.get_untracked();
        let mut state = SpringState {
            value: value.get_untracked(),
            velocity: velocity.get_untracked(),
        };

        if step_spring(&mut state, target_value, spring_cfg, dt) {
            // Snap to target at rest to avoid tiny perpetual oscillations.
            value.set(target_value);
            velocity.set(0.0);
            running.set(false);
            last_time.set_value(None);
            return;
        }

        value.set(state.value);
        velocity.set(state.velocity);

        schedule_step(value, target, velocity, spring, running, last_time);
    });
}

pub fn use_spring(initial: f64, spring: Spring) -> SpringValue {
    SpringValue::new(initial, spring)
}

#[cfg(feature = "bench")]
pub fn spring_step(
    value: f64,
    velocity: f64,
    target: f64,
    spring_cfg: Spring,
    dt: f64,
) -> (f64, f64) {
    let mut state = SpringState { value, velocity };
    let _ = step_spring(&mut state, target, spring_cfg, normalized_dt(dt));
    (state.value, state.velocity)
}
