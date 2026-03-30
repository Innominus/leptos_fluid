use std::borrow::Cow;
#[cfg(feature = "spring")]
use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::{GetValue, LocalStorage, RwSignal, Set, SetValue, StoredValue};
#[cfg(target_arch = "wasm32")]
use leptos_fluid_web::parse_js_f64;
use leptos_fluid_web::{
    animate_with_waapi, animation_cancel, animation_commit_styles, animation_pause, animation_play,
    animation_set_onfinish, computed_style, element_set_active_animation, html_style,
    keyframes_from_two, object_from_str_pairs, waapi_options,
};
use web_sys::wasm_bindgen::closure::Closure;
use web_sys::{Animation, CssStyleDeclaration, Element};

#[cfg(feature = "spring")]
use crate::Spring;
#[cfg(feature = "spring")]
use crate::spring_solver::{SpringState, normalized_dt, step_spring};
#[cfg(feature = "spring")]
use crate::timing::now_ms;
use crate::{FluidStyle, Transition};

type StyleProps = Vec<(Cow<'static, str>, String)>;

#[derive(Clone)]
pub(crate) enum ActiveAnimation {
    Waapi(WaapiAnimation),
    #[cfg(feature = "spring")]
    Spring(SpringAnimation),
}

#[derive(Clone)]
pub(crate) struct WaapiAnimation {
    animation: Animation,
    keys: Rc<Vec<String>>,
    _on_finish: Rc<Closure<dyn FnMut()>>,
}

#[cfg(feature = "spring")]
#[derive(Clone)]
pub(crate) struct SpringAnimation {
    state: Rc<RefCell<SpringAnimationState>>,
}

#[cfg(feature = "spring")]
struct SpringAnimationState {
    element: Element,
    keys: Rc<Vec<String>>,
    final_props: Rc<Vec<(String, String)>>,
    channels: SpringChannels,
    spring: Spring,
    loop_state: SpringLoopState,
    generation: u32,
    active_animation: StoredValue<Option<ActiveAnimation>, LocalStorage>,
    animation_generation: StoredValue<u32, LocalStorage>,
    is_animating: Option<RwSignal<bool>>,
}

#[cfg(feature = "spring")]
#[derive(Clone, Copy, Debug, Default)]
struct SpringLoopState {
    delay_remaining_ms: f64,
    last_time_ms: Option<f64>,
    running: bool,
    paused: bool,
    schedule_id: u32,
}

#[cfg(feature = "spring")]
#[derive(Clone, Copy, Debug, Default)]
struct SpringChannels {
    transform: Option<TransformSpringState>,
    opacity: Option<ScalarSpringState>,
    width: Option<ScalarSpringState>,
    height: Option<ScalarSpringState>,
}

#[cfg(feature = "spring")]
#[derive(Clone, Copy, Debug)]
struct TransformSpringState {
    x: SpringState,
    y: SpringState,
    scale: SpringState,
    rotate: SpringState,
    target: TransformValues,
}

#[cfg(feature = "spring")]
#[derive(Clone, Copy, Debug)]
struct ScalarSpringState {
    state: SpringState,
    target: f64,
}

#[cfg(feature = "spring")]
#[derive(Clone, Copy, Debug, Default)]
struct TransformValues {
    x: f64,
    y: f64,
    scale: f64,
    rotate: f64,
}

#[cfg(feature = "spring")]
#[derive(Clone, Copy, Debug, Default)]
struct ParsedCurrentValues {
    transform: TransformValues,
    opacity: f64,
    width: Option<f64>,
    height: Option<f64>,
}

#[cfg(feature = "spring")]
#[derive(Clone, Copy, Debug, Default)]
struct SpringTargets {
    transform: Option<TransformValues>,
    opacity: Option<f64>,
    width: Option<f64>,
    height: Option<f64>,
}

#[derive(Clone)]
struct TransitionRuntime {
    duration_ms: u32,
    delay_ms: u32,
    easing: String,
}

#[cfg(feature = "spring")]
impl SpringAnimation {
    fn new(
        element: Element,
        current: ParsedCurrentValues,
        targets: SpringTargets,
        spring: Spring,
        delay_ms: u32,
        keys: Rc<Vec<String>>,
        final_props: Rc<Vec<(String, String)>>,
        active_animation: StoredValue<Option<ActiveAnimation>, LocalStorage>,
        animation_generation: StoredValue<u32, LocalStorage>,
        generation: u32,
        is_animating: Option<RwSignal<bool>>,
    ) -> Self {
        let state = SpringAnimationState {
            element,
            keys: keys.clone(),
            final_props: final_props.clone(),
            channels: SpringChannels {
                transform: targets.transform.map(|target| TransformSpringState {
                    x: SpringState {
                        value: current.transform.x,
                        velocity: 0.0,
                    },
                    y: SpringState {
                        value: current.transform.y,
                        velocity: 0.0,
                    },
                    scale: SpringState {
                        value: current.transform.scale,
                        velocity: 0.0,
                    },
                    rotate: SpringState {
                        value: current.transform.rotate,
                        velocity: 0.0,
                    },
                    target,
                }),
                opacity: targets.opacity.map(|target| ScalarSpringState {
                    state: SpringState {
                        value: current.opacity,
                        velocity: 0.0,
                    },
                    target,
                }),
                width: targets.width.map(|target| ScalarSpringState {
                    state: SpringState {
                        value: current.width.unwrap_or(target),
                        velocity: 0.0,
                    },
                    target,
                }),
                height: targets.height.map(|target| ScalarSpringState {
                    state: SpringState {
                        value: current.height.unwrap_or(target),
                        velocity: 0.0,
                    },
                    target,
                }),
            },
            spring,
            loop_state: SpringLoopState {
                delay_remaining_ms: delay_ms as f64,
                last_time_ms: None,
                running: true,
                paused: false,
                schedule_id: 1,
            },
            generation,
            active_animation,
            animation_generation,
            is_animating,
        };
        let animation = Self {
            state: Rc::new(RefCell::new(state)),
        };
        animation.retarget(targets, spring, delay_ms, keys, final_props);
        animation
    }

    fn retarget(
        &self,
        targets: SpringTargets,
        spring: Spring,
        delay_ms: u32,
        keys: Rc<Vec<String>>,
        final_props: Rc<Vec<(String, String)>>,
    ) {
        let mut state = self.state.borrow_mut();
        state.keys = keys;
        state.final_props = final_props;
        state.spring = spring;
        state.loop_state.delay_remaining_ms = delay_ms as f64;
        state.loop_state.last_time_ms = None;
        state.loop_state.running = true;
        state.loop_state.paused = false;

        let previous_channels = state.channels;
        let current = read_current_values(&state.element, &[]);
        state.channels = SpringChannels {
            transform: targets.transform.map(|target| {
                retarget_transform_channel(previous_channels.transform, current.transform, target)
            }),
            opacity: targets.opacity.map(|target| {
                retarget_scalar_channel(previous_channels.opacity, current.opacity, target)
            }),
            width: targets.width.map(|target| {
                retarget_scalar_channel(
                    previous_channels.width,
                    current.width.unwrap_or(target),
                    target,
                )
            }),
            height: targets.height.map(|target| {
                retarget_scalar_channel(
                    previous_channels.height,
                    current.height.unwrap_or(target),
                    target,
                )
            }),
        };
    }

    fn cancel(&self) {
        let mut state = self.state.borrow_mut();
        cancel_spring_loop(&mut state.loop_state);
    }

    fn pause(&self) -> bool {
        let mut state = self.state.borrow_mut();
        pause_spring_loop(&mut state.loop_state)
    }

    fn resume(&self) -> bool {
        let schedule_id = {
            let mut state = self.state.borrow_mut();
            let Some(schedule_id) = resume_spring_loop(&mut state.loop_state) else {
                return false;
            };
            schedule_id
        };
        schedule_spring_step(self.clone(), schedule_id);
        true
    }

    fn begin_schedule(&self) -> u32 {
        let mut state = self.state.borrow_mut();
        state.loop_state.schedule_id = state.loop_state.schedule_id.wrapping_add(1);
        state.loop_state.schedule_id
    }

    fn keys(&self) -> Rc<Vec<String>> {
        self.state.borrow().keys.clone()
    }

    fn set_generation(&self, generation: u32) {
        self.state.borrow_mut().generation = generation;
    }
}

#[cfg(feature = "spring")]
fn schedule_spring_step(animation: SpringAnimation, schedule_id: u32) {
    leptos::prelude::request_animation_frame(move || {
        let (
            element,
            frame_style,
            final_props,
            finished,
            generation,
            animation_generation,
            active_animation,
            is_animating,
        ) = {
            let mut state = animation.state.borrow_mut();
            if !state.loop_state.running || state.loop_state.paused {
                return;
            }
            if state.loop_state.schedule_id != schedule_id {
                return;
            }
            if state.animation_generation.get_value() != state.generation {
                state.loop_state.running = false;
                return;
            }

            let now = now_ms();
            let last = state.loop_state.last_time_ms.unwrap_or(now);
            let dt = normalized_dt((now - last) / 1000.0);
            state.loop_state.last_time_ms = Some(now);

            if state.loop_state.delay_remaining_ms > 0.0 {
                state.loop_state.delay_remaining_ms =
                    (state.loop_state.delay_remaining_ms - dt * 1000.0).max(0.0);
                let generation = state.generation;
                let animation_generation = state.animation_generation;
                let active_animation = state.active_animation;
                let is_animating = state.is_animating;
                let element = state.element.clone();
                let final_props = state.final_props.clone();
                drop(state);
                let _ = (
                    element,
                    final_props,
                    generation,
                    animation_generation,
                    active_animation,
                    is_animating,
                );
                schedule_spring_step(animation.clone(), schedule_id);
                return;
            }

            let spring = state.spring;
            let finished = step_spring_channels(&mut state.channels, spring, dt);

            let frame_style = spring_frame_style_from_channels(state.channels);
            let element = state.element.clone();
            let final_props = state.final_props.clone();
            let generation = state.generation;
            let animation_generation = state.animation_generation;
            let active_animation = state.active_animation;
            let is_animating = state.is_animating;
            if finished {
                state.loop_state.running = false;
                state.loop_state.paused = false;
                state.loop_state.last_time_ms = None;
            }
            (
                element,
                frame_style,
                final_props,
                finished,
                generation,
                animation_generation,
                active_animation,
                is_animating,
            )
        };

        apply_style(&element, &frame_style);

        if finished {
            if animation_generation.get_value() == generation {
                apply_owned_props(&element, final_props.as_ref());
                active_animation.set_value(None);
                if let Some(signal) = is_animating {
                    signal.set(false);
                }
            }
            return;
        }

        schedule_spring_step(animation.clone(), schedule_id);
    });
}

pub(crate) fn apply_style(element: &Element, style: &FluidStyle) {
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

#[inline(never)]
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
        return parse_f64_token(raw.trim()).map(|value| value.max(0.0).round() as u32);
    }
    if let Some(raw) = token.strip_suffix('s') {
        return parse_f64_token(raw.trim()).map(|value| {
            let ms = value.max(0.0) * 1000.0;
            ms.round() as u32
        });
    }
    None
}

