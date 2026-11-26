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
    frame: StoredValue<Option<AnimationFrameRequestHandle>>,
}

impl SpringValue {
    pub fn new(initial: f64, spring: Spring) -> Self {
        let frame = StoredValue::new(None);
        // Re-requested frames run outside any owner, so cancel the pending one with this value's owner.
        on_cleanup(move || cancel_frame(frame));
        Self {
            value: RwSignal::new(initial),
            target: RwSignal::new(initial),
            velocity: RwSignal::new(0.0),
            spring: StoredValue::new(spring),
            running: RwSignal::new(false),
            last_time: StoredValue::new(None),
            frame,
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
        // Cancel the pending step so a following `set` can't start a second frame chain.
        cancel_frame(self.frame);
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
            self.frame,
        );
    }
}

fn cancel_frame(frame: StoredValue<Option<AnimationFrameRequestHandle>>) {
    if let Some(handle) = frame.try_update_value(Option::take).flatten() {
        handle.cancel();
    }
}

fn schedule_step(
    value: RwSignal<f64>,
    target: RwSignal<f64>,
    velocity: RwSignal<f64>,
    spring: StoredValue<Spring>,
    running: RwSignal<bool>,
    last_time: StoredValue<Option<f64>>,
    frame: StoredValue<Option<AnimationFrameRequestHandle>>,
) {
    let scheduled = request_animation_frame(move || {
        if running.try_get_untracked() != Some(true) {
            return;
        }
        frame.set_value(None);

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

        schedule_step(value, target, velocity, spring, running, last_time, frame);
    });
    match scheduled {
        Ok(handle) => {
            frame.set_value(Some(handle));
            if Owner::current().is_some() {
                // Leptos cancels an owned frame when its owner cleans up (e.g. an effect rerun);
                // release `running` then so the next `set` restarts instead of wedging.
                on_cleanup(move || {
                    if frame.try_get_value().flatten() == Some(handle) {
                        frame.set_value(None);
                        running.try_set(false);
                    }
                });
            }
        }
        Err(_) => {
            // Scheduling failed: settle at the target so `running` can't stay
            // stuck true and swallow every later `set`.
            value.set(target.get_untracked());
            velocity.set(0.0);
            running.set(false);
            last_time.set_value(None);
        }
    }
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
