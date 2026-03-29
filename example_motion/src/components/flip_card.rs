use leptos::prelude::*;
use leptos_fluid_flip::{Easing as FlipEasing, Flip, FlipOptions, ScaleMode as FlipScaleMode};

#[derive(Clone, Copy, PartialEq, Eq)]
enum CardScale {
    Compact,
    Standard,
    Panorama,
}

impl CardScale {
    fn label(self) -> &'static str {
        match self {
            Self::Compact => "Compact",
            Self::Standard => "Standard",
            Self::Panorama => "Panorama",
        }
    }
}

#[component]
pub fn FlipCardSection() -> impl IntoView {
    let on_right = RwSignal::new(false);
    let scale = RwSignal::new(CardScale::Standard);
    let flip = Flip::new_with_options(
        "flip-workbench-card".to_string(),
        FlipOptions {
            duration: 880,
            easing: FlipEasing::EaseInOut,
            scale_mode: FlipScaleMode::PositionAndScale,
            scale_correction_selector: Some(".flip-workbench-card-shell"),
            ..Default::default()
        },
    );

    view! {
        <section class="section-grid">
            <div class="panel">
                <p class="section-kicker">"Flip::new"</p>
                <h2>"Move and resize one live card"</h2>
                <p>
                    "The DOM node stays mounted while position and dimensions change. FLIP captures first and last layouts, then plays the inverted transform."
                </p>
                <div class="button-row">
                    <button class="alt" on:click=move |_| animate_flip_side(flip, on_right, scale, false)>
                        "Dock left"
                    </button>
                    <button on:click=move |_| animate_flip_side(flip, on_right, scale, true)>
                        "Dock right"
                    </button>
                    <button class="alt" on:click=move |_| animate_flip_scale(flip, on_right, scale, CardScale::Compact)>
                        "Compact"
                    </button>
                    <button class="alt" on:click=move |_| animate_flip_scale(flip, on_right, scale, CardScale::Standard)>
                        "Standard"
                    </button>
                    <button on:click=move |_| animate_flip_scale(flip, on_right, scale, CardScale::Panorama)>
                        "Panorama"
                    </button>
                </div>
                <p class="panel-note">
                    {move || {
                        let lane = if on_right.get() { "Right lane" } else { "Left lane" };
                        format!("{lane} · {}", scale.get().label())
                    }}
                </p>
            </div>

            <div class="flip-lane" class:lane-right=move || on_right.get()>
                <div
                    id="flip-workbench-card"
                    class="flip-workbench-card"
                    class:scale-compact=move || scale.get() == CardScale::Compact
                    class:scale-standard=move || scale.get() == CardScale::Standard
                    class:scale-panorama=move || scale.get() == CardScale::Panorama
                >
                    <div class="flip-workbench-card-shell">
                        <p class="chip">"flip"</p>
                        <h3>"Workbench card"</h3>
                        <p>
                            {move || format!("{} layout in the {}", scale.get().label(), if on_right.get() { "right lane" } else { "left lane" })}
                        </p>
                    </div>
                </div>
            </div>
        </section>
    }
}

fn animate_flip_side(
    flip: Flip,
    on_right: RwSignal<bool>,
    scale: RwSignal<CardScale>,
    next_side: bool,
) {
    let current_scale = scale.get_untracked();
    if on_right.get_untracked() == next_side {
        return;
    }

    flip.animate(move || {
        on_right.set(next_side);
        scale.set(current_scale);
    });
}

fn animate_flip_scale(
    flip: Flip,
    on_right: RwSignal<bool>,
    scale: RwSignal<CardScale>,
    next_scale: CardScale,
) {
    let current_side = on_right.get_untracked();
    if scale.get_untracked() == next_scale {
        return;
    }

    flip.animate(move || {
        on_right.set(current_side);
        scale.set(next_scale);
    });
}
