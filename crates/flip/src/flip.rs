use self::corrections::{
    read_border_radius_target, run_border_radius_correction, run_scale_correction_animations,
};
use leptos::prelude::*;
use leptos_fluid_web::{
    animate_with_waapi, animation_cancel, animation_set_onfinish, computed_style, css_push_number,
    css_push_px, html_style, keyframes_from_two, node_list_to_elements, object_from_str_pairs,
    restore_inline_property, safe_f64_ratio, waapi_options,
};
use std::cell::Cell;
use std::rc::Rc;
use web_sys::{Animation, Element, wasm_bindgen::prelude::Closure};

mod corrections;

const LINEAR: &str = "linear(\n    0, 0.009, 0.035 2.1%, 0.141, 0.281 6.7%, 0.723 12.9%, 0.938 16.7%, 1.017,\n    1.077, 1.121, 1.149 24.3%, 1.159, 1.163, 1.161, 1.154 29.9%, 1.129 32.8%,\n    1.051 39.6%, 1.017 43.1%, 0.991, 0.977 51%, 0.974 53.8%, 0.975 57.1%,\n    0.997 69.8%, 1.003 76.9%, 1.004 83.8%, 1\n)";

const EASE_IN_OUT: &str = "cubic-bezier(0.83, 0, 0.17, 1)";

/// Single-element FLIP animator identified by DOM id.
#[derive(Clone, Copy)]
pub struct Flip {
    id_selector: StoredValue<String>,
    is_animating: RwSignal<bool>,
    options: FlipOptions,
    animation: StoredValue<Option<FlipAnimation>, LocalStorage>,
}

impl Flip {
    /// `id_selector` must be the raw id value (for example `"card-a"`), not `"#card-a"`.
    pub fn new(id_selector: String) -> Self {
        Self {
            id_selector: StoredValue::new(id_selector),
            is_animating: RwSignal::new(false),
            options: FlipOptions::new(),
            animation: StoredValue::new_local(None),
        }
    }

    pub fn new_with_options(id_selector: String, options: FlipOptions) -> Self {
        Self {
            id_selector: StoredValue::new(id_selector),
            is_animating: RwSignal::new(false),
            options,
            animation: StoredValue::new_local(None),
        }
    }

    pub fn set_id_selector(&mut self, id_selector: String) {
        self.id_selector.set_value(id_selector);
    }

    pub fn set_options(&mut self, options: FlipOptions) {
        self.options = options;
    }

    pub fn get_is_animating_signal(&self) -> Signal<bool> {
        self.is_animating.into()
    }

    /// Runs a FLIP capture around a state mutation closure.
    ///
    /// The closure should perform the layout-changing updates.
    pub fn animate<F>(&self, animator_fn: F)
    where
        F: FnMut() + 'static,
    {
        self.animate_dyn(Box::new(animator_fn));
    }

    fn animate_dyn(&self, mut animator_fn: Box<dyn FnMut() + 'static>) {
        let is_animating = self.is_animating;
        let mut carried_inline_styles: Option<InlineStyles> = None;

        if let Some(animation_state) = self.animation.get_value() {
            // Preserve current visual progress before starting a new FLIP run.
            stop_flip_animation_state(&animation_state);
            carried_inline_styles = Some(animation_state.inline_styles);
        }

        let (_el, from_values) = self.measure(None);

        animator_fn();
        is_animating.set(true);

        let inner_options = self.options;
        let inner_animation = self.animation;
        let carried_inline_styles = carried_inline_styles;

        let inner_self = *self;
        request_animation_frame(move || {
            // Measure on next frame so layout mutations from `animator_fn` are committed.
            let (el, _) = inner_self.measure(None);
            if let Some(inline_styles) = carried_inline_styles.as_ref() {
                // Carry inline transform state across interruptions.
                restore_inline_styles(&el, inline_styles);
            }
            let (el, to_values) = inner_self.measure(Some(el));

            Self::invert(
                el,
                from_values,
                to_values,
                inner_options,
                is_animating,
                inner_animation,
            );
        })
    }

    pub fn measure(&self, element: Option<Element>) -> (Element, FlipValues) {
        if let Some(el) = element {
            return Self::rect(el);
        }

        let element = self
            .id_selector
            .with_value(|val| document().get_element_by_id(val).unwrap());

        Self::rect(element)
    }

