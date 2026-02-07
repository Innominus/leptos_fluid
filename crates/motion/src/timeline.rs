use std::sync::Arc;

use leptos::prelude::{
    Callable, Callback, GetUntracked, GetValue, ReadValue, RwSignal, Set, SetValue, Signal,
    StoredValue, Update, WriteValue,
};
use leptos_fluid_web::{animation_pause, animation_play, element_get_active_animation};

use crate::components::FluidNodeRef;
use crate::timing::{now_ms, schedule_after};
use crate::{FluidSignal, FluidStyle, Transition};
use web_sys::wasm_bindgen::JsCast;

#[derive(Clone, Debug)]
pub struct FluidStep {
    style: FluidStyle,
    wait_ms: u32,
    easing: Option<String>,
    on_complete: Option<Callback<()>>,
}

impl FluidStep {
    pub fn new(style: FluidStyle) -> Self {
        Self {
            style,
            wait_ms: 0,
            easing: None,
            on_complete: None,
        }
    }

    pub fn wait_ms(mut self, ms: u32) -> Self {
        self.wait_ms = ms;
        self
    }

    pub fn wait_for(mut self, transition: &Transition) -> Self {
        self.wait_ms = transition.duration_ms + transition.delay_ms;
        self.easing = Some(transition.easing_string().to_string());
        self
    }

    pub fn on_complete(mut self, callback: Callback<()>) -> Self {
        self.on_complete = Some(callback);
        self
    }
}

#[derive(Debug, Clone)]
struct FluidTimelineInner {
    value: RwSignal<FluidStyle>,
    generation: StoredValue<u32>,
    running: RwSignal<bool>,
    paused: RwSignal<bool>,
    step_index: RwSignal<usize>,
    step_start: RwSignal<f64>,
    step_wait_ms: RwSignal<u32>,
    remaining_ms: RwSignal<u32>,
    auto_loop: RwSignal<bool>,
    steps: StoredValue<Arc<Vec<FluidStep>>>,
    node_ref: StoredValue<Option<FluidNodeRef>>,
}

#[derive(Clone, Copy, Debug)]
pub struct FluidTimeline {
    inner: StoredValue<FluidTimelineInner>,
}

impl FluidTimeline {
    pub fn new(initial: FluidStyle) -> Self {
        let inner = FluidTimelineInner {
            value: RwSignal::new(initial),
            generation: StoredValue::new(0u32),
            running: RwSignal::new(false),
            paused: RwSignal::new(false),
            step_index: RwSignal::new(0),
            step_start: RwSignal::new(now_ms()),
            step_wait_ms: RwSignal::new(0),
            remaining_ms: RwSignal::new(0),
            auto_loop: RwSignal::new(false),
            steps: StoredValue::new(Arc::new(Vec::new())),
            node_ref: StoredValue::new(None),
        };

        Self {
            inner: StoredValue::new(inner),
        }
    }

    pub fn signal(&self) -> FluidSignal<FluidStyle> {
        self.inner.write_value().value.into()
    }

    pub fn set_steps<I>(&self, steps: I)
    where
        I: IntoIterator<Item = FluidStep>,
    {
        let steps: Vec<FluidStep> = steps.into_iter().collect();
        self.inner.write_value().steps.set_value(Arc::new(steps));
    }

    pub fn attach_node_ref(&self, node_ref: FluidNodeRef) {
        self.inner.write_value().node_ref.set_value(Some(node_ref));
    }

    pub fn set_auto_loop(&self, value: bool) {
        self.inner.write_value().auto_loop.set(value);
    }

    pub fn toggle_auto_loop(&self) {
        self.inner
            .write_value()
            .auto_loop
            .update(|value| *value = !*value);
    }

    pub fn auto_loop(&self) -> Signal<bool> {
        self.inner.write_value().auto_loop.into()
    }

    pub fn step_index(&self) -> Signal<usize> {
        self.inner.write_value().step_index.into()
    }

    pub fn is_paused(&self) -> Signal<bool> {
        self.inner.write_value().paused.into()
    }

    pub fn is_running(&self) -> Signal<bool> {
        self.inner.write_value().running.into()
    }

    pub fn play(&self) {
        start_sequence(self.inner, 0);
    }

    pub fn play_steps<I>(&self, steps: I)
    where
        I: IntoIterator<Item = FluidStep>,
    {
        self.set_steps(steps);
        self.play();
    }

    pub fn pause(&self) {
        let inner = self.inner.get_value();
        if inner.paused.get_untracked() {
            return;
        }
        if !inner.running.get_untracked() {
            return;
        }

        let steps = inner.steps.get_value();
        if steps.is_empty() {
            return;
        }

        inner.paused.set(true);
        inner.running.set(false);
        let generation = inner.generation.get_value().wrapping_add(1);
        inner.generation.set_value(generation);

        let wait_ms = inner.step_wait_ms.get_untracked();
        let elapsed = (now_ms() - inner.step_start.get_untracked()).max(0.0) as u32;
        let remaining = wait_ms.saturating_sub(elapsed).max(1);
        inner.remaining_ms.set(remaining);

        pause_active_animation(&inner);
    }

