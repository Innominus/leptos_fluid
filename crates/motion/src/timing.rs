use js_sys::Date;
use leptos::prelude::{Callable, Callback, GetValue, StoredValue, request_animation_frame};

pub(crate) fn now_ms() -> f64 {
    Date::now()
}

pub(crate) fn schedule_after(
    generation: u32,
    generation_store: StoredValue<u32>,
    total_ms: u32,
    on_complete: Callback<()>,
) {
    let start_ms = now_ms();

    fn step(
        generation: u32,
        generation_store: StoredValue<u32>,
        total_ms: u32,
        start_ms: f64,
        on_complete: Callback<()>,
    ) {
        request_animation_frame(move || {
            // Abort stale timers when a newer timeline generation starts.
            if generation_store.get_value() != generation {
                return;
            }

            let elapsed = now_ms() - start_ms;
            if elapsed >= total_ms as f64 {
                on_complete.run(());
                return;
            }

            step(
                generation,
                generation_store,
                total_ms,
                start_ms,
                on_complete,
            );
        });
    }

    step(
        generation,
        generation_store,
        total_ms,
        start_ms,
        on_complete,
    );
}
