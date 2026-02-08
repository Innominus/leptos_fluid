use std::borrow::Cow;
use std::rc::Rc;

use leptos::prelude::*;
use leptos_fluid_web::{
    animate_with_waapi, animation_cancel, animation_commit_styles, animation_set_onfinish,
    computed_style, element_set_active_animation, html_style, keyframes_from_two,
    object_from_str_pairs, parse_js_f64, waapi_options,
};

use crate::{FluidSignal, FluidStyle, Transition};

use web_sys::wasm_bindgen::JsCast;
use web_sys::wasm_bindgen::closure::Closure;
use web_sys::{Animation, CssStyleDeclaration, Element};

/// Node reference type used by motion elements.
pub type FluidNodeRef = NodeRef<leptos::html::Custom<&'static str>>;

type StyleProps = Vec<(Cow<'static, str>, String)>;

#[derive(Clone)]
struct ActiveAnimation {
    animation: Animation,
    keys: Rc<Vec<String>>,
    _on_finish: Rc<Closure<dyn FnMut()>>,
}

#[derive(Clone)]
struct TransitionRuntime {
    duration_ms: u32,
    delay_ms: u32,
    easing: String,
}

fn apply_style(element: &Element, style: &FluidStyle) {
    let Some(style_decl) = html_style(element) else {
        return;
    };
    style.apply_to(&style_decl);
}

fn apply_props(element: &Element, props: &StyleProps) {
    let Some(style_decl) = html_style(element) else {
        return;
    };
    for (key, value) in props {
        let _ = style_decl.set_property(key.as_ref(), value);
    }
}

fn apply_owned_props(element: &Element, props: &[(String, String)]) {
    let Some(style_decl) = html_style(element) else {
        return;
    };
    for (key, value) in props {
        let _ = style_decl.set_property(key, value);
    }
}

fn push_keyframe_prop(props: &mut Vec<(String, String)>, key: &str, value: &str) {
    if let Some((_, existing)) = props
        .iter_mut()
        .find(|(existing_key, _)| existing_key == key)
    {
        existing.clear();
        existing.push_str(value);
        return;
    }
    props.push((key.to_string(), value.to_string()));
}

fn keyframe_property_name(css_key: &str) -> String {
    if css_key.is_empty() || css_key.starts_with("--") || !css_key.contains('-') {
        return css_key.to_string();
    }

    // WAAPI keyframe objects expect camelCase for hyphenated CSS property names.
    let mut out = String::with_capacity(css_key.len());
    let mut uppercase_next = false;
    for ch in css_key.chars() {
        if ch == '-' {
            uppercase_next = true;
            continue;
        }
        if uppercase_next {
            out.extend(ch.to_uppercase());
            uppercase_next = false;
        } else {
            out.push(ch);
        }
    }
    out
}

fn normalize_transform_value(value: String) -> String {
    if value.trim().is_empty() || value.trim() == "none" {
        // Using an explicit identity matrix avoids browser-dependent interpolation
        // quirks when animating between "none" and transform functions.
        return "matrix(1, 0, 0, 1, 0, 0)".to_string();
    }
    value
}

fn read_computed_animation_value(computed: &CssStyleDeclaration, key: &str) -> String {
    if key == "border-color" {
        return computed
            .get_property_value("border-top-color")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| computed.get_property_value(key).ok())
            .unwrap_or_default();
    }
    computed.get_property_value(key).unwrap_or_default()
}

fn read_style_or_computed_value(
    style: &CssStyleDeclaration,
    computed: &CssStyleDeclaration,
    key: &str,
) -> String {
    // Prefer inline styles first to preserve author intent when present.
    let inline_value = style
        .get_property_value(key)
        .ok()
        .map(|value| value.trim().to_string())
        .unwrap_or_default();
    if !inline_value.is_empty() {
        return inline_value;
    }
    read_computed_animation_value(computed, key)
}

fn split_animation_props(
    style: &FluidStyle,
    transition: &Transition,
) -> (StyleProps, StyleProps, TransitionRuntime) {
    let mut animated = Vec::new();
    let mut immediate = Vec::new();
    let has_excluded = !transition.excluded_properties.is_empty();

    let mut runtime = TransitionRuntime {
        duration_ms: transition.duration_ms,
        delay_ms: transition.delay_ms,
        easing: transition.easing_string().to_string(),
    };

    for (key, value) in style.to_props() {
        if key.as_ref() == "transition" {
            // Allow per-style runtime overrides (for example timeline-driven styles
            // that want one-off timing different from the component default).
            if let Some(parsed) = parse_transition_override(&value) {
                runtime = parsed;
            }
            continue;
        }
        if has_excluded
            && transition
                .excluded_properties
                .iter()
                .any(|excluded| excluded.as_ref() == key.as_ref())
        {
            immediate.push((key, value));
        } else {
            animated.push((key, value));
        }
    }

    (animated, immediate, runtime)
}