fn parse_f64_token(token: &str) -> Option<f64> {
    #[cfg(target_arch = "wasm32")]
    {
        parse_js_f64(token)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        token.parse::<f64>().ok()
    }
}

#[inline(never)]
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

#[cfg(feature = "spring")]
fn retarget_transform_channel(
    previous: Option<TransformSpringState>,
    current: TransformValues,
    target: TransformValues,
) -> TransformSpringState {
    let mut channel = previous.unwrap_or(TransformSpringState {
        x: SpringState::default(),
        y: SpringState::default(),
        scale: SpringState {
            value: 1.0,
            velocity: 0.0,
        },
        rotate: SpringState::default(),
        target,
    });
    if previous.is_none() {
        channel.x.value = current.x;
        channel.y.value = current.y;
        channel.scale.value = current.scale;
        channel.rotate.value = current.rotate;
    }
    channel.target = target;
    channel
}

#[cfg(feature = "spring")]
fn retarget_scalar_channel(
    previous: Option<ScalarSpringState>,
    current: f64,
    target: f64,
) -> ScalarSpringState {
    let mut channel = previous.unwrap_or(ScalarSpringState {
        state: SpringState::default(),
        target,
    });
    if previous.is_none() {
        channel.state.value = current;
    }
    channel.target = target;
    channel
}