    fn invert(
        element: Element,
        from: FlipValues,
        to: FlipValues,
        options: FlipOptions,
        is_animating: RwSignal<bool>,
        animation_store: StoredValue<Option<FlipAnimation>, LocalStorage>,
    ) {
        if !has_flip_delta_with_size(&from, &to, options.scale_mode.uses_scale()) {
            is_animating.set(false);
            animation_store.set_value(None);
            return;
        }
        let on_finish: Rc<dyn Fn()> = Rc::new(move || {
            is_animating.set(false);
        });
        let animation = run_flip_animation(element, from, to, options, on_finish);
        animation_store.set_value(Some(animation));
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
}

/// Opaque layout snapshot used internally by the FLIP pipeline.
#[derive(Debug, Clone, Copy)]
pub struct FlipValues {
    left: f64,
    top: f64,
    width: f64,
    height: f64,
    border_radius: Option<BorderRadiusTarget>,
}

#[derive(Clone)]
struct InlineStyles {
    transform: String,
    transform_origin: String,
    will_change: String,
    transition: String,
}

#[derive(Clone)]
struct FlipAnimation {
    animation: Option<Animation>,
    element: Element,
    inline_styles: InlineStyles,
    scale_corrections: Rc<Vec<ScaleCorrectionAnimation>>,
    border_radius_correction: Option<BorderRadiusCorrectionAnimation>,
}

#[derive(Clone)]
struct ScaleCorrectionAnimation {
    stop_signal: Rc<Cell<bool>>,
    element: Element,
    inline_styles: Rc<InlineStyles>,
}

#[derive(Clone)]
struct ScaleCorrectionTarget {
    element: Element,
    offset_x: f64,
    offset_y: f64,
}

#[derive(Clone)]
struct BorderRadiusInlineStyles {
    top_left: String,
    top_right: String,
    bottom_right: String,
    bottom_left: String,
}

#[derive(Clone)]
struct BorderRadiusCorrectionAnimation {
    stop_signal: Rc<Cell<bool>>,
    inline_styles: Rc<BorderRadiusInlineStyles>,
}

#[derive(Debug, Clone, Copy)]
struct BorderRadiusTarget {
    top_left: RadiusPair,
    top_right: RadiusPair,
    bottom_right: RadiusPair,
    bottom_left: RadiusPair,
}

#[derive(Debug, Clone, Copy)]
struct RadiusPair {
    x: f64,
    y: f64,
}

/// Multi-element FLIP animator identified by a CSS selector.
#[derive(Clone, Copy)]
pub struct FlipGroup {
    selector: StoredValue<String>,
    is_animating: RwSignal<bool>,
    options: FlipOptions,
    animations: StoredValue<Vec<FlipAnimation>, LocalStorage>,
}

impl FlipGroup {
    pub fn new(selector: String) -> Self {
        Self {
            selector: StoredValue::new(selector),
            is_animating: RwSignal::new(false),
            options: FlipOptions::new(),
            animations: StoredValue::new_local(Vec::new()),
        }
    }

    pub fn new_with_options(selector: String, options: FlipOptions) -> Self {
        Self {
            selector: StoredValue::new(selector),
            is_animating: RwSignal::new(false),
            options,
            animations: StoredValue::new_local(Vec::new()),
        }
    }

    pub fn set_selector(&mut self, selector: String) {
        self.selector.set_value(selector);
    }

    pub fn set_options(&mut self, options: FlipOptions) {
        self.options = options;
    }

    pub fn get_is_animating_signal(&self) -> Signal<bool> {
        self.is_animating.into()
    }

    pub fn animate<F>(&self, animator_fn: F)
    where
        F: FnMut() + 'static,
    {
        self.animate_dyn(Box::new(animator_fn));
    }