fn parse_transition_override(value: &str) -> Option<TransitionRuntime> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if value == "none" {
        // Explicitly short-circuit to "no animation" semantics.
        return Some(TransitionRuntime {
            duration_ms: 0,
            delay_ms: 0,
            easing: "linear".to_string(),
        });
    }

    let rest = value.strip_prefix("all ")?;
    let (duration_raw, rest) = rest.split_once(' ')?;
    let duration_ms = parse_time_token(duration_raw)?;
    let (easing_raw, delay_ms) = if let Some((easing, delay_raw)) = rest.rsplit_once(' ') {
        if let Some(delay_ms) = parse_time_token(delay_raw) {
            (easing.trim(), delay_ms)
        } else {
            (rest.trim(), 0)
        }
    } else {
        (rest.trim(), 0)
    };
    if easing_raw.is_empty() {
        return None;
    }

    Some(TransitionRuntime {
        duration_ms,
        delay_ms,
        easing: easing_raw.to_string(),
    })
}

fn parse_time_token(token: &str) -> Option<u32> {
    let token = token.trim();
    if let Some(raw) = token.strip_suffix("ms") {
        return parse_js_f64(raw.trim()).map(|value| value.max(0.0).round() as u32);
    }
    if let Some(raw) = token.strip_suffix('s') {
        return parse_js_f64(raw.trim()).map(|value| {
            let ms = value.max(0.0) * 1000.0;
            ms.round() as u32
        });
    }
    None
}

fn freeze_computed_values(
    element: &Element,
    keys: &[String],
    prefer_inline: bool,
) -> Vec<(String, String)> {
    if keys.is_empty() {
        return Vec::new();
    }
    let Some(style_decl) = html_style(element) else {
        return Vec::new();
    };
    let Some(computed) = computed_style(element) else {
        return Vec::new();
    };

    let mut frozen = Vec::with_capacity(keys.len());
    for key in keys {
        // On interruption we need a stable "from" frame. Reading computed values and
        // writing them inline prevents a snap when the previous WAAPI animation is canceled.
        let mut value = if prefer_inline {
            read_style_or_computed_value(&style_decl, &computed, key)
        } else {
            read_computed_animation_value(&computed, key)
        };
        if key == "transform" {
            value = normalize_transform_value(value);
        }
        if value.trim().is_empty() {
            continue;
        }
        let _ = style_decl.set_property(key, value.trim());
        frozen.push((key.clone(), value));
    }
    frozen
}

fn snapshot_value(snapshot: &[(String, String)], key: &str) -> Option<String> {
    snapshot
        .iter()
        .find(|(snapshot_key, _)| snapshot_key == key)
        .map(|(_, value)| value.clone())
}

fn cancel_active_animation(
    element: &Element,
    active_animation: StoredValue<Option<ActiveAnimation>, LocalStorage>,
) -> Vec<(String, String)> {
    let Some(active) = active_animation.get_value() else {
        return Vec::new();
    };
    // `commitStyles()` preserves in-flight interpolated values where supported.
    // Engines without it still work via computed-style freezing below.
    let committed = animation_commit_styles(&active.animation);
    let frozen = freeze_computed_values(element, active.keys.as_ref(), committed);
    animation_set_onfinish(&active.animation, None);
    animation_cancel(&active.animation);
    element_set_active_animation(element, None);
    active_animation.set_value(None);
    frozen
}

