use leptos::prelude::*;
use leptos_fluid::motion::{use_spring, MotionDiv, MotionStyle, Spring, Transition};

#[component]
pub fn TabsSection() -> impl IntoView {
    let active_tab = RwSignal::new(0usize);
    let tabs_ref = NodeRef::<leptos::html::Div>::new();
    let tab_refs = (0..4)
        .map(|_| NodeRef::<leptos::html::Button>::new())
        .collect::<Vec<_>>();
    let tab_refs_effect = tab_refs.clone();
    let underline_spring = Spring::new(520, 0.4);
    let underline_x = use_spring(0.0, underline_spring);
    let underline_w = use_spring(0.0, underline_spring);

    Effect::new({
        let tabs_ref = tabs_ref.clone();
        let tab_refs_effect = tab_refs_effect.clone();
        let underline_x = underline_x.clone();
        let underline_w = underline_w.clone();
        move || {
            let index = active_tab.get();
            let tabs_ref = tabs_ref.clone();
            let tab_refs_effect = tab_refs_effect.clone();
            let underline_x = underline_x.clone();
            let underline_w = underline_w.clone();
            request_animation_frame(move || {
                let Some(container) = tabs_ref.get_untracked() else {
                    return;
                };
                let Some(tab_ref) = tab_refs_effect.get(index) else {
                    return;
                };
                let Some(tab) = tab_ref.get_untracked() else {
                    return;
                };
                let container_rect = container.get_bounding_client_rect();
                let tab_rect = tab.get_bounding_client_rect();
                underline_x.set(tab_rect.left() - container_rect.left());
                underline_w.set(tab_rect.width());
            });
        }
    });

    let underline_style = move || {
        MotionStyle::new()
            .x(underline_x.get())
            .width(underline_w.get())
            .opacity(1.0)
    };

    view! {
        <section class="tabs-demo">
            <div class="panel">
                <h2>"Interruptible underline"</h2>
                <p>
                    "Click while the underline is moving. The animation reads its current progress and"
                    "retargets smoothly to the next tab instead of snapping."
                </p>
                <div class="list">
                    <span class="chip">"progress aware"</span>
                    <span class="chip">"spring retarget"</span>
                    <span class="chip">"mid-flight updates"</span>
                </div>
            </div>

            <div class="tabs-card">
                <div class="tab-list" node_ref=tabs_ref>
                    <button
                        node_ref=tab_refs[0]
                        class="tab-button"
                        class:active=move || active_tab.get() == 0
                        on:click=move |_| active_tab.set(0)
                    >
                        "Overview"
                    </button>
                    <button
                        node_ref=tab_refs[1]
                        class="tab-button"
                        class:active=move || active_tab.get() == 1
                        on:click=move |_| active_tab.set(1)
                    >
                        "Motion"
                    </button>
                    <button
                        node_ref=tab_refs[2]
                        class="tab-button"
                        class:active=move || active_tab.get() == 2
                        on:click=move |_| active_tab.set(2)
                    >
                        "Tokens"
                    </button>
                    <button
                        node_ref=tab_refs[3]
                        class="tab-button"
                        class:active=move || active_tab.get() == 3
                        on:click=move |_| active_tab.set(3)
                    >
                        "Notes"
                    </button>
                    <MotionDiv
                        class="tab-underline"
                        initial=MotionStyle::new().x(0.0).width(0.0)
                        animate=underline_style
                        transition=Transition::new().duration_ms(0)
                    ></MotionDiv>
                </div>
                <div class="tab-panel">
                    {move || match active_tab.get() {
                        0 => view! { <p>"Overview content slides in smoothly."</p> }.into_any(),
                        1 => view! { <p>"Motion settings react mid-flight."</p> }.into_any(),
                        2 => view! { <p>"Tokens shift without snapping."</p> }.into_any(),
                        _ => view! { <p>"Notes keep the underline moving."</p> }.into_any(),
                    }}
                </div>
            </div>
        </section>
    }
}
