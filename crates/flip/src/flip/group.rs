use std::cell::Cell;
use std::rc::Rc;

use leptos::prelude::*;

use super::{
    FlipAnimation, FlipGroupBuilder, FlipItem, FlipOptions, FlipValues, find_inline_by_key,
    find_values_by_key, has_flip_delta_with_size, query_elements, run_flip_animation,
    snapshot_elements, stop_group_animations,
};

/// Multi-element FLIP animator identified by a CSS selector.
#[derive(Clone, Copy)]
pub struct FlipGroup {
    selector: StoredValue<String>,
    is_animating: RwSignal<bool>,
    options: StoredValue<FlipOptions>,
    animations: StoredValue<Vec<FlipAnimation>, LocalStorage>,
}

impl FlipGroup {
    pub fn builder() -> FlipGroupBuilder {
        FlipGroupBuilder::new()
    }

    pub(crate) fn from_selector(selector: String, options: FlipOptions) -> Self {
        Self {
            selector: StoredValue::new(selector),
            is_animating: RwSignal::new(false),
            options: StoredValue::new(options),
            animations: StoredValue::new_local(Vec::new()),
        }
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
        stop_group_animations(self.animations);
        self.is_animating.set(false);
    }

    fn run_dyn(&self, mut animator_fn: Box<dyn FnMut() + 'static>) {
        let is_animating = self.is_animating;
        let carried_inline = stop_group_animations(self.animations);
        let from_values = self.snapshot_values();

        animator_fn();
        is_animating.set(true);

        let selector = self.selector;
        let options = self.options();
        let animations_store = self.animations;

        request_animation_frame(move || {
            let elements = selector.with_value(|value| query_elements(value));
            for element in &elements {
                if let Some(key) = super::element_key(element)
                    && let Some(inline) = find_inline_by_key(&carried_inline, &key)
                {
                    super::restore_inline_styles(element, inline);
                }
            }

            let to_items = snapshot_elements(elements);
            let remaining = Rc::new(Cell::new(0usize));
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

            if remaining.get() == 0 {
                is_animating.set(false);
            }

            animations_store.set_value(new_animations);
        });
    }

    fn snapshot_values(&self) -> Vec<(String, FlipValues)> {
        let elements = self.selector.with_value(|value| query_elements(value));
        snapshot_elements(elements)
            .into_iter()
            .map(|FlipItem { key, values, .. }| (key, values))
            .collect()
    }
}