fn animate_to(
    element: &Element,
    to: &FluidStyle,
    transition: &Transition,
    active_animation: StoredValue<Option<ActiveAnimation>, LocalStorage>,
    animation_generation: StoredValue<u32, LocalStorage>,
) {
    let generation = animation_generation.get_value().wrapping_add(1);
    animation_generation.set_value(generation);

    let (animated_props, immediate_props, runtime) = split_animation_props(to, transition);
    let mut final_props = Vec::with_capacity(immediate_props.len() + animated_props.len());
    for (key, value) in immediate_props.iter().chain(animated_props.iter()) {
        final_props.push((key.as_ref().to_string(), value.clone()));
    }

    let snapshot = cancel_active_animation(element, active_animation);
    apply_props(element, &immediate_props);

    if animated_props.is_empty() {
        // Nothing to animate; immediate props are already applied above.
        active_animation.set_value(None);
        return;
    }

    if runtime.duration_ms == 0 && runtime.delay_ms == 0 {
        // Fast path for "no transition" updates.
        apply_props(element, &animated_props);
        active_animation.set_value(None);
        return;
    }

    let computed = computed_style(element);
    if computed.is_none() && snapshot.is_empty() {
        // Without either a computed baseline or an interruption snapshot, we cannot
        // construct meaningful keyframes, so apply final values directly.
        apply_props(element, &animated_props);
        active_animation.set_value(None);
        return;
    }

    let mut from_props = Vec::with_capacity(animated_props.len());
    let mut to_props = Vec::with_capacity(animated_props.len());
    let mut animated_keys = Vec::with_capacity(animated_props.len());
    for (css_key, to_value) in &animated_props {
        let css_key = css_key.as_ref();
        let mut from_value = snapshot_value(&snapshot, css_key).unwrap_or_else(|| {
            computed
                .as_ref()
                .map(|style| read_computed_animation_value(style, css_key))
                .unwrap_or_default()
        });
        let mut to_value = to_value.clone();
        if css_key == "transform" {
            // WAAPI keyframes behave better with explicit matrix values than "none".
            from_value = normalize_transform_value(from_value);
            to_value = normalize_transform_value(to_value);
        }
        let keyframe_key = keyframe_property_name(css_key);
        push_keyframe_prop(&mut from_props, &keyframe_key, &from_value);
        push_keyframe_prop(&mut to_props, &keyframe_key, &to_value);
        animated_keys.push(css_key.to_string());
    }

    let mut frame_from_entries = Vec::with_capacity(from_props.len());
    for (key, value) in &from_props {
        frame_from_entries.push((key.as_str(), value.as_str()));
    }
    let frame_from = object_from_str_pairs(&frame_from_entries);

    let mut frame_to_entries = Vec::with_capacity(to_props.len());
    for (key, value) in &to_props {
        frame_to_entries.push((key.as_str(), value.as_str()));
    }
    let frame_to = object_from_str_pairs(&frame_to_entries);
    let keyframes = keyframes_from_two(&frame_from, &frame_to);

    let animation_options = waapi_options(
        runtime.duration_ms.max(1),
        runtime.delay_ms,
        &runtime.easing,
        "both",
    );
    let Some(animation) = animate_with_waapi(element, &keyframes, &animation_options) else {
        // Missing WAAPI support (or call failure): keep behavior functional by
        // applying end-state styles immediately.
        apply_props(element, &animated_props);
        active_animation.set_value(None);
        return;
    };

    let inner_element = element.clone();
    let inner_final_props = Rc::new(final_props);
    let on_finish = Rc::new(Closure::wrap(Box::new(move || {
        if animation_generation.get_value() != generation {
            // Ignore stale finish callbacks from superseded animations.
            return;
        }
        apply_owned_props(&inner_element, inner_final_props.as_ref());
        element_set_active_animation(&inner_element, None);
        active_animation.set_value(None);
    }) as Box<dyn FnMut()>));
    animation_set_onfinish(&animation, Some(on_finish.as_ref().as_ref()));
    element_set_active_animation(element, Some(&animation));

    active_animation.set_value(Some(ActiveAnimation {
        animation,
        keys: Rc::new(animated_keys),
        _on_finish: on_finish,
    }));
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn fluid_element_view(
    tag: &'static str,
    initial: FluidStyle,
    animate: FluidSignal<FluidStyle>,
    transition: Transition,
    reset: Signal<u32>,
    while_hover: Option<FluidStyle>,
    while_tap: Option<FluidStyle>,
    class: FluidSignal<String>,
    style: FluidSignal<String>,
    node_ref: FluidNodeRef,
    content: AnyView,
) -> AnyView {
    let is_hovered = RwSignal::new(false);
    let is_pressed = RwSignal::new(false);

    let base_style = StoredValue::new(initial.clone());
    let transition_store = StoredValue::new(transition);
    let initialized = StoredValue::new(false);
    let reset_last = StoredValue::new(reset.get_untracked());
    let active_animation: StoredValue<Option<ActiveAnimation>, LocalStorage> =
        StoredValue::new_local(None);
    let animation_generation: StoredValue<u32, LocalStorage> = StoredValue::new_local(0);
    let last_element: StoredValue<Option<Element>, LocalStorage> = StoredValue::new_local(None);

    Effect::new({
        let animate = animate.clone();
        move || {
            let reset_value = reset.get();
            if reset_value != reset_last.get_value() {
                reset_last.set_value(reset_value);
                initialized.set_value(false);
            }
            let Some(element) = node_ref.get() else {
                return;
            };
            let element: Element = element.unchecked_into();
            let element_changed = match last_element.get_value() {
                Some(prev) => prev != element,
                None => true,
            };
            if element_changed {
                if let Some(prev) = last_element.get_value() {
                    cancel_active_animation(&prev, active_animation);
                }
                last_element.set_value(Some(element.clone()));
                initialized.set_value(false);
            }
            if initialized.get_value() {
                return;
            }
            cancel_active_animation(&element, active_animation);
            if !initial.is_empty() {
                apply_style(&element, &initial);
            }

            let target = animate.get();
            let next_base = if target.is_empty() {
                initial.clone()
            } else {
                target.clone()
            };
            base_style.set_value(next_base);
            if !target.is_empty() {
                // Defer initial animation by one frame so the browser commits initial
                // inline styles first; this prevents mount-time jumps.
                let transition = transition_store.get_value();
                request_animation_frame(move || {
                    animate_to(
                        &element,
                        &target,
                        &transition,
                        active_animation,
                        animation_generation,
                    );
                });
            }

            initialized.set_value(true);
        }
    });

    Effect::new(move || {
        let target = animate.get();

        if !initialized.get_value() {
            return;
        }

        if target.is_empty() {
            return;
        }
        base_style.set_value(target.clone());

        if is_hovered.get_untracked() || is_pressed.get_untracked() {
            return;
        }

        let Some(element) = node_ref.get_untracked() else {
            return;
        };
        let element: Element = element.unchecked_into();
        let transition = transition_store.get_value();
        animate_to(
            &element,
            &target,
            &transition,
            active_animation,
            animation_generation,
        );
    });

    let on_pointerenter = {
        let while_hover = while_hover.clone();
        move |_| {
            let Some(hover_style) = while_hover.clone() else {
                return;
            };
            let Some(element) = node_ref.get_untracked() else {
                return;
            };
            is_hovered.set(true);
            let element: Element = element.unchecked_into();
            let transition = transition_store.get_value();
            animate_to(
                &element,
                &hover_style,
                &transition,
                active_animation,
                animation_generation,
            );
        }
    };

    let on_pointerleave = {
        move |_| {
            is_hovered.set(false);
            if is_pressed.get_untracked() {
                is_pressed.set(false);
            }

            let Some(element) = node_ref.get_untracked() else {
                return;
            };
            let element: Element = element.unchecked_into();
            let transition = transition_store.get_value();
            let target = base_style.get_value();

            animate_to(
                &element,
                &target,
                &transition,
                active_animation,
                animation_generation,
            );
        }
    };

    let on_pointerdown = move |_| {
        let Some(tap_style) = while_tap.clone() else {
            return;
        };
        let Some(element) = node_ref.get_untracked() else {
            return;
        };
        is_pressed.set(true);
        let element: Element = element.unchecked_into();
        let transition = transition_store.get_value();
        animate_to(
            &element,
            &tap_style,
            &transition,
            active_animation,
            animation_generation,
        );
    };

    let make_pointer_release = move || {
        let while_hover = while_hover.clone();
        move |_| {
            if !is_pressed.get_untracked() {
                return;
            }
            is_pressed.set(false);
            let Some(element) = node_ref.get_untracked() else {
                return;
            };
            let element: Element = element.unchecked_into();
            let transition = transition_store.get_value();
            let target = if is_hovered.get_untracked() {
                while_hover
                    .clone()
                    .unwrap_or_else(|| base_style.get_value())
            } else {
                base_style.get_value()
            };
            animate_to(
                &element,
                &target,
                &transition,
                active_animation,
                animation_generation,
            );
        }
    };

    let on_pointerup = make_pointer_release();
    let on_pointercancel = make_pointer_release();

    leptos::html::custom(tag)
        .node_ref(node_ref)
        .class(move || class.get())
        .style(move || style.get())
        .on(leptos::ev::pointerenter, on_pointerenter)
        .on(leptos::ev::pointerleave, on_pointerleave)
        .on(leptos::ev::pointerdown, on_pointerdown)
        .on(leptos::ev::pointerup, on_pointerup)
        .on(leptos::ev::pointercancel, on_pointercancel)
        .child(content)
        .into_any()
}

/// Motion-enabled element component for arbitrary HTML tags.
#[component]
pub fn FluidElement(
    /// HTML tag name for the underlying element (e.g. "div", "span").
    #[prop(default = "div")]
    tag: &'static str,
    /// Initial animated style applied on mount before the first transition.
    #[prop(default = FluidStyle::default())]
    initial: FluidStyle,
    /// Target style to animate to; updates reactively when this signal changes.
    #[prop(default = FluidSignal::static_value(FluidStyle::default()), into)]
    animate: FluidSignal<FluidStyle>,
    /// Transition settings for animated properties (duration/easing/spring).
    #[prop(default = Transition::default())]
    transition: Transition,
    /// Forces the initial animation to re-run when this counter changes.
    #[prop(default = Signal::derive(|| 0), into)]
    reset: Signal<u32>,
    /// Optional style applied while the pointer is hovering the element.
    #[prop(optional)]
    while_hover: Option<FluidStyle>,
    /// Optional style applied while the pointer is pressed down.
    #[prop(optional)]
    while_tap: Option<FluidStyle>,
    /// Class attribute (static or reactive).
    #[prop(default = FluidSignal::static_value(String::new()), into)]
    class: FluidSignal<String>,
    /// Extra CSS style string (non-animated); useful for layout/base styles.
    #[prop(default = FluidSignal::static_value(String::new()), into)]
    style: FluidSignal<String>,
    /// NodeRef for the underlying element; created automatically if omitted.
    #[prop(optional)]
    node_ref: Option<FluidNodeRef>,
    /// Child view(s) inside the fluid element.
    #[prop(optional)]
    children: Option<Children>,
) -> AnyView {
    let node_ref = node_ref.unwrap_or_default();
    let content = children.map(|c| c()).unwrap_or_else(|| ().into_any());
    fluid_element_view(
        tag,
        initial,
        animate,
        transition,
        reset,
        while_hover,
        while_tap,
        class,
        style,
        node_ref,
        content,
    )
}

macro_rules! fluid_wrapper {
    ($name:ident, $tag:literal) => {
        #[component]
        pub fn $name(
            /// Initial animated style applied on mount before the first transition.
            #[prop(default = FluidStyle::default())]
            initial: FluidStyle,
            /// Target style to animate to; updates reactively when this signal changes.
            #[prop(default = FluidSignal::static_value(FluidStyle::default()), into)]
            animate: FluidSignal<FluidStyle>,
            /// Transition settings for animated properties (duration/easing/spring).
            #[prop(default = Transition::default())]
            transition: Transition,
            /// Forces the initial animation to re-run when this counter changes.
            #[prop(default = Signal::derive(|| 0), into)]
            reset: Signal<u32>,
            /// Optional style applied while the pointer is hovering the element.
            #[prop(optional)]
            while_hover: Option<FluidStyle>,
            /// Optional style applied while the pointer is pressed down.
            #[prop(optional)]
            while_tap: Option<FluidStyle>,
            /// Class attribute (static or reactive).
            #[prop(default = FluidSignal::static_value(String::new()), into)]
            class: FluidSignal<String>,
            /// Extra CSS style string (non-animated); useful for layout/base styles.
            #[prop(default = FluidSignal::static_value(String::new()), into)]
            style: FluidSignal<String>,
            /// NodeRef for the underlying element; created automatically if omitted.
            #[prop(optional)]
            node_ref: Option<FluidNodeRef>,
            /// Child view(s) inside the fluid element.
            #[prop(optional)]
            children: Option<Children>,
        ) -> AnyView {
            let node_ref = node_ref.unwrap_or_default();
            let content = children.map(|c| c()).unwrap_or_else(|| ().into_any());
            fluid_element_view(
                $tag,
                initial,
                animate,
                transition,
                reset,
                while_hover,
                while_tap,
                class,
                style,
                node_ref,
                content,
            )
        }
    };
}

fluid_wrapper!(FluidDiv, "div");
fluid_wrapper!(FluidSpan, "span");
fluid_wrapper!(FluidButton, "button");
