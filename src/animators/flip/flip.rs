use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use web_sys::{
    wasm_bindgen::{prelude::Closure, JsCast},
    Animation, Element, KeyframeAnimationOptions,
};

const LINEAR: &str = "linear(
    0, 0.009, 0.035 2.1%, 0.141, 0.281 6.7%, 0.723 12.9%, 0.938 16.7%, 1.017,
    1.077, 1.121, 1.149 24.3%, 1.159, 1.163, 1.161, 1.154 29.9%, 1.129 32.8%,
    1.051 39.6%, 1.017 43.1%, 0.991, 0.977 51%, 0.974 53.8%, 0.975 57.1%,
    0.997 69.8%, 1.003 76.9%, 1.004 83.8%, 1
)";

const EASE_IN_OUT: &str = "cubic-bezier(0.83, 0, 0.17, 1)";

#[derive(Clone, Copy)]
pub struct Flip {
    id_selector: StoredValue<String>,
    is_animating: RwSignal<bool>,
    options: FlipOptions,
    animation: StoredValue<Option<Animation>, LocalStorage>,
}

impl Flip {
    pub fn new(id_selector: String) -> Self {
        let new_self = Self {
            id_selector: StoredValue::new(id_selector),
            is_animating: RwSignal::new(false),
            options: FlipOptions::new(),
            animation: StoredValue::new_local(None),
        };

        new_self
    }

    pub fn new_with_options(id_selector: String, options: FlipOptions) -> Self {
        let new_self = Self {
            id_selector: StoredValue::new(id_selector),
            is_animating: RwSignal::new(false),
            options,
            animation: StoredValue::new_local(None),
        };

        new_self
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
        let (el, from_values) = self.measure(None);

        if self.is_animating.get_untracked() {
            self.animation.get_value().unwrap().cancel();
            // TODO: probs need to hold onto the old element that's attached to the animation
            el.style("");
        }

        animator_fn();
        is_animating.set(true);

        let inner_options = self.options;
        let inner_animation = self.animation;

        let inner_self = self.clone();
        request_animation_frame(move || {
            let (el, to_values) = inner_self.measure(None);

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

    pub fn invert(
        element: Element,
        from: FlipValues,
        to: FlipValues,
        options: FlipOptions,
        is_animating: RwSignal<bool>,
        animation_store: StoredValue<Option<Animation>, LocalStorage>,
    ) {
        let dx = from.left - to.left;
        let dy = from.top - to.top;

        element.style(format!(
            "position: absolute; top: {}px; left: {}px; width: {}px; height: {}px;",
            to.top, to.left, to.width, to.height
        ));

        let keyframes = vec![
            KeyFrame {
                transform: format!("translate({}px, {}px)", dx, dy),
                height: format!("{}px", from.height),
                width: format!("{}px", from.width),
            },
            KeyFrame {
                transform: "translate(0px, 0px)".to_string(),
                height: format!("{}px", to.height),
                width: format!("{}px", to.width),
            },
        ];

        let keyframes_js = serde_wasm_bindgen::to_value(&keyframes).unwrap();
        let animation_options = KeyframeAnimationOptions::new();
        animation_options.set_duration(&options.duration.into());
        animation_options.set_delay(options.delay as f64);
        animation_options.set_easing(&options.easing.get_easing_fn());
        animation_options.set_fill(web_sys::FillMode::Backwards);

        let inner_element = element.clone();
        let closure = Closure::wrap(Box::new(move |_: web_sys::AnimationEvent| {
            is_animating.set(false);
            inner_element.style("");
        }) as Box<dyn FnMut(_)>);

        let animation = element.animate_with_keyframe_animation_options(
            Some(&keyframes_js.into()),
            &animation_options,
        );

        animation.set_onfinish(Some(closure.as_ref().unchecked_ref()));

        closure.into_js_value();

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
    height: String,
    width: String,
}

#[derive(Debug)]
pub struct FlipValues {
    left: f64,
    top: f64,
    width: f64,
    height: f64,
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