    pub fn resume(&self) {
        let inner = self.inner.get_value();
        if !inner.paused.get_untracked() {
            return;
        }

        let remaining = inner.remaining_ms.get_untracked().max(1);
        let current_index = inner.step_index.get_untracked();
        let steps = inner.steps.get_value();
        if steps.is_empty() {
            return;
        }

        let generation = inner.generation.get_value().wrapping_add(1);
        inner.generation.set_value(generation);
        inner.running.set(true);
        inner.paused.set(false);
        inner.step_start.set(now_ms());
        inner.step_wait_ms.set(remaining);

        resume_active_animation(&inner);

        let inner_store = self.inner;
        let on_done = make_on_done(inner_store);
        let on_tick = Callback::new(move |_| {
            let inner = inner_store.read_value();
            if inner.generation.get_value() != generation {
                return;
            }

            let steps = inner.steps.get_value();
            if let Some(step) = steps.get(current_index)
                && let Some(callback) = step.on_complete
            {
                callback.run(());
            }

            run_steps(
                inner_store,
                generation,
                steps,
                current_index + 1,
                Some(on_done),
            );
        });
        schedule_after(generation, inner.generation, remaining, on_tick);
    }

    pub fn stop(&self) {
        let inner = self.inner.write_value();
        inner.running.set(false);
        inner.paused.set(false);
        inner.step_wait_ms.set(0);
        let generation = inner.generation.get_value().wrapping_add(1);
        inner.generation.set_value(generation);
    }

    pub fn set_immediate(&self, style: FluidStyle) {
        self.stop();
        self.inner.write_value().value.set(style);
    }
}

fn start_sequence(inner_store: StoredValue<FluidTimelineInner>, start_at: usize) {
    let inner = inner_store.get_value();
    let steps_source = inner.steps.get_value();
    if steps_source.is_empty() || start_at >= steps_source.len() {
        inner.running.set(false);
        return;
    }

    let generation = inner.generation.get_value().wrapping_add(1);
    inner.generation.set_value(generation);
    inner.running.set(true);
    inner.paused.set(false);
    inner.step_wait_ms.set(0);

    let on_done = make_on_done(inner_store);
    run_steps(
        inner_store,
        generation,
        steps_source,
        start_at,
        Some(on_done),
    );
}

fn make_on_done(inner_store: StoredValue<FluidTimelineInner>) -> Callback<()> {
    Callback::new(move |_| {
        let inner = inner_store.get_value();
        if inner.auto_loop.get_untracked() {
            start_sequence(inner_store, 0);
        }
    })
}

fn pause_active_animation(inner: &FluidTimelineInner) {
    let Some(node_ref) = inner.node_ref.get_value() else {
        return;
    };
    let Some(node) = node_ref.get_untracked() else {
        return;
    };
    let element: web_sys::Element = node.unchecked_into();
    let Some(animation) = element_get_active_animation(&element) else {
        return;
    };
    let _ = animation_pause(&animation);
}

fn resume_active_animation(inner: &FluidTimelineInner) {
    let Some(node_ref) = inner.node_ref.get_value() else {
        return;
    };
    let Some(node) = node_ref.get_untracked() else {
        return;
    };
    let element: web_sys::Element = node.unchecked_into();
    let Some(animation) = element_get_active_animation(&element) else {
        return;
    };
    let _ = animation_play(&animation);
}

fn run_steps(
    inner_store: StoredValue<FluidTimelineInner>,
    generation: u32,
    steps: Arc<Vec<FluidStep>>,
    index: usize,
    on_done: Option<Callback<()>>,
) {
    let inner = inner_store.read_value();
    if inner.generation.get_value() != generation {
        return;
    }

    if index >= steps.len() {
        inner.running.set(false);
        if let Some(callback) = on_done {
            callback.run(());
        }
        return;
    }

    let step = steps[index].clone();

    inner.step_index.set(index);
    inner.step_start.set(now_ms());
    inner.step_wait_ms.set(step.wait_ms);
    inner.value.set(step.style);

    let wait_ms = step.wait_ms;
    if wait_ms == 0 {
        if let Some(callback) = step.on_complete {
            callback.run(());
        }
        run_steps(inner_store, generation, steps, index + 1, on_done);
        return;
    }

    let inner_store_next = inner_store;
    let steps_next = steps.clone();
    let on_complete = step.on_complete;
    let on_tick = Callback::new(move |_| {
        if let Some(callback) = on_complete {
            callback.run(());
        }
        run_steps(
            inner_store_next,
            generation,
            steps_next.clone(),
            index + 1,
            on_done,
        );
    });
    schedule_after(generation, inner.generation, wait_ms, on_tick);
}