#[cfg(feature = "spring")]
fn cancel_spring_loop(loop_state: &mut SpringLoopState) {
    loop_state.running = false;
    loop_state.paused = false;
    loop_state.last_time_ms = None;
    loop_state.schedule_id = loop_state.schedule_id.wrapping_add(1);
}

#[cfg(feature = "spring")]
fn pause_spring_loop(loop_state: &mut SpringLoopState) -> bool {
    if !loop_state.running || loop_state.paused {
        return false;
    }
    loop_state.paused = true;
    loop_state.last_time_ms = None;
    loop_state.schedule_id = loop_state.schedule_id.wrapping_add(1);
    true
}

#[cfg(feature = "spring")]
fn resume_spring_loop(loop_state: &mut SpringLoopState) -> Option<u32> {
    if !loop_state.running || !loop_state.paused {
        return None;
    }
    loop_state.paused = false;
    loop_state.last_time_ms = None;
    loop_state.schedule_id = loop_state.schedule_id.wrapping_add(1);
    Some(loop_state.schedule_id)
}

#[cfg(feature = "spring")]
fn step_spring_channels(channels: &mut SpringChannels, spring: Spring, dt: f64) -> bool {
    let mut finished = true;

    if let Some(transform) = &mut channels.transform {
        finished &= step_spring(&mut transform.x, transform.target.x, spring, dt);
        finished &= step_spring(&mut transform.y, transform.target.y, spring, dt);
        finished &= step_spring(&mut transform.scale, transform.target.scale, spring, dt);
        finished &= step_spring(&mut transform.rotate, transform.target.rotate, spring, dt);
    }
    if let Some(opacity) = &mut channels.opacity {
        finished &= step_spring(&mut opacity.state, opacity.target, spring, dt);
    }
    if let Some(width) = &mut channels.width {
        finished &= step_spring(&mut width.state, width.target, spring, dt);
    }
    if let Some(height) = &mut channels.height {
        finished &= step_spring(&mut height.state, height.target, spring, dt);
    }

    finished
}

