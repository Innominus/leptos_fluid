use leptos::prelude::*;

#[component]
pub fn FooterSection() -> impl IntoView {
    view! {
        <footer class="footer-panel">
            <div>
                <p class="kicker">"Next stop"</p>
                <h2>"Controller-first playground"</h2>
                <p>
                    "If you want the plain-node, builder, macro, and resolver-heavy story, open the dedicated controller example next."
                </p>
            </div>

            <div class="footer-notes">
                <p>"This page focuses on wrappers, style composition, springs, timelines, and FLIP."</p>
                <p>"The sibling example focuses on controller-first orchestration."</p>
            </div>
        </footer>
    }
}