    fn animate_dyn(&self, mut animator_fn: Box<dyn FnMut() + 'static>) {
        let is_animating = self.is_animating;
        let carried_inline = stop_group_animations(self.animations);
        let from_values = self.snapshot_values();

        animator_fn();
        is_animating.set(true);

        let selector = self.selector;
        let options = self.options;
        let animations_store = self.animations;

        request_animation_frame(move || {
            // Re-query after mutation and restore interruption styles before measuring "last".
            let elements = selector.with_value(|value| query_elements(value));
            for element in &elements {
                if let Some(key) = element_key(element)
                    && let Some(inline) = find_inline_by_key(&carried_inline, &key)
                {
                    restore_inline_styles(element, inline);
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
            .map(|item| (item.key, item.values))
            .collect()
    }
}

const FLIP_DELTA_EPSILON: f64 = 0.1;

fn stop_group_animations(
    animations_store: StoredValue<Vec<FlipAnimation>, LocalStorage>,
) -> Vec<(String, InlineStyles)> {
    let active_animations = animations_store.get_value();
    let mut carried_inline = Vec::new();

    for animation in active_animations {
        // Stop current group animations and capture inline state keyed by element identity.
        stop_flip_animation_state(&animation);
        if let Some(key) = element_key(&animation.element) {
            carried_inline.push((key, animation.inline_styles.clone()));
        }
    }

    animations_store.set_value(Vec::new());
    carried_inline
}

fn stop_flip_animation_state(animation: &FlipAnimation) {
    // Lock the current computed transform inline before canceling to avoid visual jumps.
    apply_computed_transform(&animation.element);
    if let Some(active) = animation.animation.as_ref() {
        animation_cancel(active);
    }
    if let Some(correction) = animation.border_radius_correction.as_ref() {
        correction.stop_signal.set(true);
        restore_border_radius_inline_styles(&animation.element, correction.inline_styles.as_ref());
    }
    for correction in animation.scale_corrections.iter() {
        correction.stop_signal.set(true);
        restore_inline_styles(&correction.element, correction.inline_styles.as_ref());
    }
}

fn find_inline_by_key<'a>(
    entries: &'a [(String, InlineStyles)],
    key: &str,
) -> Option<&'a InlineStyles> {
    entries
        .iter()
        .find_map(|(entry_key, value)| (entry_key == key).then_some(value))
}

fn find_values_by_key<'a>(
    entries: &'a [(String, FlipValues)],
    key: &str,
) -> Option<&'a FlipValues> {
    entries
        .iter()
        .find_map(|(entry_key, value)| (entry_key == key).then_some(value))
}

fn has_flip_delta_with_size(from: &FlipValues, to: &FlipValues, include_size_delta: bool) -> bool {
    let has_position_delta = (from.left - to.left).abs() > FLIP_DELTA_EPSILON
        || (from.top - to.top).abs() > FLIP_DELTA_EPSILON;
    if has_position_delta {
        return true;
    }
    include_size_delta
        && ((from.width - to.width).abs() > FLIP_DELTA_EPSILON
            || (from.height - to.height).abs() > FLIP_DELTA_EPSILON)
}

#[derive(Clone)]
struct FlipItem {
    key: String,
    element: Element,
    values: FlipValues,
}

/// Runtime options for both `Flip` and `FlipGroup`.
#[derive(Clone, Copy, Default)]
pub struct FlipOptions {
    /// Animation duration in milliseconds.
    pub duration: usize,
    /// Initial delay in milliseconds.
    pub delay: usize,
    /// Per-item delay increment in milliseconds (group mode).
    pub stagger: usize,
    /// Easing curve used by WAAPI.
    pub easing: Easing,
    /// Whether to animate only position or position+size.
    pub scale_mode: ScaleMode,
    /// Optional descendant selector used for inverse-scale correction.
    pub scale_correction_selector: Option<&'static str>,
}

impl FlipOptions {
    pub fn new() -> Self {
        Self {
            duration: 1000,
            ..Self::default()
        }
    }