#[cfg(feature = "spring")]
fn spring_frame_style_from_channels(channels: SpringChannels) -> FluidStyle {
    let mut style = FluidStyle::new();
    if let Some(transform) = channels.transform {
        style = style
            .x(transform.x.value)
            .y(transform.y.value)
            .scale(transform.scale.value)
            .rotate(transform.rotate.value);
    }
    if let Some(opacity) = channels.opacity {
        style = style.opacity(opacity.state.value);
    }
    if let Some(width) = channels.width {
        style = style.width(width.state.value);
    }
    if let Some(height) = channels.height {
        style = style.height(height.state.value);
    }
    style
}

#[cfg(feature = "spring")]
fn has_spring_targets(targets: SpringTargets) -> bool {
    targets.transform.is_some()
        || targets.opacity.is_some()
        || targets.width.is_some()
        || targets.height.is_some()
}

#[cfg(feature = "spring")]
fn read_current_values(element: &Element, snapshot: &[(String, String)]) -> ParsedCurrentValues {
    let Some(computed) = computed_style(element) else {
        return ParsedCurrentValues {
            transform: TransformValues {
                scale: 1.0,
                ..TransformValues::default()
            },
            opacity: 1.0,
            width: None,
            height: None,
        };
    };

    let transform_raw = snapshot_value(snapshot, "transform")
        .unwrap_or_else(|| read_computed_animation_value(&computed, "transform"));
    let opacity_raw = snapshot_value(snapshot, "opacity")
        .unwrap_or_else(|| read_computed_animation_value(&computed, "opacity"));
    let width_raw = snapshot_value(snapshot, "width")
        .unwrap_or_else(|| read_computed_animation_value(&computed, "width"));
    let height_raw = snapshot_value(snapshot, "height")
        .unwrap_or_else(|| read_computed_animation_value(&computed, "height"));

    ParsedCurrentValues {
        transform: parse_transform_value(&transform_raw).unwrap_or(TransformValues {
            scale: 1.0,
            ..TransformValues::default()
        }),
        opacity: parse_f64_token(opacity_raw.trim()).unwrap_or(1.0),
        width: parse_px_token(width_raw.trim()),
        height: parse_px_token(height_raw.trim()),
    }
}

#[cfg(feature = "spring")]
fn parse_px_token(token: &str) -> Option<f64> {
    let token = token.trim();
    if token.is_empty() {
        return None;
    }
    if let Some(raw) = token.strip_suffix("px") {
        return parse_f64_token(raw.trim());
    }
    parse_f64_token(token)
}

