use js_sys::Date;
#[cfg(not(target_arch = "wasm32"))]
use leptos::prelude::request_animation_frame;
use leptos::prelude::{Callable, Callback, GetValue, StoredValue};
#[cfg(target_arch = "wasm32")]
use web_sys::wasm_bindgen::{JsCast, closure::Closure};

pub(crate) fn now_ms() -> f64 {
    Date::now()
}

pub(crate) fn schedule_after(
    generation: u32,
    generation_store: StoredValue<u32>,
    total_ms: u32,
    on_complete: Callback<()>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        if generation_store.get_value() != generation {
            return;
        }
        if total_ms == 0 {
            on_complete.run(());
            return;
        }

        let Some(window) = web_sys::window() else {
            on_complete.run(());
            return;
        };

        let callback = Closure::once_into_js(move || {
            if generation_store.get_value() != generation {
                return;
            }
            on_complete.run(());
        });
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            callback.unchecked_ref(),
            total_ms as i32,
        );
        return;
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let start_ms = now_ms();

        fn step(
            generation: u32,
            generation_store: StoredValue<u32>,
            total_ms: u32,
            start_ms: f64,
            on_complete: Callback<()>,
        ) {
            request_animation_frame(move || {
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
}
