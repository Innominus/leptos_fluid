#![cfg(feature = "wrappers")]
#![allow(dead_code)]

use leptos::prelude::*;
use leptos_fluid_motion::{FluidDiv, FluidSignal, FluidStyle, Transition};

#[component]
fn Demo(open: RwSignal<bool>) -> impl IntoView {
    view! {
        <FluidDiv
            initial=FluidStyle::new().opacity(0.0)
            animate=move || {
                if open.get() {
                    FluidStyle::new().opacity(1.0)
                } else {
                    FluidStyle::new().opacity(0.0)
                }
            }
            transition=Transition::new()
        />
    }
}

#[test]
fn accepts_known_signal_and_value_types() {
    let value: FluidSignal<FluidStyle> = FluidStyle::new().opacity(1.0).into();
    let _ = value.get();

    let rw = RwSignal::new(FluidStyle::new().opacity(0.5));
    let from_rw: FluidSignal<FluidStyle> = FluidSignal::from_rw_signal(rw);
    let _ = from_rw.get();

    let memo = Memo::new(move |_| rw.get());
    let from_memo: FluidSignal<FluidStyle> = FluidSignal::from_memo(memo);
    let _ = from_memo.get();

    let signal: Signal<FluidStyle> = rw.into();
    let from_signal: FluidSignal<FluidStyle> = FluidSignal::from_signal(signal);
    let _ = from_signal.get();
}
