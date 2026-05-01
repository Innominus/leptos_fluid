use leptos::prelude::{Get, Memo, RwSignal, Signal};

/// A flexible signal wrapper accepted by motion component props.
///
/// `FluidSignal` can be created from static values, closure-based derived
/// values, or explicitly from Leptos signals/memos.
#[derive(Clone, Copy)]
pub struct FluidSignal<T: Clone + Send + Sync + 'static>(Signal<T>);

impl<T: Clone + Send + Sync + 'static> FluidSignal<T> {
    pub fn get(&self) -> T {
        self.0.get()
    }

    pub fn derive(f: impl Fn() -> T + Send + Sync + 'static) -> Self {
        Self(Signal::derive(f))
    }

    pub fn from_signal(signal: Signal<T>) -> Self {
        Self(signal)
    }

    pub fn from_rw_signal(signal: RwSignal<T>) -> Self {
        Self(signal.into())
    }

    pub fn from_memo(memo: Memo<T>) -> Self {
        Self(memo.into())
    }
}

impl<T: Clone + Send + Sync + 'static> FluidSignal<T> {
    pub fn static_value(value: T) -> Self {
        let stored = value;
        Self::derive(move || stored.clone())
    }
}

impl<T, F> From<F> for FluidSignal<T>
where
    T: Clone + Send + Sync + 'static,
    F: Fn() -> T + Send + Sync + 'static,
{
    fn from(value: F) -> Self {
        Self::derive(value)
    }
}

impl From<String> for FluidSignal<String> {
    fn from(value: String) -> Self {
        Self::static_value(value)
    }
}

impl From<&'static str> for FluidSignal<String> {
    fn from(value: &'static str) -> Self {
        Self::static_value(value.to_string())
    }
}

impl From<crate::FluidStyle> for FluidSignal<crate::FluidStyle> {
    fn from(value: crate::FluidStyle) -> Self {
        Self::static_value(value)
    }
}
