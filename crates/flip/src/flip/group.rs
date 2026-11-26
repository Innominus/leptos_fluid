use std::cell::Cell;
use std::rc::Rc;

use leptos::prelude::*;
use web_sys::Element;

use super::{
    FlipAnimation, FlipGroupBuilder, FlipItem, FlipOptions, FlipValues, InlineStyles,
    find_inline_by_key, find_values_by_key, has_flip_delta_with_size, query_elements,
    run_flip_animation, snapshot_elements, stop_group_animations,
};

type CarriedInline = Vec<(String, InlineStyles)>;

/// Multi-element FLIP animator identified by a CSS selector.
#[derive(Clone, Copy)]
pub struct FlipGroup {
    selector: StoredValue<String>,
    is_animating: RwSignal<bool>,
    options: StoredValue<FlipOptions>,
    animations: StoredValue<Vec<FlipAnimation>, LocalStorage>,
    pending_frame: StoredValue<Option<(AnimationFrameRequestHandle, CarriedInline)>, LocalStorage>,
}

impl FlipGroup {
    pub fn builder() -> FlipGroupBuilder {
        FlipGroupBuilder::new()
    }

    pub(crate) fn from_selector(selector: String, options: FlipOptions) -> Self {
        let group = Self {
            selector: StoredValue::new(selector),
            is_animating: RwSignal::new(false),
            options: StoredValue::new(options),
            animations: StoredValue::new_local(Vec::new()),
            pending_frame: StoredValue::new_local(None),
        };
        // Owner disposal: no frame or WAAPI callback may outlive these stores.
        on_cleanup(move || group.stop_animations());
        group
    }

    pub fn new(selector: String) -> Self {
        Self::from_selector(selector, FlipOptions::new())
    }

    pub fn new_with_options(selector: String, options: FlipOptions) -> Self {
        Self::from_selector(selector, options)
    }

    pub fn set_selector(&mut self, selector: String) {
        self.selector.set_value(selector);
    }

    pub fn set_options(&mut self, options: FlipOptions) {
        self.options.set_value(options);
    }

    pub fn options(&self) -> FlipOptions {
        self.options.get_value()
    }

    pub fn is_animating(&self) -> Signal<bool> {
        self.is_animating.into()
    }

    pub fn get_is_animating_signal(&self) -> Signal<bool> {
        self.is_animating()
    }

    pub fn run<F>(&self, animator_fn: F)
    where
        F: FnMut() + 'static,
    {
        self.run_dyn(Box::new(animator_fn));
    }

    pub fn animate<F>(&self, animator_fn: F)
    where
        F: FnMut() + 'static,
    {
        self.run(animator_fn);
    }

    pub fn stop(&self) {
        self.stop_animations();
        self.is_animating.set(false);
    }

    /// Cancels the pending frame and freezes active animations, without touching signals.
    fn stop_animations(&self) {
        if self.animations.is_disposed() {
            return;
        }
        self.take_pending_frame();
        stop_group_animations(self.animations);
    }

    /// Cancels a measurement frame that has not run yet and returns the inline
    /// styles it would have restored, so a stale run cannot start after stop/replay.
    fn take_pending_frame(&self) -> CarriedInline {
        let Some((handle, carried_inline)) =
            self.pending_frame.try_update_value(Option::take).flatten()
        else {
            return Vec::new();
        };
        handle.cancel();
        carried_inline
    }

    fn run_dyn(&self, mut animator_fn: Box<dyn FnMut() + 'static>) {
        if self.animations.is_disposed() {
            animator_fn();
            return;
        }
        let is_animating = self.is_animating;
        // A replayed pending frame never ran, so its carried styles still need restoring.
        let mut carried_inline = self.take_pending_frame();
        carried_inline.extend(stop_group_animations(self.animations));
        let from_values = self.snapshot_values();

        animator_fn();
        is_animating.set(true);

        let selector = self.selector;
        let options = self.options();
        let animations_store = self.animations;
        let pending_frame = self.pending_frame;

        match request_animation_frame(move || {
            // The carried styles live in the pending slot until the frame runs; take them back here.
            let carried_inline = pending_frame
                .try_update_value(Option::take)
                .flatten()
                .map(|(_, carried_inline)| carried_inline)
                .unwrap_or_default();
            let elements = selector.with_value(|value| query_elements(value));
            restore_carried_inline(&elements, &carried_inline);

            let to_items = snapshot_elements(elements);
            // Starts at 1 as a loop guard so a synchronous completion (refused fallback frame)
            // can't reach zero and clear `is_animating` before later items start.
            let remaining = Rc::new(Cell::new(1usize));
            let mut new_animations = Vec::new();

            for (index, to_item) in to_items.into_iter().enumerate() {
                let Some(from_item_values) = find_values_by_key(&from_values, &to_item.key) else {
                    continue;
                };
                if !has_flip_delta_with_size(
                    from_item_values,
                    &to_item.values,
                    options.scale_mode.uses_scale(),
                ) {
                    continue;
                }

                remaining.set(remaining.get() + 1);
                let remaining_inner = remaining.clone();
                let is_animating_inner = is_animating;
                let item_options = options.with_stagger_index(index);
                let on_finish: Rc<dyn Fn()> = Rc::new(move || {
                    let next = remaining_inner.get().saturating_sub(1);
                    remaining_inner.set(next);
                    if next == 0 {
                        is_animating_inner.set(false);
                    }
                });
                let animation = run_flip_animation(
                    to_item.element,
                    *from_item_values,
                    to_item.values,
                    item_options,
                    on_finish,
                );
                new_animations.push(animation);
            }

            let left = remaining.get() - 1;
            remaining.set(left);
            if left == 0 {
                is_animating.set(false);
            }

            animations_store.set_value(new_animations);
        }) {
            Ok(handle) => {
                pending_frame.set_value(Some((handle, carried_inline)));
                if Owner::current().is_some() {
                    // Leptos cancels an owned frame when its owner cleans up (e.g. an effect
                    // rerun); settle then so `is_animating` and frozen styles can't wedge.
                    on_cleanup(move || {
                        let abandoned = pending_frame
                            .try_update_value(|slot| {
                                if slot.as_ref().is_some_and(|(pending, _)| *pending == handle) {
                                    slot.take()
                                } else {
                                    None
                                }
                            })
                            .flatten();
                        if let Some((_, carried_inline)) = abandoned {
                            abandon_frame(selector, &carried_inline, is_animating);
                        }
                    });
                }
            }
            // Scheduling failed: don't report the animation as forever in-flight.
            Err(_) => abandon_frame(selector, &carried_inline, is_animating),
        }
    }

    fn snapshot_values(&self) -> Vec<(String, FlipValues)> {
        let elements = self.selector.with_value(|value| query_elements(value));
        snapshot_elements(elements)
            .into_iter()
            .map(|FlipItem { key, values, .. }| (key, values))
            .collect()
    }
}

/// The measurement frame never ran: drop frozen transforms carried from the
/// stopped animations and clear in-flight state.
fn abandon_frame(
    selector: StoredValue<String>,
    carried_inline: &[(String, InlineStyles)],
    is_animating: RwSignal<bool>,
) {
    if let Some(elements) = selector.try_with_value(|value| query_elements(value)) {
        restore_carried_inline(&elements, carried_inline);
    }
    is_animating.try_set(false);
}

fn restore_carried_inline(elements: &[Element], carried_inline: &[(String, InlineStyles)]) {
    for element in elements {
        if let Some(key) = super::element_key(element)
            && let Some(inline) = find_inline_by_key(carried_inline, &key)
        {
            super::restore_inline_styles(element, inline);
        }
    }
}