#[cfg(feature = "spring")]
fn parse_transform_value(value: &str) -> Option<TransformValues> {
    let value = value.trim();
    if value.is_empty() || value == "none" {
        return Some(TransformValues {
            scale: 1.0,
            ..TransformValues::default()
        });
    }

    if let Some(values) = parse_function_arguments(value, "matrix3d") {
        if values.len() == 16 {
            let a = values[0];
            let b = values[1];
            return Some(TransformValues {
                x: values[12],
                y: values[13],
                scale: (a * a + b * b).sqrt(),
                rotate: b.atan2(a).to_degrees(),
            });
        }
    }

    if let Some(values) = parse_function_arguments(value, "matrix") {
        if values.len() == 6 {
            let a = values[0];
            let b = values[1];
            return Some(TransformValues {
                x: values[4],
                y: values[5],
                scale: (a * a + b * b).sqrt(),
                rotate: b.atan2(a).to_degrees(),
            });
        }
    }

    let mut parsed = TransformValues {
        scale: 1.0,
        ..TransformValues::default()
    };
    let mut matched = false;

    if let Some(values) = parse_function_arguments(value, "translate3d") {
        if values.len() >= 2 {
            parsed.x = values[0];
            parsed.y = values[1];
            matched = true;
        }
    }
    if let Some(values) = parse_function_arguments(value, "scale") {
        if let Some(scale) = values.first().copied() {
            parsed.scale = scale;
            matched = true;
        }
    }
    if let Some(values) = parse_function_arguments(value, "rotate") {
        if let Some(rotate) = values.first().copied() {
            parsed.rotate = rotate;
            matched = true;
        }
    }

    matched.then_some(parsed)
}

#[cfg(feature = "spring")]
fn parse_function_arguments(value: &str, function_name: &str) -> Option<Vec<f64>> {
    let start = value.find(function_name)?;
    let rest = &value[start + function_name.len()..];
    let rest = rest.strip_prefix('(')?;
    let end = rest.find(')')?;
    let args = &rest[..end];
    let mut values = Vec::new();
    for token in args.split(',') {
        let token = token.trim();
        let parsed = if function_name == "rotate" {
            token
                .strip_suffix("deg")
                .and_then(|raw| parse_f64_token(raw.trim()))
        } else {
            parse_px_token(token)
        }?;
        values.push(parsed);
    }
    Some(values)
}

#[cfg(feature = "spring")]
fn split_spring_animation_props(
    style: &FluidStyle,
    transition: &Transition,
) -> (
    SpringTargets,
    StyleProps,
    TransitionRuntime,
    Rc<Vec<String>>,
    Rc<Vec<(String, String)>>,
) {
    let mut targets = SpringTargets::default();
    let mut immediate = Vec::new();
    let mut keys = Vec::new();
    let mut final_props = Vec::new();
    let has_excluded = !transition.excluded_properties.is_empty();
    let mut runtime = TransitionRuntime {
        duration_ms: transition.duration_ms,
        delay_ms: transition.delay_ms,
        easing: transition.easing_string().to_string(),
    };

    for (key, value) in style.to_props() {
        if key.as_ref() == "transition" {
            if let Some(parsed) = parse_transition_override(&value) {
                runtime.duration_ms = parsed.duration_ms;
                runtime.delay_ms = parsed.delay_ms;
            }
            continue;
        }

        let key_name = key.as_ref();
        keys.push(key_name.to_string());
        final_props.push((key_name.to_string(), value.clone()));

        if has_excluded
            && transition
                .excluded_properties
                .iter()
                .any(|excluded| excluded.as_ref() == key_name)
        {
            immediate.push((key, value));
            continue;
        }

        match key_name {
            "transform" => {
                if let Some(transform) = parse_transform_value(&value) {
                    targets.transform = Some(transform);
                } else {
                    immediate.push((key, value));
                }
            }
            "opacity" => {
                if let Some(opacity) = parse_f64_token(&value) {
                    targets.opacity = Some(opacity);
                } else {
                    immediate.push((key, value));
                }
            }
            "width" => {
                if let Some(width) = parse_px_token(&value) {
                    targets.width = Some(width);
                } else {
                    immediate.push((key, value));
                }
            }
            "height" => {
                if let Some(height) = parse_px_token(&value) {
                    targets.height = Some(height);
                } else {
                    immediate.push((key, value));
                }
            }
            _ => immediate.push((key, value)),
        }
    }

    (
        targets,
        immediate,
        runtime,
        Rc::new(keys),
        Rc::new(final_props),
    )
}

