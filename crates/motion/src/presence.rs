use leptos::prelude::*;

use crate::components::fluid_element_view;
use crate::timing::schedule_after;
use crate::{FluidNodeRef, FluidSignal, FluidStyle, Transition};

use std::sync::Arc;

#[component]
pub fn FluidPresence(
    /// Whether the element should be present in the DOM.
    #[prop(into)]
    show: Signal<bool>,
    /// Initial style applied on mount before animating to `animate`.
    #[prop(default = FluidStyle::default())]
    initial: FluidStyle,
    /// Style applied while present.
    #[prop(default = FluidStyle::default())]
    animate: FluidStyle,
    /// Style applied before exit.
    #[prop(default = FluidStyle::default())]
    exit: FluidStyle,
    /// Transition used for enter/exit.
    #[prop(default = Transition::default())]
    transition: Transition,
    /// Underlying HTML tag (defaults to "div").
    #[prop(default = "div")]
    tag: &'static str,
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
    /// Optional callback fired after the exit animation completes.
    #[prop(optional)]
    on_exit_complete: Option<Callback<()>>,
    /// Disable the initial enter animation when the element is present on first render.
    #[prop(default = true)]
    initial_animation: bool,
    /// Child view(s) inside the fluid element.
    #[prop(optional)]
    children: Option<ChildrenFn>,
) -> impl IntoView {
    let initial_present = show.get_untracked();
    let present = RwSignal::new(initial_present);
    let anim_style = RwSignal::new(animate.clone());
    let exit_generation = StoredValue::new(0u32);
    let animate_store = StoredValue::new(animate);
    let exit_store = StoredValue::new(exit);
    let transition_store = StoredValue::new(transition);
    let initial_store = StoredValue::new(initial);
    let initial_override = RwSignal::new(!initial_animation && initial_present);
    let reset_counter = RwSignal::new(0u32);

    Effect::new(move || {
        let is_showing = show.get();
        if is_showing {
            exit_generation.set_value(exit_generation.get_value().wrapping_add(1));
            if !present.get_untracked() {
                reset_counter.update(|value| *value = value.wrapping_add(1));
                present.set(true);
            }
            anim_style.set(animate_store.get_value());
            return;
        }

        if !present.get_untracked() {
            return;
        }

        let exit_style = exit_store.get_value();
        if exit_style.is_empty() {
            present.set(false);
            if let Some(callback) = on_exit_complete.as_ref() {
                callback.run(());
            }
            return;
        }

        anim_style.set(exit_style);
        let transition = transition_store.get_value();
        let total_ms = transition.duration_ms + transition.delay_ms;
        if total_ms == 0 {
            present.set(false);
            if let Some(callback) = on_exit_complete.as_ref() {
                callback.run(());
            }
            return;
        }

        let generation = exit_generation.get_value().wrapping_add(1);
        exit_generation.set_value(generation);
        let on_exit = Callback::new(move |_| {
            present.set(false);
            if let Some(callback) = on_exit_complete.as_ref() {
                callback.run(());
            }
        });
        schedule_after(generation, exit_generation, total_ms, on_exit);
    });

    let children_store = StoredValue::new(children);
    let while_hover_store = StoredValue::new(while_hover);
    let while_tap_store = StoredValue::new(while_tap);
    let class_store = StoredValue::new(class);
    let style_store = StoredValue::new(style);
    let node_ref_store = StoredValue::new(node_ref);
    let reset_store = StoredValue::new(reset_counter);

    view! {
        <Show when=move || {
            present.get()
        }>
            {move || {
                let content = children_store
                    .with_value(|children| {
                        children.as_ref().map(|c| c()).unwrap_or_else(|| ().into_any())
                    });
                let node_ref = node_ref_store.get_value().unwrap_or_default();
                let reset = reset_store.get_value();
                let class = class_store.get_value();
                let style = style_store.get_value();
                let initial = if initial_override.get_untracked() {
                    initial_override.set(false);
                    animate_store.get_value()
                } else {
                    initial_store.get_value()
                };
                let transition = transition_store.get_value();
                let while_hover = while_hover_store.get_value();
                let while_tap = while_tap_store.get_value();
                let animate_signal = FluidSignal::from(move || anim_style.get());
                let reset_signal = reset.into();
                fluid_element_view(
                        tag,
                        initial,
                        animate_signal,
                        transition,
                        reset_signal,
                        while_hover,
                        while_tap,
                        class,
                        style,
                        node_ref,
                        content,
                    )
                    .into_any()
            }}
        </Show>
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresenceMode {
    Sync,
    Wait,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SwapState {
    active: bool,
    visible: bool,
    pending: Option<bool>,
}

impl SwapState {
    fn new(active: bool) -> Self {
        Self {
            active,
            visible: true,
            pending: None,
        }
    }

    fn on_show(mut self, target: bool) -> Self {
        if self.pending.is_some() {
            self.pending = Some(target);
            return self;
        }

        if target == self.active && self.visible {
            return self;
        }

        if target == self.active && !self.visible {
            self.pending = Some(target);
            return self;
        }

        self.pending = Some(target);
        self.visible = false;
        self
    }

    fn on_exit_complete(mut self) -> Self {
        if let Some(next) = self.pending.take() {
            self.active = next;
            self.visible = true;
        }
        self
    }
}

#[component]
pub fn FluidSwap<F1, F2, V1, V2>(
    /// When true, render the `first` view; when false, render `second`.
    #[prop(into)]
    show: Signal<bool>,
    /// Initial style applied on mount before animating to `animate`.
    #[prop(default = FluidStyle::default())]
    initial: FluidStyle,
    /// Style applied while present.
    #[prop(default = FluidStyle::default())]
    animate: FluidStyle,
    /// Style applied before exit.
    #[prop(default = FluidStyle::default())]
    exit: FluidStyle,
    /// Transition used for enter/exit.
    #[prop(default = Transition::default())]
    transition: Transition,
    /// Whether the incoming view should wait for the outgoing view to exit.
    #[prop(default = PresenceMode::Sync)]
    mode: PresenceMode,
    /// Optional callback fired after exit animations complete.
    #[prop(optional)]
    on_exit_complete: Option<Callback<()>>,
    /// Render the primary view.
    first: F1,
    /// Render the secondary view.
    second: F2,
) -> impl IntoView
where
    F1: Fn() -> V1 + Clone + Send + Sync + 'static,
    F2: Fn() -> V2 + Clone + Send + Sync + 'static,
    V1: IntoView + 'static,
    V2: IntoView + 'static,
{
    let first = Arc::new(move || first().into_view().into_any());
    let second = Arc::new(move || second().into_view().into_any());
    fluid_swap_view(
        show,
        initial,
        animate,
        exit,
        transition,
        mode,
        on_exit_complete,
        first,
        second,
    )
}

fn fluid_swap_view(
    show: Signal<bool>,
    initial: FluidStyle,
    animate: FluidStyle,
    exit: FluidStyle,
    transition: Transition,
    mode: PresenceMode,
    on_exit_complete: Option<Callback<()>>,
    first: ChildrenFn,
    second: ChildrenFn,
) -> impl IntoView {
    let initial_show = show.get_untracked();
    let on_exit_store = StoredValue::new(on_exit_complete);
    let first_store = StoredValue::new(first);
    let second_store = StoredValue::new(second);
    let initial_store = StoredValue::new(initial);
    let animate_store = StoredValue::new(animate);
    let exit_store = StoredValue::new(exit);
    let transition_store = StoredValue::new(transition);

    let render_swap = {
        let first_store = first_store;
        let second_store = second_store;
        let initial_store = initial_store;
        let animate_store = animate_store;
        let exit_store = exit_store;
        let transition_store = transition_store;
        move |show_first: Signal<bool>, show_second: Signal<bool>, on_exit: Callback<()>| {
            let on_exit_first = on_exit.clone();
            let on_exit_second = on_exit;
            view! {
                <>
                    <FluidPresence
                        show=show_first
                        initial=initial_store.get_value()
                        animate=animate_store.get_value()
                        exit=exit_store.get_value()
                        transition=transition_store.get_value()
                        on_exit_complete=on_exit_first
                    >
                        {move || first_store.with_value(|first| first())}
                    </FluidPresence>
                    <FluidPresence
                        show=show_second
                        initial=initial_store.get_value()
                        animate=animate_store.get_value()
                        exit=exit_store.get_value()
                        transition=transition_store.get_value()
                        on_exit_complete=on_exit_second
                    >
                        {move || second_store.with_value(|second| second())}
                    </FluidPresence>
                </>
            }
        }
    };

    match mode {
        PresenceMode::Sync => {
            let show_first = Signal::derive(move || show.get());
            let show_second = Signal::derive(move || !show.get());
            let on_exit_store_sync = on_exit_store;
            let on_exit = Callback::new(move |_| {
                if let Some(callback) = on_exit_store_sync.get_value().as_ref() {
                    callback.run(());
                }
            });

            render_swap(show_first, show_second, on_exit)
        }
        PresenceMode::Wait => {
            let state = RwSignal::new(SwapState::new(initial_show));
            let show_first = Signal::derive({
                let state = state;
                move || {
                    let state = state.get();
                    state.active && state.visible
                }
            });
            let show_second = Signal::derive({
                let state = state;
                move || {
                    let state = state.get();
                    !state.active && state.visible
                }
            });

            Effect::new({
                let state = state;
                move || {
                    let target = show.get();
                    state.update(|state| {
                        *state = state.on_show(target);
                    });
                }
            });

            let on_exit_store_wait = on_exit_store;
            let on_exit = Callback::new(move |_| {
                state.update(|state| {
                    *state = state.on_exit_complete();
                });
                if let Some(callback) = on_exit_store_wait.get_value().as_ref() {
                    callback.run(());
                }
            });

            render_swap(show_first, show_second, on_exit)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SwapState;

    #[test]
    fn swap_wait_basic_flow() {
        let mut state = SwapState::new(true);
        assert_eq!(state.active, true);
        assert_eq!(state.visible, true);
        assert_eq!(state.pending, None);

        state = state.on_show(false);
        assert_eq!(state.active, true);
        assert_eq!(state.visible, false);
        assert_eq!(state.pending, Some(false));

        state = state.on_exit_complete();
        assert_eq!(state.active, false);
        assert_eq!(state.visible, true);
        assert_eq!(state.pending, None);

        state = state.on_show(true);
        assert_eq!(state.active, false);
        assert_eq!(state.visible, false);
        assert_eq!(state.pending, Some(true));

        state = state.on_exit_complete();
        assert_eq!(state.active, true);
        assert_eq!(state.visible, true);
        assert_eq!(state.pending, None);
    }

    #[test]
    fn swap_wait_rapid_toggle_keeps_latest_target() {
        let mut state = SwapState::new(true);
        state = state.on_show(false);
        assert_eq!(state.pending, Some(false));
        state = state.on_show(true);
        assert_eq!(state.pending, Some(true));
        state = state.on_exit_complete();
        assert_eq!(state.active, true);
        assert_eq!(state.visible, true);
        assert_eq!(state.pending, None);
    }

    #[test]
    fn swap_wait_noop_when_target_matches() {
        let mut state = SwapState::new(false);
        state = state.on_show(false);
        assert_eq!(state.active, false);
        assert_eq!(state.visible, true);
        assert_eq!(state.pending, None);
    }
}
