use leptos::prelude::*;
use leptos_fluid_motion::{bind_auto_height, bind_auto_width, AnimationController, Transition};

const LAYOUT_NOTES: [&str; 3] = [
    "Small note.",
    "Expanded note with a second line of context.",
    "A much longer production note that proves the outer shell can expand and shrink without manual height bookkeeping.",
];

const WIDTH_STEPS: [&str; 4] = [
    "Idle",
    "Preparing layout pass",
    "Publishing the wider auto-size state",
    "Ready for another width retarget",
];

#[component]
pub fn AutoLayoutSection() -> impl IntoView {
    let height_mode = RwSignal::new(0usize);
    let width_mode = RwSignal::new(0usize);

    let height_shell_ref = NodeRef::<leptos::html::Div>::new();
    let height_content_ref = NodeRef::<leptos::html::Div>::new();
    let width_shell_ref = NodeRef::<leptos::html::Div>::new();
    let width_content_ref = NodeRef::<leptos::html::Span>::new();

    let height_controller = AnimationController::builder()
        .target(height_shell_ref)
        .transition(Transition::spring_with(460, 0.2))
        .install();
    let width_controller = AnimationController::builder()
        .target(width_shell_ref)
        .transition(Transition::spring_with(420, 0.18))
        .install();

    bind_auto_height(height_controller, height_shell_ref, height_content_ref);
    bind_auto_width(width_controller, width_shell_ref, width_content_ref);

    let show_extra_note = move || height_mode.get() > 0;

    view! {
        <section class="section-grid">
            <div class="panel">
                <p class="section-kicker">"Auto layout"</p>
                <h2>"Animate height and width from measured content"</h2>
                <p>
                    "These helpers watch inner content with ResizeObserver and animate the outer shell to the new dimensions."
                </p>
                <div class="button-row">
                    <button class="alt" on:click=move |_| height_mode.update(|mode| *mode = (*mode + 1) % LAYOUT_NOTES.len())>
                        "Next height state"
                    </button>
                    <button class="alt" on:click=move |_| width_mode.update(|mode| *mode = (*mode + 1) % WIDTH_STEPS.len())>
                        "Next width state"
                    </button>
                </div>
            </div>

            <div class="auto-layout-grid">
                <div class="auto-layout-shell" node_ref=height_shell_ref>
                    <div class="auto-layout-content" node_ref=height_content_ref>
                        <p class="chip">"height"</p>
                        <h3>"Measured note card"</h3>
                        <p>{move || LAYOUT_NOTES[height_mode.get()]}</p>
                        <Show when=show_extra_note>
                            <p class="panel-note">"Additional nested content makes the outer shell retarget cleanly."</p>
                        </Show>
                    </div>
                </div>

                <div class="auto-layout-width-stage">
                    <div class="auto-layout-width-shell" node_ref=width_shell_ref>
                        <span class="auto-layout-width-chip" node_ref=width_content_ref>
                            {move || WIDTH_STEPS[width_mode.get()]}
                        </span>
                    </div>
                </div>
            </div>
        </section>
    }
}
