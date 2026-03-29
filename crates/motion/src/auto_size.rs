use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use leptos::html::ElementType;
use leptos::prelude::{Effect, Get, NodeRef, on_cleanup, request_animation_frame};
use leptos::wasm_bindgen::JsCast;
use leptos_fluid_web::{ResizeObserverHandle, html_style, observe_resize, restore_inline_property};
use web_sys::Element;

use crate::{AnimationController, FluidStyle, Transition};

const SIZE_EPSILON: f64 = 0.5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutoSizeAxis {
    Height,
    Width,
    Both,
}

#[derive(Clone, Debug)]
pub struct AutoSizeOptions {
    pub transition: Option<Transition>,
    pub clear_inline_size: bool,
    pub hide_overflow: bool,
}

impl Default for AutoSizeOptions {
    fn default() -> Self {
        Self {
            transition: None,
            clear_inline_size: true,
            hide_overflow: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct MeasuredSize {
    width: f64,
    height: f64,
}

#[derive(Clone, Debug, Default)]
struct InlineSnapshot {
    width: String,
    height: String,
    overflow: String,
}

struct AutoSizeBinding {
    controller: AnimationController,
    axis: AutoSizeAxis,
    options: AutoSizeOptions,
    container: Option<Element>,
    content: Option<Element>,
    inline_snapshot: Option<InlineSnapshot>,
    last_measured: Option<MeasuredSize>,
    is_pinned: bool,
    raf_scheduled: bool,
    generation: u32,
}

impl AutoSizeBinding {
    fn new(controller: AnimationController, axis: AutoSizeAxis, options: AutoSizeOptions) -> Self {
        Self {
            controller,
            axis,
            options,
            container: None,
            content: None,
            inline_snapshot: None,
            last_measured: None,
            is_pinned: false,
            raf_scheduled: false,
            generation: 0,
        }
    }

    fn update_nodes(
        &mut self,
        container: Option<Element>,
        content: Option<Element>,
        handle: Rc<RefCell<Self>>,
        cleanup_observer: Arc<Mutex<Option<ResizeObserverHandle>>>,
    ) {
        let container_changed = self.container != container;
        let content_changed = self.content != content;
        if !container_changed && !content_changed {
            return;
        }

        if let Some(mut observer) = cleanup_observer
            .lock()
            .expect("auto-size observer lock poisoned")
            .take()
        {
            observer.disconnect();
        }
        self.container = container.clone();
        self.content = content.clone();
        self.inline_snapshot = container.as_ref().map(capture_inline_snapshot);
        self.last_measured = content.as_ref().and_then(measure_size);
        self.is_pinned = false;
        self.raf_scheduled = false;
        self.generation = self.generation.wrapping_add(1);

        let Some(content) = content else {
            return;
        };

        let mut observer = observe_resize(&content, move || schedule_resize_flush(handle.clone()));
        if self.last_measured.is_none() {
            observer.disconnect();
            return;
        }
        *cleanup_observer
            .lock()
            .expect("auto-size observer lock poisoned") = Some(observer);
    }
}

pub fn bind_auto_height<C, T>(
    controller: AnimationController,
    container_ref: NodeRef<C>,
    content_ref: NodeRef<T>,
) where
    C: ElementType,
    C::Output: JsCast + Clone + 'static,
    T: ElementType,
    T::Output: JsCast + Clone + 'static,
{
    bind_auto_height_with(
        controller,
        container_ref,
        content_ref,
        AutoSizeOptions::default(),
    );
}

pub fn bind_auto_height_with<C, T>(
    controller: AnimationController,
    container_ref: NodeRef<C>,
    content_ref: NodeRef<T>,
    options: AutoSizeOptions,
) where
    C: ElementType,
    C::Output: JsCast + Clone + 'static,
    T: ElementType,
    T::Output: JsCast + Clone + 'static,
{
    bind_auto_size_with(
        controller,
        container_ref,
        content_ref,
        AutoSizeAxis::Height,
        options,
    );
}

pub fn bind_auto_width<C, T>(
    controller: AnimationController,
    container_ref: NodeRef<C>,
    content_ref: NodeRef<T>,
) where
    C: ElementType,
    C::Output: JsCast + Clone + 'static,
    T: ElementType,
    T::Output: JsCast + Clone + 'static,
{
    bind_auto_width_with(
        controller,
        container_ref,
        content_ref,
        AutoSizeOptions::default(),
    );
}

pub fn bind_auto_width_with<C, T>(
    controller: AnimationController,
    container_ref: NodeRef<C>,
    content_ref: NodeRef<T>,
    options: AutoSizeOptions,
) where
    C: ElementType,
    C::Output: JsCast + Clone + 'static,
    T: ElementType,
    T::Output: JsCast + Clone + 'static,
{
    bind_auto_size_with(
        controller,
        container_ref,
        content_ref,
        AutoSizeAxis::Width,
        options,
    );
}

pub fn bind_auto_size<C, T>(
    controller: AnimationController,
    container_ref: NodeRef<C>,
    content_ref: NodeRef<T>,
) where
    C: ElementType,
    C::Output: JsCast + Clone + 'static,
    T: ElementType,
    T::Output: JsCast + Clone + 'static,
{
    bind_auto_size_with(
        controller,
        container_ref,
        content_ref,
        AutoSizeAxis::Both,
        AutoSizeOptions::default(),
    );
}

pub fn bind_auto_size_with<C, T>(
    controller: AnimationController,
    container_ref: NodeRef<C>,
    content_ref: NodeRef<T>,
    axis: AutoSizeAxis,
    options: AutoSizeOptions,
) where
    C: ElementType,
    C::Output: JsCast + Clone + 'static,
    T: ElementType,
    T::Output: JsCast + Clone + 'static,
{
    let cleanup_observer = Arc::new(Mutex::new(None));
    let binding = Rc::new(RefCell::new(AutoSizeBinding::new(
        controller, axis, options,
    )));

    Effect::new({
        let binding = binding.clone();
        let cleanup_observer = cleanup_observer.clone();
        move || {
            let container = container_ref.get().map(|node| node.unchecked_into());
            let content = content_ref.get().map(|node| node.unchecked_into());
            binding.borrow_mut().update_nodes(
                container,
                content,
                binding.clone(),
                cleanup_observer.clone(),
            );
        }
    });

    on_cleanup(move || {
        if let Some(mut observer) = cleanup_observer
            .lock()
            .expect("auto-size observer lock poisoned")
            .take()
        {
            observer.disconnect();
        }
    });
}

fn schedule_resize_flush(binding: Rc<RefCell<AutoSizeBinding>>) {
    {
        let mut binding_ref = binding.borrow_mut();
        if binding_ref.raf_scheduled {
            return;
        }
        binding_ref.raf_scheduled = true;
    }

    request_animation_frame(move || process_resize(binding));
}

fn process_resize(binding: Rc<RefCell<AutoSizeBinding>>) {
    let pending_cleanup = {
        let mut binding_ref = binding.borrow_mut();
        binding_ref.raf_scheduled = false;

        let Some(container) = binding_ref.container.clone() else {
            return;
        };
        let Some(content) = binding_ref.content.clone() else {
            return;
        };
        let Some(next_size) = measure_size(&content) else {
            return;
        };

        let Some(previous_size) = binding_ref.last_measured else {
            binding_ref.last_measured = Some(next_size);
            return;
        };

        if !size_changed(previous_size, next_size, binding_ref.axis) {
            return;
        }

        let from_size = if binding_ref.is_pinned {
            measure_size(&container).unwrap_or(previous_size)
        } else {
            previous_size
        };

        apply_temporary_size(&container, &binding_ref, from_size);
        let transition = binding_ref
            .options
            .transition
            .clone()
            .unwrap_or_else(|| binding_ref.controller.transition());
        binding_ref
            .controller
            .animate_with(size_style(binding_ref.axis, next_size), transition.clone());
        binding_ref.last_measured = Some(next_size);
        binding_ref.is_pinned = true;
        binding_ref.generation = binding_ref.generation.wrapping_add(1);

        if !binding_ref.options.clear_inline_size {
            None
        } else {
            Some((
                binding_ref.generation,
                transition
                    .duration_ms
                    .saturating_add(transition.delay_ms)
                    .saturating_add(34),
            ))
        }
    };

    if let Some((generation, delay_ms)) = pending_cleanup {
        schedule_cleanup(binding, generation, delay_ms);
    }
}

fn apply_temporary_size(container: &Element, binding: &AutoSizeBinding, size: MeasuredSize) {
    let Some(style) = html_style(container) else {
        return;
    };

    if binding.options.hide_overflow {
        let _ = style.set_property("overflow", "hidden");
    }

    match binding.axis {
        AutoSizeAxis::Height => {
            let _ = style.set_property("height", &format!("{}px", size.height));
        }
        AutoSizeAxis::Width => {
            let _ = style.set_property("width", &format!("{}px", size.width));
        }
        AutoSizeAxis::Both => {
            let _ = style.set_property("width", &format!("{}px", size.width));
            let _ = style.set_property("height", &format!("{}px", size.height));
        }
    }
}

fn schedule_cleanup(binding: Rc<RefCell<AutoSizeBinding>>, generation: u32, delay_ms: u32) {
    #[cfg(target_arch = "wasm32")]
    {
        let Some(window) = web_sys::window() else {
            finalize_cleanup(binding, generation);
            return;
        };

        let callback = web_sys::wasm_bindgen::closure::Closure::once_into_js(move || {
            finalize_cleanup(binding, generation);
        });
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            callback.unchecked_ref(),
            delay_ms as i32,
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = delay_ms;
        finalize_cleanup(binding, generation);
    }
}

fn finalize_cleanup(binding: Rc<RefCell<AutoSizeBinding>>, generation: u32) {
    let mut binding_ref = binding.borrow_mut();
    if binding_ref.generation != generation {
        return;
    }

    let Some(container) = binding_ref.container.clone() else {
        return;
    };
    let Some(snapshot) = binding_ref.inline_snapshot.clone() else {
        return;
    };

    let Some(style) = html_style(&container) else {
        return;
    };

    match binding_ref.axis {
        AutoSizeAxis::Height => restore_inline_property(&style, "height", &snapshot.height),
        AutoSizeAxis::Width => restore_inline_property(&style, "width", &snapshot.width),
        AutoSizeAxis::Both => {
            restore_inline_property(&style, "width", &snapshot.width);
            restore_inline_property(&style, "height", &snapshot.height);
        }
    }

    if binding_ref.options.hide_overflow {
        restore_inline_property(&style, "overflow", &snapshot.overflow);
    }
    binding_ref.is_pinned = false;
}

fn capture_inline_snapshot(container: &Element) -> InlineSnapshot {
    let Some(style) = html_style(container) else {
        return InlineSnapshot::default();
    };

    InlineSnapshot {
        width: style.get_property_value("width").unwrap_or_default(),
        height: style.get_property_value("height").unwrap_or_default(),
        overflow: style.get_property_value("overflow").unwrap_or_default(),
    }
}

fn measure_size(element: &Element) -> Option<MeasuredSize> {
    let rect = element.get_bounding_client_rect();
    let width = rect.width();
    let height = rect.height();
    if !width.is_finite() || !height.is_finite() {
        return None;
    }

    Some(MeasuredSize { width, height })
}

fn size_changed(previous: MeasuredSize, next: MeasuredSize, axis: AutoSizeAxis) -> bool {
    match axis {
        AutoSizeAxis::Height => (previous.height - next.height).abs() > SIZE_EPSILON,
        AutoSizeAxis::Width => (previous.width - next.width).abs() > SIZE_EPSILON,
        AutoSizeAxis::Both => {
            (previous.width - next.width).abs() > SIZE_EPSILON
                || (previous.height - next.height).abs() > SIZE_EPSILON
        }
    }
}

fn size_style(axis: AutoSizeAxis, size: MeasuredSize) -> FluidStyle {
    match axis {
        AutoSizeAxis::Height => FluidStyle::new().height(size.height),
        AutoSizeAxis::Width => FluidStyle::new().width(size.width),
        AutoSizeAxis::Both => FluidStyle::new().size(size.width, size.height),
    }
}

#[cfg(test)]
mod tests {
    use super::{AutoSizeAxis, MeasuredSize, size_changed};

    #[test]
    fn size_change_respects_axis() {
        let previous = MeasuredSize {
            width: 120.0,
            height: 48.0,
        };
        let next = MeasuredSize {
            width: 120.0,
            height: 96.0,
        };

        assert!(size_changed(previous, next, AutoSizeAxis::Height));
        assert!(!size_changed(previous, next, AutoSizeAxis::Width));
        assert!(size_changed(previous, next, AutoSizeAxis::Both));
    }
}
