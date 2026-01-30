use leptos::prelude::{Get, Memo, RwSignal, Signal};

#[derive(Clone, Copy)]
pub struct MotionSignal<T: Clone + Send + Sync + 'static>(Signal<T>);

impl<T: Clone + Send + Sync + 'static> MotionSignal<T> {
    pub fn get(&self) -> T {
        self.0.get()
    }

    pub fn derive(f: impl Fn() -> T + Send + Sync + 'static) -> Self {
        Self(Signal::derive(f))
    }
}

impl<T: Clone + Send + Sync + 'static> MotionSignal<T> {
    pub fn static_value(value: T) -> Self {
        let stored = value;
        Self::derive(move || stored.clone())
    }
}

impl<T: Clone + Send + Sync + 'static> From<Signal<T>> for MotionSignal<T> {
    fn from(value: Signal<T>) -> Self {
        Self(value)
    }
}

impl<T: Clone + Send + Sync + 'static> From<RwSignal<T>> for MotionSignal<T> {
    fn from(value: RwSignal<T>) -> Self {
        Self(value.into())
    }
}

impl<T: Clone + Send + Sync + 'static> From<Memo<T>> for MotionSignal<T> {
    fn from(value: Memo<T>) -> Self {
        Self(value.into())
    }
}

impl<T, F> From<F> for MotionSignal<T>
where
    T: Clone + Send + Sync + 'static,
    F: Fn() -> T + Send + Sync + 'static,
{
    fn from(value: F) -> Self {
        Self::derive(value)
    }
}

impl From<String> for MotionSignal<String> {
    fn from(value: String) -> Self {
        Self::static_value(value)
    }
}

impl From<&'static str> for MotionSignal<String> {
    fn from(value: &'static str) -> Self {
        Self::static_value(value.to_string())
    }
}

impl From<crate::MotionStyle> for MotionSignal<crate::MotionStyle> {
    fn from(value: crate::MotionStyle) -> Self {
        Self::static_value(value)
    }
}