    fn with_stagger_index(self, index: usize) -> Self {
        let stagger_delay = self.stagger.saturating_mul(index);
        Self {
            delay: self.delay.saturating_add(stagger_delay),
            ..self
        }
    }
}

/// Easing presets for FLIP animations.
#[derive(Clone, Copy, Default)]
pub enum Easing {
    /// Smooth custom `linear(...)` curve tuned for FLIP movement.
    #[default]
    Linear,
    /// Cubic-bezier ease-in-out curve.
    EaseInOut,
    /// Caller-provided CSS easing string.
    Custom(&'static str),
}

impl Easing {
    fn get_easing_fn(&self) -> &'static str {
        match self {
            Easing::Linear => LINEAR,
            Easing::EaseInOut => EASE_IN_OUT,
            Easing::Custom(val) => val,
        }
    }
}

/// Controls whether size deltas participate in FLIP inversion.
#[derive(Clone, Copy, Default)]
pub enum ScaleMode {
    /// Animate position changes only.
    #[default]
    PositionOnly,
    /// Animate both position and size via scale transforms.
    PositionAndScale,
}

impl ScaleMode {
    fn uses_scale(&self) -> bool {
        matches!(self, ScaleMode::PositionAndScale)
    }
}

fn capture_inline_styles(element: &Element) -> InlineStyles {
    let Some(style) = html_style(element) else {
        return InlineStyles {
            transform: String::new(),
            transform_origin: String::new(),
            will_change: String::new(),
            transition: String::new(),
        };
    };
    InlineStyles {
        transform: style.get_property_value("transform").unwrap_or_default(),
        transform_origin: style
            .get_property_value("transform-origin")
            .unwrap_or_default(),
        will_change: style.get_property_value("will-change").unwrap_or_default(),
        transition: style.get_property_value("transition").unwrap_or_default(),
    }
}

fn capture_border_radius_inline_styles(element: &Element) -> BorderRadiusInlineStyles {
    let Some(style) = html_style(element) else {
        return BorderRadiusInlineStyles {
            top_left: String::new(),
            top_right: String::new(),
            bottom_right: String::new(),
            bottom_left: String::new(),
        };
    };
    BorderRadiusInlineStyles {
        top_left: style
            .get_property_value("border-top-left-radius")
            .unwrap_or_default(),
        top_right: style
            .get_property_value("border-top-right-radius")
            .unwrap_or_default(),
        bottom_right: style
            .get_property_value("border-bottom-right-radius")
            .unwrap_or_default(),
        bottom_left: style
            .get_property_value("border-bottom-left-radius")
            .unwrap_or_default(),
    }
}

fn apply_inline_transform(element: &Element, transform: &str) {
    let Some(style) = html_style(element) else {
        return;
    };
    let _ = style.set_property("transform-origin", "0 0");
    let _ = style.set_property("transform", transform);
    let _ = style.set_property("will-change", "transform");
}

fn restore_inline_styles(element: &Element, inline_styles: &InlineStyles) {
    let Some(style) = html_style(element) else {
        return;
    };
    restore_inline_property(&style, "transform", &inline_styles.transform);
    restore_inline_property(&style, "transform-origin", &inline_styles.transform_origin);
    restore_inline_property(&style, "will-change", &inline_styles.will_change);
    restore_inline_property(&style, "transition", &inline_styles.transition);
}

fn restore_border_radius_inline_styles(
    element: &Element,
    inline_styles: &BorderRadiusInlineStyles,
) {
    let Some(style) = html_style(element) else {
        return;
    };
    restore_inline_property(&style, "border-top-left-radius", &inline_styles.top_left);
    restore_inline_property(&style, "border-top-right-radius", &inline_styles.top_right);
    restore_inline_property(
        &style,
        "border-bottom-right-radius",
        &inline_styles.bottom_right,
    );
    restore_inline_property(
        &style,
        "border-bottom-left-radius",
        &inline_styles.bottom_left,
    );
}

fn apply_computed_transform(element: &Element) {
    let Some(computed) = computed_style(element) else {
        return;
    };
    let Ok(transform) = computed.get_property_value("transform") else {
        return;
    };
    let Ok(origin) = computed.get_property_value("transform-origin") else {
        return;
    };
    let Some(style) = html_style(element) else {
        return;
    };
    // Persist the current transform so an interrupted run can continue from this frame.
    let transform_value = if transform.trim().is_empty() {
        "none"
    } else {
        transform.trim()
    };
    let _ = style.set_property("transform", transform_value);
    if !origin.trim().is_empty() {
        let _ = style.set_property("transform-origin", origin.trim());
    }
    let _ = style.set_property("will-change", "transform");
}

fn query_elements(selector: &str) -> Vec<Element> {
    let Ok(list) = document().query_selector_all(selector) else {
        return Vec::new();
    };
    node_list_to_elements(list)
}

fn query_elements_within(root: &Element, selector: &str) -> Vec<Element> {
    let Ok(list) = root.query_selector_all(selector) else {
        return Vec::new();
    };
    node_list_to_elements(list)
}

fn snapshot_elements(elements: Vec<Element>) -> Vec<FlipItem> {
    elements
        .into_iter()
        .enumerate()
        .map(|(index, element)| {
            // Prefer explicit identity markers, but fall back to selector order so
            // group FLIP still works when authors forget to add data/id keys.
            let key = element_key(&element).unwrap_or_else(|| {
                let mut value = String::from("__flip-index-");
                value.push_str(&index.to_string());
                value
            });
            let (_, values) = Flip::rect(element.clone());
            FlipItem {
                key,
                element,
                values,
            }
        })
        .collect()
}

fn element_key(element: &Element) -> Option<String> {
    element
        .get_attribute("data-flip-id")
        .or_else(|| element.get_attribute("id"))
        .filter(|value| !value.is_empty())
}

fn run_flip_animation(
    element: Element,
    from: FlipValues,
    to: FlipValues,
    options: FlipOptions,
    on_finish: Rc<dyn Fn()>,
) -> FlipAnimation {
    let dx = from.left - to.left;
    let dy = from.top - to.top;
    let use_scale = options.scale_mode.uses_scale();
    let scale_x = if use_scale {
        safe_f64_ratio(from.width, to.width)
    } else {
        1.0
    };
    let scale_y = if use_scale {
        safe_f64_ratio(from.height, to.height)
    } else {
        1.0
    };

    let transform_from = if use_scale {
        build_translate_scale_transform(dx, dy, scale_x, scale_y)
    } else {
        build_translate_transform(dx, dy)
    };
    let transform_to = if use_scale {
        "translate(0px, 0px) scale(1, 1)"
    } else {
        "translate(0px, 0px)"
    };

    let inline_styles = capture_inline_styles(&element);
    apply_inline_transform(&element, &transform_from);

    let frame_from = object_from_str_pairs(&[("transform", transform_from.as_str())]);
    let frame_to = object_from_str_pairs(&[("transform", transform_to)]);
    let keyframes = keyframes_from_two(&frame_from, &frame_to);

    let animation_options = waapi_options(
        options.duration.max(1) as u32,
        options.delay as u32,
        options.easing.get_easing_fn(),
        "backwards",
    );
    let animation = animate_with_waapi(&element, &keyframes, &animation_options);

    let border_radius_correction = if use_scale {
        run_border_radius_correction(
            &element,
            from.border_radius,
            to.border_radius,
            scale_x,
            scale_y,
        )
    } else {
        None
    };

    let scale_corrections = if use_scale {
        options
            .scale_correction_selector
            .map(|selector| run_scale_correction_animations(&element, selector, scale_x, scale_y))
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let scale_corrections = Rc::new(scale_corrections);

    let inner_element = element.clone();
    let inner_inline_styles = inline_styles.clone();
    let border_radius_correction_for_finish = border_radius_correction.clone();
    let scale_corrections_for_finish = scale_corrections.clone();
    let on_complete: Rc<dyn Fn()> = Rc::new(move || {
        if let Some(correction) = &border_radius_correction_for_finish {
            correction.stop_signal.set(true);
            restore_border_radius_inline_styles(&inner_element, correction.inline_styles.as_ref());
        }
        for correction in scale_corrections_for_finish.iter() {
            correction.stop_signal.set(true);
            restore_inline_styles(&correction.element, correction.inline_styles.as_ref());
        }
        restore_inline_styles(&inner_element, &inner_inline_styles);
        on_finish();
    });
    if let Some(animation) = animation.as_ref() {
        // Keep callback alive for the lifetime of the active WAAPI animation.
        let on_complete = Closure::wrap(Box::new(move || on_complete()) as Box<dyn FnMut()>);
        animation_set_onfinish(animation, Some(on_complete.as_ref()));
        on_complete.forget();
    } else {
        // Fallback behavior if WAAPI animate call failed/unavailable.
        request_animation_frame(move || on_complete());
    }

    FlipAnimation {
        animation,
        element,
        inline_styles,
        scale_corrections,
        border_radius_correction,
    }
}

fn build_translate_transform(dx: f64, dy: f64) -> String {
    let mut out = String::from("translate(");
    css_push_px(&mut out, dx);
    out.push_str(", ");
    css_push_px(&mut out, dy);
    out.push(')');
    out
}

fn build_translate_scale_transform(dx: f64, dy: f64, scale_x: f64, scale_y: f64) -> String {
    let mut out = build_translate_transform(dx, dy);
    out.push_str(" scale(");
    css_push_number(&mut out, scale_x);
    out.push_str(", ");
    css_push_number(&mut out, scale_y);
    out.push(')');
    out
}
