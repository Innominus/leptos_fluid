use leptos::prelude::{Effect, GetValue, LocalStorage, SetValue, StoredValue};

pub fn watch_on_change<T>(
    source: Box<dyn Fn() -> T + 'static>,
    mut on_change: Box<dyn FnMut(T) + 'static>,
)
where
    T: Clone + PartialEq + 'static,
{
    let previous: StoredValue<Option<T>, LocalStorage> = StoredValue::new_local(None);
    Effect::new(move || {
        let next = source();
        let last = previous.get_value();
        if last.as_ref() == Some(&next) {
            return;
        }

        previous.set_value(Some(next.clone()));
        if last.is_none() {
            return;
        }

        on_change(next);
    });
}

#[cfg(test)]
mod tests {
    use super::watch_on_change;
    use leptos::prelude::{Get, GetUntracked, RwSignal, Set, Update};
    use leptos::reactive::owner::Owner;

    #[test]
    fn watch_on_change_skips_initial_sample_and_ignores_repeats() {
        let _ = any_spawner::Executor::init_futures_executor();

        Owner::new().with(|| {
            let source = RwSignal::new(false);
            let observed = RwSignal::new(Vec::<bool>::new());

            watch_on_change(
                Box::new(move || source.get()),
                Box::new(move |next| {
                    observed.update(|values| values.push(next));
                }),
            );
            any_spawner::Executor::poll_local();

            assert_eq!(observed.get_untracked(), Vec::<bool>::new());

            source.set(true);
            any_spawner::Executor::poll_local();
            assert_eq!(observed.get_untracked(), vec![true]);

            source.set(true);
            any_spawner::Executor::poll_local();
            assert_eq!(observed.get_untracked(), vec![true]);

            source.set(false);
            any_spawner::Executor::poll_local();
            assert_eq!(observed.get_untracked(), vec![true, false]);
        });
    }

    #[test]
    fn watch_on_change_detects_change() {
        let _ = any_spawner::Executor::init_futures_executor();
        Owner::new().with(|| {
            let signal = RwSignal::new(0u32);
            let observed = RwSignal::new(0u32);
            let observed_handle = observed;
            watch_on_change(
                Box::new(move || signal.get()),
                Box::new(move |next: u32| {
                    observed_handle.set(next);
                }),
            );
            any_spawner::Executor::poll_local();
            // Initial run — should NOT fire (skips initial sample)
            assert_eq!(observed.get(), 0);
            // Change value
            signal.set(42);
            any_spawner::Executor::poll_local();
            assert_eq!(observed.get(), 42);
        });
    }

    #[test]
    fn watch_on_change_ignores_repeats() {
        let _ = any_spawner::Executor::init_futures_executor();
        Owner::new().with(|| {
            let signal = RwSignal::new(5u32);
            let call_count = RwSignal::new(0u32);
            let call_count_handle = call_count;
            watch_on_change(
                Box::new(move || signal.get()),
                Box::new(move |_next: u32| {
                    call_count_handle.set(call_count_handle.get() + 1);
                }),
            );
            any_spawner::Executor::poll_local();
            assert_eq!(call_count.get(), 0); // initial skipped
            signal.set(5); // same value
            any_spawner::Executor::poll_local();
            assert_eq!(call_count.get(), 0); // repeat ignored
            signal.set(10); // different value
            any_spawner::Executor::poll_local();
            assert_eq!(call_count.get(), 1);
            signal.set(10); // same again
            any_spawner::Executor::poll_local();
            assert_eq!(call_count.get(), 1); // still 1
        });
    }
}
