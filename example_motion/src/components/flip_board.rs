use leptos::prelude::*;
use leptos_fluid_flip::{FlipGroup, FlipOptions, ScaleMode as FlipScaleMode};

#[derive(Clone, Copy)]
struct BoardTile {
    id: &'static str,
    title: &'static str,
    detail: &'static str,
    wide: bool,
    tall: bool,
}

const BOARD_TILES: [BoardTile; 6] = [
    BoardTile {
        id: "suite-a",
        title: "Signal map",
        detail: "Live inputs",
        wide: true,
        tall: false,
    },
    BoardTile {
        id: "suite-b",
        title: "Review lane",
        detail: "Shared edits",
        wide: false,
        tall: true,
    },
    BoardTile {
        id: "suite-c",
        title: "Tone guide",
        detail: "Palette lock",
        wide: false,
        tall: false,
    },
    BoardTile {
        id: "suite-d",
        title: "Camera notes",
        detail: "Shot list",
        wide: false,
        tall: false,
    },
    BoardTile {
        id: "suite-e",
        title: "Launch brief",
        detail: "Ready cue",
        wide: true,
        tall: false,
    },
    BoardTile {
        id: "suite-f",
        title: "Rhythm pass",
        detail: "Stagger grid",
        wide: false,
        tall: true,
    },
];

#[component]
pub fn FlipBoardSection() -> impl IntoView {
    let dense = RwSignal::new(false);
    let order = RwSignal::new(BOARD_TILES.iter().map(|tile| tile.id).collect::<Vec<_>>());
    let flip_group = FlipGroup::builder()
        .selector(".flip-board-tile")
        .options(
            FlipOptions::new()
                .duration_ms(240)
                .stagger_ms(20)
                .scale_mode(FlipScaleMode::PositionAndScale)
                .scale_correction_selector(".flip-board-shell"),
        )
        .install();

    view! {
        <section class="section-grid">
            <div class="panel">
                <p class="section-kicker">"FlipGroup"</p>
                <h2>"Reorder a live board"</h2>
                <p>
                    "Every tile stays mounted. Only CSS order and density change, so the group animator can interpolate the layout transition cleanly."
                </p>
                <div class="button-row">
                    <button class="alt" on:click=move |_| flip_group.run(move || order.update(rotate_left))>
                        "Rotate"
                    </button>
                    <button class="alt" on:click=move |_| flip_group.run(move || order.update(shuffle_tiles))>
                        "Shuffle"
                    </button>
                    <button on:click=move |_| flip_group.run(move || dense.update(|value| *value = !*value))>
                        {move || if dense.get() { "Relax spacing" } else { "Dense spacing" }}
                    </button>
                </div>
                <p class="panel-note">{move || format!("Order: {}", order.get().join(" -> "))}</p>
            </div>

            <div class="flip-board" class:flip-board-dense=move || dense.get()>
                {BOARD_TILES
                    .iter()
                    .copied()
                    .map(|tile| {
                        view! {
                            <div
                                class="flip-board-tile"
                                id=tile.id
                                class:wide=tile.wide
                                class:tall=tile.tall
                                style=move || format!("order: {};", tile_position(&order.get(), tile.id))
                                attr:data-flip-id=tile.id
                            >
                                <div class="flip-board-shell">
                                    <p class="chip">"board"</p>
                                    <h3>{tile.title}</h3>
                                    <p>{tile.detail}</p>
                                </div>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
    }
}

fn rotate_left(items: &mut Vec<&'static str>) {
    if !items.is_empty() {
        items.rotate_left(1);
    }
}

fn shuffle_tiles(items: &mut Vec<&'static str>) {
    if items.len() < 4 {
        items.reverse();
        return;
    }

    let len = items.len();
    items.swap(0, len - 1);
    items.swap(1, 3);
}

fn tile_position(order: &[&'static str], tile_id: &'static str) -> usize {
    order.iter().position(|id| *id == tile_id).unwrap_or(0)
}