#[inline(never)]
pub(crate) fn cancel_active_animation(
    element: &Element,
    active_animation: StoredValue<Option<ActiveAnimation>, LocalStorage>,
) -> Vec<(String, String)> {
    let Some(active) = active_animation.get_value() else {
        return Vec::new();
    };
    let frozen = match active {
        ActiveAnimation::Waapi(active) => {
            let committed = animation_commit_styles(&active.animation);
            let frozen = freeze_computed_values(element, active.keys.as_ref(), committed);
            animation_set_onfinish(&active.animation, None);
            animation_cancel(&active.animation);
            frozen
        }
        #[cfg(feature = "spring")]
        ActiveAnimation::Spring(active) => {
            active.cancel();
            freeze_computed_values(element, active.keys().as_ref(), true)
        }
    };
    element_set_active_animation(element, None);
    active_animation.set_value(None);
    frozen
}

pub(crate) fn pause_active_animation(
    active_animation: StoredValue<Option<ActiveAnimation>, LocalStorage>,
) -> bool {
    let Some(active) = active_animation.get_value() else {
        return false;
    };
    match active {
        ActiveAnimation::Waapi(active) => animation_pause(&active.animation),
        #[cfg(feature = "spring")]
        ActiveAnimation::Spring(active) => active.pause(),
    }
}

pub(crate) fn resume_active_animation(
    active_animation: StoredValue<Option<ActiveAnimation>, LocalStorage>,
) -> bool {
    let Some(active) = active_animation.get_value() else {
        return false;
    };
    match active {
        ActiveAnimation::Waapi(active) => animation_play(&active.animation),
        #[cfg(feature = "spring")]
        ActiveAnimation::Spring(active) => active.resume(),
    }
}

pub(crate) fn set_immediate(
    element: &Element,
    style: &FluidStyle,
    active_animation: StoredValue<Option<ActiveAnimation>, LocalStorage>,
    animation_generation: StoredValue<u32, LocalStorage>,
    is_animating: Option<RwSignal<bool>>,
) {
    let generation = animation_generation.get_value().wrapping_add(1);
    animation_generation.set_value(generation);

    cancel_active_animation(element, active_animation);
    if !style.is_empty() {
        apply_style(element, style);
    }
    if let Some(signal) = is_animating {
        signal.set(false);
    }
}

