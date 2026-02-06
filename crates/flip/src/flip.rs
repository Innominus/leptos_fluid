use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use web_sys::{
    wasm_bindgen::{prelude::Closure, JsCast},
    Animation, CssStyleDeclaration, Element, HtmlElement, KeyframeAnimationOptions, NodeList,
};

const LINEAR: &str = "linear(\n    0, 0.009, 0.035 2.1%, 0.141, 0.281 6.7%, 0.723 12.9%, 0.938 16.7%, 1.017,\n    1.077, 1.121, 1.149 24.3%, 1.159, 1.163, 1.161, 1.154 29.9%, 1.129 32.8%,\n    1.051 39.6%, 1.017 43.1%, 0.991, 0.977 51%, 0.974 53.8%, 0.975 57.1%,\n    0.997 69.8%, 1.003 76.9%, 1.004 83.8%, 1\n)";

const EASE_IN_OUT: &str = "cubic-bezier(0.83, 0, 0.17, 1)";

#[derive(Clone, Copy)]
pub struct Flip {
    id_selector: StoredValue<String>,
    is_animating: RwSignal<bool>,
    options: FlipOptions,
    animation: StoredValue<Option<FlipAnimation>, LocalStorage>,
}

impl Flip {
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

    pub fn animate<F>(&self, mut animator_fn: F)
    where
        F: FnMut() + Send + Sync + 'static,
    {
        let is_animating = self.is_animating;
        let mut carried_inline_styles: Option<InlineStyles> = None;

        if self.is_animating.get_untracked() {
            if let Some(animation_state) = self.animation.get_value() {
                apply_computed_transform(&animation_state.element);
                animation_state.animation.cancel();
                carried_inline_styles = Some(animation_state.inline_styles.clone());
            }
        }

        let (_el, from_values) = self.measure(None);

        animator_fn();
        is_animating.set(true);

        let inner_options = self.options;
        let inner_animation = self.animation;
        let carried_inline_styles = carried_inline_styles.clone();

        let inner_self = self.clone();
        request_animation_frame(move || {
            let (el, _) = inner_self.measure(None);
            if let Some(inline_styles) = carried_inline_styles.as_ref() {
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
        let animation = run_flip_animation(element, from, to, options, move || {
            is_animating.set(false);
        });
        animation_store.set_value(Some(animation));
    }

    pub fn rect(element: Element) -> (Element, FlipValues) {
        let rect = element.get_bounding_client_rect();

        (
            element,
            FlipValues {
                left: rect.left(),
                top: rect.top(),
                width: rect.width(),
                height: rect.height(),
            },
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct KeyFrame {
    transform: String,
}

#[derive(Debug, Clone)]
pub struct FlipValues {
    left: f64,
    top: f64,
    width: f64,
    height: f64,
}

#[derive(Clone)]
struct InlineStyles {
    transform: String,
    transform_origin: String,
    will_change: String,
}

#[derive(Clone)]
struct FlipAnimation {
    animation: Animation,
    element: Element,
    inline_styles: InlineStyles,
}

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

    pub fn animate<F>(&self, mut animator_fn: F)
    where
        F: FnMut() + Send + Sync + 'static,
    {
        let is_animating = self.is_animating;
        let carried_inline = stop_group_animations(self.animations);
        let from_values = self.snapshot_values();

        animator_fn();
        is_animating.set(true);

        let selector = self.selector;
        let options = self.options;
        let animations_store = self.animations;

        request_animation_frame(move || {
            let elements = selector.with_value(|value| query_elements(value));
            for element in &elements {
                if let Some(key) = element_key(element) {
                    if let Some(inline) = carried_inline.get(&key) {
                        restore_inline_styles(element, inline);
                    }
                }
            }

            let to_items = snapshot_elements(elements);
            let remaining = Rc::new(Cell::new(0usize));
            let mut new_animations = Vec::new();

            for (index, to_item) in to_items.into_iter().enumerate() {
                let Some(from_item_values) = from_values.get(&to_item.key) else {
                    continue;
                };
                if !has_flip_delta(from_item_values, &to_item.values) {
                    continue;
                }

                remaining.set(remaining.get() + 1);
                let remaining_inner = remaining.clone();
                let is_animating_inner = is_animating;
                let item_options = options.with_stagger_index(index);
                let animation = run_flip_animation(
                    to_item.element,
                    from_item_values.clone(),
                    to_item.values,
                    item_options,
                    move || {
                        let next = remaining_inner.get().saturating_sub(1);
                        remaining_inner.set(next);
                        if next == 0 {
                            is_animating_inner.set(false);
                        }
                    },
                );
                new_animations.push(animation);
            }

            if remaining.get() == 0 {
                is_animating.set(false);
            }

            animations_store.set_value(new_animations);
        });
    }

    fn snapshot_values(&self) -> HashMap<String, FlipValues> {
        let elements = self.selector.with_value(|value| query_elements(value));
        let mut values = HashMap::new();
        for item in snapshot_elements(elements) {
            values.insert(item.key, item.values);
        }
        values
    }
}

const FLIP_DELTA_EPSILON: f64 = 0.1;

fn stop_group_animations(
    animations_store: StoredValue<Vec<FlipAnimation>, LocalStorage>,
) -> HashMap<String, InlineStyles> {
    let active_animations = animations_store.get_value();
    let mut carried_inline = HashMap::new();

    for animation in active_animations {
        apply_computed_transform(&animation.element);
        animation.animation.cancel();
        if let Some(key) = element_key(&animation.element) {
            carried_inline.insert(key, animation.inline_styles.clone());
        }
    }

    animations_store.set_value(Vec::new());
    carried_inline
}

fn has_flip_delta(from: &FlipValues, to: &FlipValues) -> bool {
    (from.left - to.left).abs() > FLIP_DELTA_EPSILON
        || (from.top - to.top).abs() > FLIP_DELTA_EPSILON
        || (from.width - to.width).abs() > FLIP_DELTA_EPSILON
        || (from.height - to.height).abs() > FLIP_DELTA_EPSILON
}

#[derive(Clone)]
struct FlipItem {
    key: String,
    element: Element,
    values: FlipValues,
}

#[derive(Clone, Copy, Default)]
pub struct FlipOptions {
    pub duration: usize,
    pub delay: usize,
    pub stagger: usize,
    pub easing: Easing,
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

#[derive(Clone, Copy, Default)]
pub enum Easing {
    #[default]
    Linear,
    EaseInOut,
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

fn html_style(element: &Element) -> Option<CssStyleDeclaration> {
    element.dyn_ref::<HtmlElement>().map(|el| el.style())
}

fn capture_inline_styles(element: &Element) -> InlineStyles {
    let Some(style) = html_style(element) else {
        return InlineStyles {
            transform: String::new(),
            transform_origin: String::new(),
            will_change: String::new(),
        };
    };
    InlineStyles {
        transform: style.get_property_value("transform").unwrap_or_default(),
        transform_origin: style
            .get_property_value("transform-origin")
            .unwrap_or_default(),
        will_change: style.get_property_value("will-change").unwrap_or_default(),
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
}

fn apply_computed_transform(element: &Element) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(Some(computed)) = window.get_computed_style(element) else {
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

fn restore_inline_property(style: &CssStyleDeclaration, property: &str, value: &str) {
    if value.is_empty() {
        let _ = style.remove_property(property);
    } else {
        let _ = style.set_property(property, value);
    }
}

fn query_elements(selector: &str) -> Vec<Element> {
    let Ok(list) = document().query_selector_all(selector) else {
        return Vec::new();
    };
    node_list_to_elements(list)
}

fn node_list_to_elements(list: NodeList) -> Vec<Element> {
    let mut elements = Vec::new();
    let length = list.length();
    for index in 0..length {
        let Some(node) = list.get(index) else {
            continue;
        };
        let Ok(element) = node.dyn_into::<Element>() else {
            continue;
        };
        elements.push(element);
    }
    elements
}

fn snapshot_elements(elements: Vec<Element>) -> Vec<FlipItem> {
    elements
        .into_iter()
        .enumerate()
        .map(|(index, element)| {
            // Prefer explicit identity markers, but fall back to selector order so
            // group FLIP still works when authors forget to add data/id keys.
            let key = element_key(&element).unwrap_or_else(|| format!("__flip-index-{}", index));
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

fn run_flip_animation<F>(
    element: Element,
    from: FlipValues,
    to: FlipValues,
    options: FlipOptions,
    mut on_finish: F,
) -> FlipAnimation
where
    F: FnMut() + 'static,
{
    let dx = from.left - to.left;
    let dy = from.top - to.top;
    let scale_x = safe_div(from.width, to.width);
    let scale_y = safe_div(from.height, to.height);

    let transform_from = format!(
        "translate({}px, {}px) scale({}, {})",
        dx, dy, scale_x, scale_y
    );
    let transform_to = "translate(0px, 0px) scale(1, 1)".to_string();

    let inline_styles = capture_inline_styles(&element);
    apply_inline_transform(&element, &transform_from);

    let keyframes = vec![
        KeyFrame {
            transform: transform_from,
        },
        KeyFrame {
            transform: transform_to,
        },
    ];

    let keyframes_js = serde_wasm_bindgen::to_value(&keyframes).unwrap();
    let animation_options = KeyframeAnimationOptions::new();
    let duration = options.duration.max(1) as f64;
    animation_options.set_duration(&duration.into());
    animation_options.set_delay(options.delay as f64);
    animation_options.set_easing(options.easing.get_easing_fn());
    animation_options.set_fill(web_sys::FillMode::Backwards);

    let inner_element = element.clone();
    let inner_inline_styles = inline_styles.clone();
    let closure = Closure::wrap(Box::new(move |_: web_sys::AnimationEvent| {
        restore_inline_styles(&inner_element, &inner_inline_styles);
        on_finish();
    }) as Box<dyn FnMut(_)>);

    let animation = element
        .animate_with_keyframe_animation_options(Some(&keyframes_js.into()), &animation_options);

    animation.set_onfinish(Some(closure.as_ref().unchecked_ref()));

    closure.into_js_value();

    FlipAnimation {
        animation,
        element,
        inline_styles,
    }
}

fn safe_div(numerator: f64, denominator: f64) -> f64 {
    if denominator.abs() <= f64::EPSILON {
        return 1.0;
    }
    let value = numerator / denominator;
    if value.is_finite() {
        value
    } else {
        1.0
    }
}
