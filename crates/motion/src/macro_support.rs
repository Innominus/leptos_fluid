use leptos::prelude::{Effect, GetValue, LocalStorage, SetValue, StoredValue};

pub fn watch_on_change<T, Source, OnChange>(source: Source, mut on_change: OnChange)
where
    T: Clone + PartialEq + 'static,
    Source: Fn() -> T + 'static,
    OnChange: FnMut(T) + 'static,
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
                move || source.get(),
                move |next| {
                    observed.update(|values| values.push(next));
                },
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
}
