use leptos::prelude::*;

use crate::components::common::PageShell;

#[component]
pub fn Home() -> impl IntoView {
    view! {
        <PageShell>
            <div
                style="min-height:1000px;"
                class="inline-block w-full text-center text-white bg-teal-500"
            >
                "Home"
            </div>
        </PageShell>
    }
}
