use leptos::prelude::*;

use crate::examples::{
    AutoSizeExample, BuilderCardExample, MacroStateExample, ResolverDeckExample,
    SpringRetargetExample, SpringTimelineExample, TimelineBuilderExample, TimelineMacroExample,
};

const HERO_POINTS: [(&str, &str); 5] = [
    (
        "Typed builders",
        "Method-complete controller and timeline setup.",
    ),
    (
        "Declarative macros",
        "controller!, when!, and timeline! on plain nodes.",
    ),
    (
        "Dynamic resolvers",
        "Retarget controllers without wrapper components.",
    ),
    (
        "Timeline control",
        "Builder and macro sequences with pause, restart, and stop.",
    ),
    (
        "Auto size helpers",
        "Animate height and width changes from ResizeObserver-driven measurements.",
    ),
];

#[component]
pub fn App() -> impl IntoView {
    let highlights = HERO_POINTS
        .into_iter()
        .map(|(label, body)| {
            view! {
                <div class="summary-card">
                    <p class="summary-label">{label}</p>
                    <p class="summary-body">{body}</p>
                </div>
            }
        })
        .collect_view();

    view! {
        <main class="page">
            <header class="hero" data-testid="controller-hero">
                <p class="eyebrow">"Leptos Fluid Motion"</p>
                <h1>"Controller-first motion lab"</h1>
                <p class="lead">
                    "Plain elements, stable builder APIs, and declarative macros all driving the same controller runtime."
                </p>
            </header>

            <section class="summary-grid">{highlights}</section>

            <section class="demo-grid">
                <BuilderCardExample />
                <MacroStateExample />
                <ResolverDeckExample />
                <SpringRetargetExample />
                <SpringTimelineExample />
                <TimelineBuilderExample />
                <TimelineMacroExample />
                <AutoSizeExample />
            </section>
        </main>
    }
}
