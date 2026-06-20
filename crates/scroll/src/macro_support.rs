//! Shared runtime helpers used by the builder and the `scroll_trigger!` macro.
//!
//! Mirrors `crates/motion/src/macro_support.rs`: a thin, doc-hidden home for
//! small reactive utilities that the typed builder and the declarative macro
//! both rely on. Keeping these out of `trigger.rs` lets `trigger.rs` stay
//! focused on the runtime handle while the ergonomics layer imports from here.
//!
//! The scroll builder lowers every macro field to a builder method, so the
//! macro itself does not need a bespoke watcher. `watch_progress` is provided
//! for callers that want the same skip-initial `Effect` pattern the motion
//! crate's `watch_on_change` exposes, useful for hand-rolled `on_update`
//! handlers driven off `progress()`.
//!
//! `ScrubKind` is a small dispatch helper that lets `scroll_trigger! { scrub: ... }`
//! accept `true` / `false` / a numeric catch-up duration / or a ready-made
//! `Scrub` value, resolving the ambiguity at runtime via `Into<Scrub>`.

use leptos::prelude::{Effect, Get, GetValue, LocalStorage, SetValue, StoredValue};

use crate::config::Scrub;

/// Dispatch helper for the `scroll_trigger!` macro's `scrub:` field.
///
/// `from_auto` picks the right `Scrub` variant based on the input type:
/// `bool` -> `Scrub::Bool`, `f64`/`i32`/etc. -> `Scrub::Number`, and any
/// `Scrub` value is passed through unchanged. This sidesteps the macro-level
/// ambiguity between `true` (a literal that is also a valid `expr`) and
/// numeric literals, which a declarative `macro_rules!` matcher cannot
/// reliably distinguish once the value is captured as `$expr`.
pub enum ScrubKind {
    Bool(bool),
    Number(f64),
    Scrub(Scrub),
}

impl ScrubKind {
    pub fn from_auto<T>(value: T) -> Self
    where
        T: ScrubAuto,
    {
        value.into_scrub_kind()
    }

    pub fn into_scrub(self) -> Scrub {
        match self {
            ScrubKind::Bool(b) => Scrub::Bool(b),
            ScrubKind::Number(n) => Scrub::Number(n),
            ScrubKind::Scrub(s) => s,
        }
    }
}

/// Trait used by [`ScrubKind::from_auto`] to pick the right variant.
pub trait ScrubAuto {
    fn into_scrub_kind(self) -> ScrubKind;
}

impl ScrubAuto for bool {
    fn into_scrub_kind(self) -> ScrubKind {
        ScrubKind::Bool(self)
    }
}

impl ScrubAuto for f64 {
    fn into_scrub_kind(self) -> ScrubKind {
        ScrubKind::Number(self)
    }
}

impl ScrubAuto for f32 {
    fn into_scrub_kind(self) -> ScrubKind {
        ScrubKind::Number(self as f64)
    }
}

impl ScrubAuto for i32 {
    fn into_scrub_kind(self) -> ScrubKind {
        ScrubKind::Number(self as f64)
    }
}

impl ScrubAuto for u32 {
    fn into_scrub_kind(self) -> ScrubKind {
        ScrubKind::Number(self as f64)
    }
}

impl ScrubAuto for usize {
    fn into_scrub_kind(self) -> ScrubKind {
        ScrubKind::Number(self as f64)
    }
}

impl ScrubAuto for Scrub {
    fn into_scrub_kind(self) -> ScrubKind {
        ScrubKind::Scrub(self)
    }
}

/// Runs `on_change` whenever `progress` changes, skipping the initial sample.
///
/// Mirrors `watch_on_change` in `crates/motion/src/macro_support.rs`: the first
/// sample records the baseline but does not invoke `on_change`; subsequent
/// samples fire only when the value actually differs from the last seen value.
#[cfg_attr(not(test), allow(dead_code))]
pub fn watch_progress<F>(progress: leptos::prelude::Signal<f64>, mut on_change: F)
where
    F: FnMut(f64) + 'static,
{
    let previous: StoredValue<Option<f64>, LocalStorage> = StoredValue::new_local(None);
    Effect::new(move || {
        let next = progress.get();
        let last = previous.get_value();
        if last.as_ref() == Some(&next) {
            return;
        }
        previous.set_value(Some(next));
        if last.is_none() {
            return;
        }
        on_change(next);
    });
}

#[cfg(test)]
mod tests {
    use super::watch_progress;
    use leptos::prelude::{GetUntracked, RwSignal, Set, Update};
    use leptos::reactive::owner::Owner;

    #[test]
    fn watch_progress_skips_initial_sample_and_ignores_repeats() {
        let _ = any_spawner::Executor::init_futures_executor();
        Owner::new().with(|| {
            let source = RwSignal::new(0.0);
            let observed = RwSignal::new(Vec::<f64>::new());

            watch_progress(
                source.into(),
                move |next| {
                    observed.update(|values| values.push(next));
                },
            );
            any_spawner::Executor::poll_local();

            assert_eq!(observed.get_untracked(), Vec::<f64>::new());

            source.set(0.5);
            any_spawner::Executor::poll_local();
            assert_eq!(observed.get_untracked(), vec![0.5]);

            source.set(0.5);
            any_spawner::Executor::poll_local();
            assert_eq!(observed.get_untracked(), vec![0.5]);

            source.set(1.0);
            any_spawner::Executor::poll_local();
            assert_eq!(observed.get_untracked(), vec![0.5, 1.0]);
        });
    }
}