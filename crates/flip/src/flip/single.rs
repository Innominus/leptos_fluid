use std::rc::Rc;

use leptos::prelude::*;
use web_sys::Element;

use super::{
    FlipAnimation, FlipBuilder, FlipOptions, FlipTarget, FlipTargetSource, FlipValues,
    has_flip_delta_with_size, read_border_radius_target, restore_inline_styles, run_flip_animation,
    stop_flip_animation_state,
};

/// Single-element FLIP animator.
#[derive(Clone, Copy)]
pub struct Flip {
    target_source: StoredValue<FlipTargetSource, LocalStorage>,
    is_animating: RwSignal<bool>,
    options: StoredValue<FlipOptions>,
    animation: StoredValue<Option<FlipAnimation>, LocalStorage>,
    pending_frame: StoredValue<Option<AnimationFrameRequestHandle>, LocalStorage>,
}

impl Flip {
    pub fn builder() -> FlipBuilder {
        FlipBuilder::new()
    }

    pub(crate) fn from_source(source: FlipTargetSource, options: FlipOptions) -> Self {
        let flip = Self {
            target_source: StoredValue::new_local(source),
            is_animating: RwSignal::new(false),
            options: StoredValue::new(options),
            animation: StoredValue::new_local(None),
            pending_frame: StoredValue::new_local(None),
        };
        // Owner disposal: no frame or WAAPI callback may outlive these stores.
        on_cleanup(move || flip.stop_animation());
        flip
    }

    /// `id_selector` must be the raw id value (for example `"card-a"`), not `"#card-a"`.
    pub fn new(id_selector: String) -> Self {
        Self::from_source(
            FlipTargetSource::IdSelector(id_selector),
            FlipOptions::new(),
        )
    }

    pub fn new_with_options(id_selector: String, options: FlipOptions) -> Self {
        Self::from_source(FlipTargetSource::IdSelector(id_selector), options)
    }

    pub fn set_id_selector(&mut self, id_selector: String) {
        self.target_source
            .set_value(FlipTargetSource::IdSelector(id_selector));
    }

    pub fn set_target<T: FlipTarget>(&mut self, target: T) {
        self.target_source.set_value(target.into_source());
    }

    pub fn set_resolver<F>(&mut self, resolver: F)
    where
        F: Fn() -> Option<Element> + 'static,
    {
        self.target_source
            .set_value(FlipTargetSource::Resolver(Rc::new(resolver)));
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

    /// Runs a FLIP capture around a state mutation closure.
    pub fn run<F>(&self, animator_fn: F)
    where
        F: FnMut() + 'static,
    {
        self.run_dyn(Box::new(animator_fn));
    }

    /// Compatibility alias for `run(...)`.
    pub fn animate<F>(&self, animator_fn: F)
    where
        F: FnMut() + 'static,
    {
        self.run(animator_fn);
    }

    pub fn stop(&self) {
        self.stop_animation();
        self.is_animating.set(false);
    }

    /// Cancels the pending frame and freezes any active animation, without touching signals.
    fn stop_animation(&self) {
        if self.animation.is_disposed() {
            return;
        }
        self.cancel_pending_frame();
        if let Some(animation_state) = self.animation.get_value() {
            stop_flip_animation_state(&animation_state);
        }
        self.animation.set_value(None);
    }

    pub fn try_measure(&self, element: Option<Element>) -> Option<(Element, FlipValues)> {
        element
            .map(Self::rect)
            .or_else(|| self.resolve_target().map(Self::rect))
    }

    pub fn measure(&self, element: Option<Element>) -> (Element, FlipValues) {
        self.try_measure(element)
            .expect("Flip target could not be resolved for measurement")
    }

    pub fn rect(element: Element) -> (Element, FlipValues) {
        let rect = element.get_bounding_client_rect();
        let border_radius = read_border_radius_target(&element);

        (
            element,
            FlipValues {
                left: rect.left(),
                top: rect.top(),
                width: rect.width(),
                height: rect.height(),
                border_radius,
            },
        )
    }

    fn resolve_target(&self) -> Option<Element> {
        self.target_source.get_value().resolve()
    }

    /// Cancels a measurement frame that has not run yet so a stale run cannot start after stop/replay.
    fn cancel_pending_frame(&self) {
        if let Some(handle) = self.pending_frame.get_value() {
            handle.cancel();
            self.pending_frame.set_value(None);
        }
    }

    fn run_dyn(&self, mut animator_fn: Box<dyn FnMut() + 'static>) {
        if self.animation.is_disposed() {
            animator_fn();
            return;
        }
        let is_animating = self.is_animating;
        let mut carried_inline_styles = None;

        // The stopped animation stays stored until the frame runs, so replay re-carries its styles.
        self.cancel_pending_frame();

        if let Some(animation_state) = self.animation.get_value() {
            stop_flip_animation_state(&animation_state);
            carried_inline_styles = Some(animation_state.inline_styles);
        }

        let Some((_element, from_values)) = self.try_measure(None) else {
            self.animation.set_value(None);
            self.is_animating.set(false);
            return;
        };

        animator_fn();
        is_animating.set(true);

        let options = self.options();
        let animation_store = self.animation;
        let pending_frame = self.pending_frame;
        let inner_self = *self;

        match request_animation_frame(move || {
            pending_frame.set_value(None);
            let Some(element) = inner_self.resolve_target() else {
                is_animating.set(false);
                animation_store.set_value(None);
                return;
            };

            if let Some(inline_styles) = carried_inline_styles.as_ref() {
                restore_inline_styles(&element, inline_styles);
            }

            let (element, to_values) = Self::rect(element);
            if !has_flip_delta_with_size(&from_values, &to_values, options.scale_mode.uses_scale())
            {
                is_animating.set(false);
                animation_store.set_value(None);
                return;
            }

            let on_finish: Rc<dyn Fn()> = Rc::new(move || {
                is_animating.set(false);
            });
            let animation = run_flip_animation(element, from_values, to_values, options, on_finish);
            animation_store.set_value(Some(animation));
        }) {
            Ok(handle) => {
                pending_frame.set_value(Some(handle));
                if Owner::current().is_some() {
                    // Leptos cancels an owned frame when its owner cleans up (e.g. an effect
                    // rerun); settle then so `is_animating` and frozen styles can't wedge.
                    on_cleanup(move || {
                        if pending_frame.try_get_value().flatten() == Some(handle) {
                            pending_frame.set_value(None);
                            abandon_frame(animation_store, is_animating);
                        }
                    });
                }
            }
            // Scheduling failed: don't report the animation as forever in-flight.
            Err(_) => abandon_frame(animation_store, is_animating),
        }
    }
}

/// The measurement frame never ran: drop the frozen transform left by the
/// stopped previous animation and clear in-flight state.
fn abandon_frame(
    animation_store: StoredValue<Option<FlipAnimation>, LocalStorage>,
    is_animating: RwSignal<bool>,
) {
    if let Some(previous) = animation_store.try_update_value(Option::take).flatten() {
        restore_inline_styles(&previous.element, &previous.inline_styles);
    }
    is_animating.try_set(false);
}