#[inline(never)]
pub(crate) fn animate_to(
    element: &Element,
    to: &FluidStyle,
    transition: &Transition,
    active_animation: StoredValue<Option<ActiveAnimation>, LocalStorage>,
    animation_generation: StoredValue<u32, LocalStorage>,
    is_animating: Option<RwSignal<bool>>,
) {
    let generation = animation_generation.get_value().wrapping_add(1);
    animation_generation.set_value(generation);

    #[cfg(feature = "spring")]
    if let Some(mut spring) = transition.spring_config() {
        let (targets, immediate_props, runtime, keys, final_props) =
            split_spring_animation_props(to, transition);

        apply_props(element, &immediate_props);

        if !has_spring_targets(targets) {
            cancel_active_animation(element, active_animation);
            apply_owned_props(element, final_props.as_ref());
            if let Some(signal) = is_animating {
                signal.set(false);
            }
            return;
        }

        spring.duration_ms = runtime.duration_ms;
        if runtime.duration_ms == 0 && runtime.delay_ms == 0 {
            cancel_active_animation(element, active_animation);
            apply_owned_props(element, final_props.as_ref());
            if let Some(signal) = is_animating {
                signal.set(false);
            }
            return;
        }

        if let Some(ActiveAnimation::Spring(animation)) = active_animation.get_value() {
            animation.set_generation(generation);
            animation.retarget(targets, spring, runtime.delay_ms, keys, final_props);
            if let Some(signal) = is_animating {
                signal.set(true);
            }
            let schedule_id = animation.begin_schedule();
            active_animation.set_value(Some(ActiveAnimation::Spring(animation.clone())));
            schedule_spring_step(animation, schedule_id);
            return;
        }

        let snapshot = cancel_active_animation(element, active_animation);
        let current = read_current_values(element, &snapshot);
        let animation = SpringAnimation::new(
            element.clone(),
            current,
            targets,
            spring,
            runtime.delay_ms,
            keys,
            final_props,
            active_animation,
            animation_generation,
            generation,
            is_animating,
        );
        if let Some(signal) = is_animating {
            signal.set(true);
        }
        let schedule_id = animation.begin_schedule();
        active_animation.set_value(Some(ActiveAnimation::Spring(animation.clone())));
        schedule_spring_step(animation, schedule_id);
        return;
    }

    let (animated_props, immediate_props, runtime) = split_animation_props(to, transition);
    let mut final_props = Vec::with_capacity(immediate_props.len() + animated_props.len());
    for (key, value) in immediate_props.iter().chain(animated_props.iter()) {
        final_props.push((key.as_ref().to_string(), value.clone()));
    }

    let snapshot = cancel_active_animation(element, active_animation);
    apply_props(element, &immediate_props);

    if animated_props.is_empty() {
        active_animation.set_value(None);
        if let Some(signal) = is_animating {
            signal.set(false);
        }
        return;
    }

    if runtime.duration_ms == 0 && runtime.delay_ms == 0 {
        apply_props(element, &animated_props);
        active_animation.set_value(None);
        if let Some(signal) = is_animating {
            signal.set(false);
        }
        return;
    }

    let computed = computed_style(element);
    if computed.is_none() && snapshot.is_empty() {
        apply_props(element, &animated_props);
        active_animation.set_value(None);
        if let Some(signal) = is_animating {
            signal.set(false);
        }
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
        apply_props(element, &animated_props);
        active_animation.set_value(None);
        if let Some(signal) = is_animating {
            signal.set(false);
        }
        return;
    };

    if let Some(signal) = is_animating {
        signal.set(true);
    }

    let inner_element = element.clone();
    let inner_final_props = Rc::new(final_props);
    let on_finish = Rc::new(Closure::wrap(Box::new(move || {
        if animation_generation.get_value() != generation {
            return;
        }
        apply_owned_props(&inner_element, inner_final_props.as_ref());
        element_set_active_animation(&inner_element, None);
        active_animation.set_value(None);
        if let Some(signal) = is_animating {
            signal.set(false);
        }
    }) as Box<dyn FnMut()>));
    animation_set_onfinish(&animation, Some(on_finish.as_ref().as_ref()));
    element_set_active_animation(element, Some(&animation));

    active_animation.set_value(Some(ActiveAnimation::Waapi(WaapiAnimation {
        animation,
        keys: Rc::new(animated_keys),
        _on_finish: on_finish,
    })));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FluidStyle;
    #[cfg(feature = "spring")]
    use crate::Spring;

    fn has_prop(props: &StyleProps, key: &str, value: &str) -> bool {
        props
            .iter()
            .any(|(prop_key, prop_value)| prop_key.as_ref() == key && prop_value == value)
    }

    #[test]
    fn parse_time_token_supports_ms_and_seconds() {
        assert_eq!(parse_time_token("180ms"), Some(180));
        assert_eq!(parse_time_token("0.25s"), Some(250));
        assert_eq!(parse_time_token("2s"), Some(2000));
        assert_eq!(parse_time_token("bogus"), None);
    }

    #[test]
    fn parse_transition_override_reads_runtime_fields() {
        let parsed = parse_transition_override("all 320ms ease-in 40ms").unwrap();
        assert_eq!(parsed.duration_ms, 320);
        assert_eq!(parsed.delay_ms, 40);
        assert_eq!(parsed.easing, "ease-in");

        let parsed_without_delay = parse_transition_override("all 140ms ease-out").unwrap();
        assert_eq!(parsed_without_delay.duration_ms, 140);
        assert_eq!(parsed_without_delay.delay_ms, 0);
        assert_eq!(parsed_without_delay.easing, "ease-out");

        let none_transition = parse_transition_override("none").unwrap();
        assert_eq!(none_transition.duration_ms, 0);
        assert_eq!(none_transition.delay_ms, 0);
        assert_eq!(none_transition.easing, "linear");

        assert!(parse_transition_override("opacity 100ms linear").is_none());
    }

    #[test]
    fn split_animation_props_honors_exclusions_and_style_override() {
        let style = FluidStyle::new()
            .with("opacity", "0.84")
            .with("width", "120px")
            .with("transition", "all 460ms linear 30ms");
        let transition = Transition::new()
            .duration_ms(120)
            .exclude_properties(&["width"]);

        let (animated, immediate, runtime) = split_animation_props(&style, &transition);

        assert!(has_prop(&animated, "opacity", "0.84"));
        assert!(has_prop(&immediate, "width", "120px"));
        assert_eq!(runtime.duration_ms, 460);
        assert_eq!(runtime.delay_ms, 30);
        assert_eq!(runtime.easing, "linear");
    }

    #[test]
    fn keyframe_property_name_camel_cases_css_keys() {
        assert_eq!(
            keyframe_property_name("background-color"),
            "backgroundColor"
        );
        assert_eq!(keyframe_property_name("opacity"), "opacity");
        assert_eq!(keyframe_property_name("--fluid-token"), "--fluid-token");
    }

    #[test]
    fn normalize_transform_rewrites_none_to_identity() {
        assert_eq!(
            normalize_transform_value("none".to_string()),
            "matrix(1, 0, 0, 1, 0, 0)"
        );
        assert_eq!(
            normalize_transform_value(" ".to_string()),
            "matrix(1, 0, 0, 1, 0, 0)"
        );
        assert_eq!(
            normalize_transform_value("translate3d(10px, 0px, 0px)".to_string()),
            "translate3d(10px, 0px, 0px)"
        );
    }

    #[cfg(feature = "spring")]
    #[test]
    fn spring_retarget_preserves_existing_velocity() {
        let previous = ScalarSpringState {
            state: SpringState {
                value: 24.0,
                velocity: 7.5,
            },
            target: 48.0,
        };

        let next = retarget_scalar_channel(Some(previous), 0.0, -12.0);

        assert_eq!(next.state.value, 24.0);
        assert_eq!(next.state.velocity, 7.5);
        assert_eq!(next.target, -12.0);
    }

    #[cfg(feature = "spring")]
    #[test]
    fn spring_pause_and_resume_update_schedule_state() {
        let mut loop_state = SpringLoopState {
            running: true,
            paused: false,
            last_time_ms: Some(16.0),
            schedule_id: 3,
            ..SpringLoopState::default()
        };

        assert!(pause_spring_loop(&mut loop_state));
        assert!(loop_state.running);
        assert!(loop_state.paused);
        assert_eq!(loop_state.last_time_ms, None);
        assert_eq!(loop_state.schedule_id, 4);

        let resumed = resume_spring_loop(&mut loop_state);
        assert_eq!(resumed, Some(5));
        assert!(loop_state.running);
        assert!(!loop_state.paused);
        assert_eq!(loop_state.last_time_ms, None);
        assert_eq!(loop_state.schedule_id, 5);

        assert_eq!(resume_spring_loop(&mut loop_state), None);
    }

    #[cfg(feature = "spring")]
    #[test]
    fn split_spring_animation_props_keeps_unsupported_values_immediate() {
        let style = FluidStyle::new()
            .with("opacity", "0.84")
            .with("width", "180px")
            .with("height", "96px")
            .with("transform", "translate3d(24px, 0px, 0px)")
            .with("box-shadow", "0 20px 40px rgba(0,0,0,.2)")
            .with("filter", "blur(12px)");
        let transition = Transition::spring_with(420, 0.28);

        let (targets, immediate, runtime, keys, final_props) =
            split_spring_animation_props(&style, &transition);

        assert_eq!(runtime.duration_ms, 420);
        assert_eq!(runtime.delay_ms, 0);
        assert_eq!(targets.opacity, Some(0.84));
        assert_eq!(targets.width, Some(180.0));
        assert_eq!(targets.height, Some(96.0));
        assert_eq!(targets.transform.unwrap().x, 24.0);
        assert!(keys.iter().any(|key| key == "box-shadow"));
        assert!(keys.iter().any(|key| key == "width"));
        assert!(final_props.iter().any(|(key, _)| key == "filter"));
        assert!(has_prop(
            &immediate,
            "box-shadow",
            "0 20px 40px rgba(0,0,0,.2)"
        ));
        assert!(has_prop(&immediate, "filter", "blur(12px)"));
    }

    #[cfg(feature = "spring")]
    #[test]
    fn spring_channel_step_moves_width_and_height_toward_targets() {
        let mut channels = SpringChannels {
            width: Some(ScalarSpringState {
                state: SpringState {
                    value: 120.0,
                    velocity: 0.0,
                },
                target: 180.0,
            }),
            height: Some(ScalarSpringState {
                state: SpringState {
                    value: 64.0,
                    velocity: 0.0,
                },
                target: 96.0,
            }),
            ..SpringChannels::default()
        };

        let finished = step_spring_channels(&mut channels, Spring::new(420, 0.28), 0.016);

        assert!(!finished);
        assert!(channels.width.unwrap().state.value > 120.0);
        assert!(channels.height.unwrap().state.value > 64.0);
    }
}
