use leptos::prelude::*;
use leptos_fluid::flip::{Easing as FlipEasing, Flip, FlipGroup, FlipOptions};

#[component]
pub fn FlipSection() -> impl IntoView {
    let move_right = RwSignal::new(false);
    let size = RwSignal::new(1usize);
    let flip = Flip::new_with_options(
        "flip-pill".to_string(),
        FlipOptions {
            duration: 5000,
            easing: FlipEasing::EaseInOut,
            ..Default::default()
        },
    );
    let is_animating = flip.get_is_animating_signal();

    let move_to = move |to_right: bool| {
        let flip = flip;
        move |_| {
            let current_right = move_right.get_untracked();
            let current_size = size.get_untracked();
            if current_right == to_right {
                return;
            }
            let move_right = move_right;
            let size = size;
            flip.animate(move || {
                move_right.set(to_right);
                size.set(current_size);
            });
        }
    };

    let resize_to = move |next_size: usize| {
        let flip = flip;
        move |_| {
            let current_right = move_right.get_untracked();
            let current_size = size.get_untracked();
            if current_size == next_size {
                return;
            }
            let move_right = move_right;
            let size = size;
            flip.animate(move || {
                move_right.set(current_right);
                size.set(next_size);
            });
        }
    };

    let size_label = move || match size.get() {
        0 => "Small",
        1 => "Medium",
        _ => "Large",
    };

    view! {
        <section class="flip-demo">
            <div class="panel">
                <h2>"FLIP: move + resize"</h2>
                <p>
                    "Now with resizing. Position and size changes are captured and animated via translate + scale."
                </p>
                <div class="button-row">
                    <button class="alt" on:click=move_to(false)>
                        "Left"
                    </button>
                    <button on:click=move_to(true)>"Right"</button>
                </div>
                <div class="button-row">
                    <button class="alt" on:click=resize_to(0)>
                        "Small"
                    </button>
                    <button class="alt" on:click=resize_to(1)>
                        "Medium"
                    </button>
                    <button on:click=resize_to(2)>"Large"</button>
                </div>
                <p class="flip-status">
                    {move || {
                        if is_animating.get() { "Re-targeting mid-flight" } else { "Ready" }
                    }}
                </p>
                <p class="flip-status">
                    {move || {
                        let pos = if move_right.get() { "Right" } else { "Left" };
                        format!("Position: {} · Size: {}", pos, size_label())
                    }}
                </p>
            </div>

            <div class="flip-lane" class:flip-right=move || move_right.get()>
                <div
                    id="flip-pill"
                    class="flip-pill"
                    class:flip-size-sm=move || size.get() == 0
                    class:flip-size-md=move || size.get() == 1
                    class:flip-size-lg=move || size.get() == 2
                >
                    <span class="chip">"FLIP"</span>
                    <h3>"Moving target"</h3>
                    <p>
                        {move || {
                            format!(
                                "{} · {}",
                                if move_right.get() { "Right lane" } else { "Left lane" },
                                size_label(),
                            )
                        }}
                    </p>
                </div>
            </div>
        </section>
    }
}

#[derive(Clone, Copy)]
struct FlipTile {
    id: &'static str,
    title: &'static str,
    detail: &'static str,
    wide: bool,
    tall: bool,
}

const FLIP_TILES: [FlipTile; 6] = [
    FlipTile {
        id: "mix-a",
        title: "Ambient pad",
        detail: "Wide texture",
        wide: true,
        tall: false,
    },
    FlipTile {
        id: "mix-b",
        title: "Bass line",
        detail: "Deep groove",
        wide: false,
        tall: true,
    },
    FlipTile {
        id: "mix-c",
        title: "Synth lead",
        detail: "Bright attack",
        wide: false,
        tall: false,
    },
    FlipTile {
        id: "mix-d",
        title: "Perc hits",
        detail: "Sharp accents",
        wide: false,
        tall: false,
    },
    FlipTile {
        id: "mix-e",
        title: "Vocal chop",
        detail: "Airy echoes",
        wide: true,
        tall: false,
    },
    FlipTile {
        id: "mix-f",
        title: "Drone swell",
        detail: "Low tide",
        wide: false,
        tall: true,
    },
];

#[component]
pub fn FlipGroupSection() -> impl IntoView {
    let dense = RwSignal::new(false);
    let order = RwSignal::new(
        FLIP_TILES
            .iter()
            .map(|tile| tile.id)
            .collect::<Vec<&'static str>>(),
    );
    let flip_group = FlipGroup::new_with_options(
        ".flip-tile".to_string(),
        FlipOptions {
            duration: 720,
            stagger: 35,
            easing: FlipEasing::EaseInOut,
            ..Default::default()
        },
    );
    let is_animating = flip_group.get_is_animating_signal();

    let rotate = {
        let flip_group = flip_group;
        move |_| {
            let order = order;
            flip_group.animate(move || {
                order.update(|items| {
                    if !items.is_empty() {
                        items.rotate_left(1);
                    }
                });
            });
        }
    };

    let reverse = {
        let flip_group = flip_group;
        move |_| {
            let order = order;
            flip_group.animate(move || order.update(|items| items.reverse()));
        }
    };

    let shuffle = {
        let flip_group = flip_group;
        move |_| {
            let order = order;
            flip_group.animate(move || {
                order.update(|items| {
                    if items.len() < 4 {
                        items.reverse();
                        return;
                    }
                    let len = items.len();
                    items.swap(0, len - 1);
                    items.swap(1, 3);
                });
            });
        }
    };

    let toggle_dense = {
        let flip_group = flip_group;
        move |_| {
            let dense = dense;
            flip_group.animate(move || dense.update(|value| *value = !*value));
        }
    };

    let order_label = move || order.get().join(" -> ");

    view! {
        <section class="flip-demo">
            <div class="panel">
                <h2>"FLIP: group reorder"</h2>
                <p>
                    "This demo keeps the same DOM nodes mounted and only changes their CSS order/size inside "
                    <code>"flip_group.animate"</code> "."
                </p>
                <p class="flip-status">
                    "Flow: First snapshot -> mutate order/density -> Last snapshot (next frame) -> invert/play."
                </p>
                <div class="button-row">
                    <button class="alt" on:click=rotate>
                        "Rotate"
                    </button>
                    <button class="alt" on:click=reverse>
                        "Reverse"
                    </button>
                    <button class="alt" on:click=shuffle>
                        "Shuffle"
                    </button>
                    <button on:click=toggle_dense>
                        {move || if dense.get() { "Relax spacing" } else { "Dense spacing" }}
                    </button>
                </div>
                <p class="flip-status">
                    {move || {
                        if is_animating.get() {
                            "Animating group (FLIP in progress)"
                        } else {
                            "Ready for next FLIP capture"
                        }
                    }}
                </p>
                <p class="flip-status">{move || format!("Order: {}", order_label())}</p>
            </div>

            <div class="flip-grid" class:flip-grid-dense=move || dense.get()>
                {FLIP_TILES
                    .iter()
                    .copied()
                    .map(|tile| {
                        view! {
                            <div
                                class="flip-tile"
                                id=tile.id
                                class:flip-tile-wide=tile.wide
                                class:flip-tile-tall=tile.tall
                                style=move || {
                                    let current_order = order.get();
                                    let position = current_order
                                        .iter()
                                        .position(|id| *id == tile.id)
                                        .unwrap_or(0);
                                    format!("order: {};", position)
                                }
                                attr:data-flip-id=tile.id
                            >
                                <span class="chip">"Track"</span>
                                <h3>{tile.title}</h3>
                                <p>{tile.detail}</p>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
    }
}
