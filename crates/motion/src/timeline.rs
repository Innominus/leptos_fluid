use std::rc::Rc;
use std::sync::Arc;

use leptos::prelude::{
    Callable, Callback, Effect, GetUntracked, GetValue, LocalStorage, NodeRef, ReadValue, RwSignal,
    Set, SetValue, Signal, StoredValue, Update, WriteValue,
};
use leptos_fluid_web::{animation_pause, animation_play, element_get_active_animation};

use crate::timing::{now_ms, schedule_after};
use crate::{AnimationController, FluidSignal, FluidStyle, Transition};
use leptos::html::ElementType;
use web_sys::Element;
use web_sys::wasm_bindgen::JsCast;

/// One timeline step containing a target style, wait duration, and callback.
#[derive(Clone, Debug)]
pub struct FluidStep {
    style: FluidStyle,
    wait_ms: u32,
    wait_defined: bool,
    on_complete: Option<Callback<()>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TimelineBindingMode {
    Animate,
    Immediate,
}

impl FluidStep {
    #[inline]
    pub fn new(style: FluidStyle) -> Self {
        Self {
            style,
            wait_ms: 0,
            wait_defined: false,
            on_complete: None,
        }
    }

    #[inline]
    pub fn to(style: FluidStyle) -> Self {
        Self::new(style)
    }

    #[inline]
    pub fn wait_ms(mut self, ms: u32) -> Self {
        self.wait_ms = ms;
        self.wait_defined = true;
        self
    }

    #[inline]
    pub fn wait_for(mut self, transition: &Transition) -> Self {
        self.wait_ms = transition.duration_ms + transition.delay_ms;
        self.wait_defined = true;
        self
    }

    #[inline]
    pub fn on_complete(mut self, callback: Callback<()>) -> Self {
        self.on_complete = Some(callback);
        self
    }

    #[inline]
    pub fn inherit_wait_from(mut self, transition: &Transition) -> Self {
        if !self.wait_defined {
            self.wait_ms = transition.duration_ms + transition.delay_ms;
        }
        self
    }
}

#[derive(Debug, Clone)]
struct FluidTimelineInner {
    value: RwSignal<FluidStyle>,
    update_mode: StoredValue<TimelineBindingMode, LocalStorage>,
    generation: StoredValue<u32>,
    running: RwSignal<bool>,
    paused: RwSignal<bool>,
    step_index: RwSignal<usize>,
    step_start: RwSignal<f64>,
    step_wait_ms: RwSignal<u32>,
    remaining_ms: RwSignal<u32>,
    auto_loop: RwSignal<bool>,
    steps: StoredValue<Arc<[FluidStep]>>,
    pause_target: StoredValue<Option<Rc<dyn Fn() -> Option<Element>>>, LocalStorage>,
    bound_controller: StoredValue<Option<AnimationController>, LocalStorage>,
}

/// Sequencer that drives a `FluidStyle` signal through ordered `FluidStep`s.
#[derive(Clone, Copy, Debug)]
pub struct FluidTimeline {
    inner: StoredValue<FluidTimelineInner>,
}

impl FluidTimeline {
    pub fn new(initial: FluidStyle) -> Self {
        let inner = FluidTimelineInner {
            value: RwSignal::new(initial),
            update_mode: StoredValue::new_local(TimelineBindingMode::Animate),
            generation: StoredValue::new(0u32),
            running: RwSignal::new(false),
            paused: RwSignal::new(false),
            step_index: RwSignal::new(0),
            step_start: RwSignal::new(now_ms()),
            step_wait_ms: RwSignal::new(0),
            remaining_ms: RwSignal::new(0),
            auto_loop: RwSignal::new(false),
            steps: StoredValue::new(Arc::from(Vec::<FluidStep>::new())),
            pause_target: StoredValue::new_local(None),
            bound_controller: StoredValue::new_local(None),
        };

        Self {
            inner: StoredValue::new(inner),
        }
    }

    pub fn signal(&self) -> FluidSignal<FluidStyle> {
        FluidSignal::from_rw_signal(self.inner.write_value().value)
    }

    pub fn set_steps(&self, steps: Vec<FluidStep>) {
        self.inner.write_value().steps.set_value(Arc::from(steps));
    }

    pub fn attach_node_ref<E>(&self, node_ref: NodeRef<E>)
    where
        E: ElementType,
        E::Output: JsCast + Clone + 'static,
    {
        self.attach_resolver(move || node_ref.get_untracked().map(|node| node.unchecked_into()));
    }

