use leptos::prelude::*;
use leptos_fluid_motion::{
    bind_auto_height, bind_auto_width, AnimationController, FluidStyle, Transition,
};

const HEIGHT_COPY: [(&str, &str); 3] = [
    ("Compact", "A short note keeps the card tight."),
    (
        "Expanded",
        "A longer note pushes the content height and the shell follows it.",
    ),
    (
        "Expanded + checklist",
        "Extra nested blocks prove the helper can keep retargeting without manual height code.",
    ),
];

const WIDTH_LABELS: [&str; 4] = [
    "Idle",
    "Prep handoff",
    "Publish system",
    "Ready for next change",
];

#[component]
pub fn AutoSizeExample() -> impl IntoView {
    let height_mode = RwSignal::new(0usize);
    let width_mode = RwSignal::new(0usize);

    let height_shell_ref = NodeRef::<leptos::html::Div>::new();
    let height_content_ref = NodeRef::<leptos::html::Div>::new();
    let width_shell_ref = NodeRef::<leptos::html::Div>::new();
    let width_content_ref = NodeRef::<leptos::html::Span>::new();

    let height_controller = AnimationController::builder()
        .target(height_shell_ref)
        .transition(Transition::spring_with(500, 0.22))
        .install();
    let width_controller = AnimationController::builder()
        .target(width_shell_ref)
        .transition(Transition::spring_with(420, 0.2))
        .install();

    bind_auto_height(height_controller, height_shell_ref, height_content_ref);
    bind_auto_width(width_controller, width_shell_ref, width_content_ref);

    let seed_height = StoredValue::new(false);
    let seed_width = StoredValue::new(false);

    Effect::new(move || {
        if seed_height.get_value()
            || height_shell_ref.get().is_none()
            || height_content_ref.get().is_none()
        {
            return;
        }
        seed_height.set_value(true);
        height_controller.set_immediate(
            FluidStyle::new()
                .opacity(1.0)
                .with("border-color", "rgba(94, 234, 212, 0.2)"),
        );
    });

    let show_height_list = move || height_mode.get() >= 1;
    let show_height_footnote = move || height_mode.get() >= 2;

    Effect::new(move || {
        if seed_width.get_value()
            || width_shell_ref.get().is_none()
            || width_content_ref.get().is_none()
        {
            return;
        }
        seed_width.set_value(true);
        width_controller.set_immediate(
            FluidStyle::new()
                .opacity(1.0)
                .with("border-color", "rgba(96, 165, 250, 0.24)"),
        );
    });

    view! {
        <article class="demo-panel panel-wide" data-testid="auto-size-panel">
            <div class="panel-header">
                <p class="panel-eyebrow">"Auto size"</p>
                <h2>"ResizeObserver-driven height and width transitions"</h2>
                <p>
                    "The helper measures inner content and animates the outer shell to match."
                </p>
            </div>

            <div class="auto-size-grid">
                <div class="auto-size-card">
                    <div class="button-row">
                        <button class="alt" data-testid="auto-height-next" on:click=move |_| height_mode.update(|mode| *mode = (*mode + 1) % HEIGHT_COPY.len())>
                            "Next height state"
                        </button>
                    </div>
                    <div class="auto-size-height-shell" node_ref=height_shell_ref data-testid="auto-height-shell">
                        <div class="auto-size-height-content" node_ref=height_content_ref>
                            <p class="chip">"height"</p>
                            <h3>{move || HEIGHT_COPY[height_mode.get()].0}</h3>
                            <p>{move || HEIGHT_COPY[height_mode.get()].1}</p>
                            <Show when=show_height_list>
                                <ul class="auto-size-list">
                                    <li>"ResizeObserver measures the inner content."</li>
                                    <li>"The outer shell animates to the new size."</li>
                                </ul>
                            </Show>
                            <Show when=show_height_footnote>
                                <p class="auto-size-footnote">
                                    "This extra block keeps pushing the shell outward."
                                </p>
                            </Show>
                        </div>
                    </div>
                </div>

                <div class="auto-size-card">
                    <div class="button-row">
                        <button class="alt" data-testid="auto-width-next" on:click=move |_| width_mode.update(|mode| *mode = (*mode + 1) % WIDTH_LABELS.len())>
                            "Next width label"
                        </button>
                    </div>
                    <div class="auto-size-width-stage">
                        <div class="auto-size-width-shell" node_ref=width_shell_ref data-testid="auto-width-shell">
                            <span class="auto-size-width-label" node_ref=width_content_ref data-testid="auto-width-label">
                                {move || WIDTH_LABELS[width_mode.get()]}
                            </span>
                        </div>
                    </div>
                    <p class="panel-note">
                        "The capsule follows the measured label width."
                    </p>
                </div>
            </div>
        </article>
    }
}