    pub fn attach_resolver<F>(&self, resolver: F)
    where
        F: Fn() -> Option<Element> + 'static,
    {
        self.inner
            .write_value()
            .pause_target
            .set_value(Some(Rc::new(resolver)));
    }

    pub fn bind(&self, controller: AnimationController) {
        self.inner
            .write_value()
            .bound_controller
            .set_value(Some(controller));

        let value = self.signal();
        let update_mode = self.inner.read_value().update_mode;
        let initialized: StoredValue<bool, LocalStorage> = StoredValue::new_local(false);
        Effect::new(move || {
            let next = value.get();
            let mode = update_mode.get_value();
            if !initialized.get_value() || mode == TimelineBindingMode::Immediate {
                controller.set_immediate(next);
                initialized.set_value(true);
            } else {
                controller.animate(next);
            }
            update_mode.set_value(TimelineBindingMode::Animate);
        });
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

    #[inline]
    pub fn restart(&self) {
        self.play();
    }

    pub fn play_steps(&self, steps: Vec<FluidStep>) {
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

        pause_timeline_animation(&inner);
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

        resume_timeline_animation(&inner);

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

        if let Some(controller) = inner.bound_controller.get_value() {
            controller.stop();
        } else if let Some(animation) = active_timeline_animation(&inner) {
            let _ = animation_pause(&animation);
        }
    }

    pub fn set_immediate(&self, style: FluidStyle) {
        self.stop();
        let inner = self.inner.write_value();
        inner.update_mode.set_value(TimelineBindingMode::Immediate);
        inner.value.set(style);
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
    // Bumping generation invalidates previously scheduled callbacks.
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
            // Re-enter from step zero to keep loop timing deterministic.
            start_sequence(inner_store, 0);
        }
    })
}

fn pause_timeline_animation(inner: &FluidTimelineInner) {
    if let Some(controller) = inner.bound_controller.get_value() {
        let _ = controller.pause();
        return;
    }

    let Some(animation) = active_timeline_animation(inner) else {
        return;
    };
    let _ = animation_pause(&animation);
}

fn resume_timeline_animation(inner: &FluidTimelineInner) {
    if let Some(controller) = inner.bound_controller.get_value() {
        let _ = controller.resume();
        return;
    }

    let Some(animation) = active_timeline_animation(inner) else {
        return;
    };
    let _ = animation_play(&animation);
}

fn active_timeline_animation(inner: &FluidTimelineInner) -> Option<web_sys::Animation> {
    let element = inner.pause_target.get_value()?.as_ref()()?;
    element_get_active_animation(&element)
}

#[inline(never)]
fn run_steps(
    inner_store: StoredValue<FluidTimelineInner>,
    generation: u32,
    steps: Arc<[FluidStep]>,
    index: usize,
    on_done: Option<Callback<()>>,
) {
    let inner = inner_store.read_value();
    if inner.generation.get_value() != generation {
        // A newer run replaced this one; stop quietly.
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
    inner.update_mode.set_value(TimelineBindingMode::Animate);
    inner.value.set(step.style);

    let wait_ms = step.wait_ms;
    if wait_ms == 0 {
        // Zero-wait steps chain synchronously to support instant style stages.
        if let Some(callback) = step.on_complete {
            callback.run(());
        }
        run_steps(inner_store, generation, steps, index + 1, on_done);
        return;
    }

    let inner_store_next = inner_store;
    let on_complete = step.on_complete;
    let on_tick = Callback::new(move |_| {
        if let Some(callback) = on_complete {
            callback.run(());
        }
        run_steps(
            inner_store_next,
            generation,
            steps.clone(),
            index + 1,
            on_done,
        );
    });
    schedule_after(generation, inner.generation, wait_ms, on_tick);
}

#[cfg(test)]
mod tests {
    use super::FluidStep;
    use crate::{FluidStyle, Transition};

    #[test]
    fn inherited_wait_uses_transition_when_unset() {
        let transition = Transition::new().duration_ms(320).delay_ms(40);
        let step = FluidStep::new(FluidStyle::new()).inherit_wait_from(&transition);

        assert_eq!(step.wait_ms, 360);
    }

    #[test]
    fn inherited_wait_preserves_explicit_zero_wait() {
        let transition = Transition::new().duration_ms(320).delay_ms(40);
        let step = FluidStep::new(FluidStyle::new())
            .wait_ms(0)
            .inherit_wait_from(&transition);

        assert_eq!(step.wait_ms, 0);
    }
}
