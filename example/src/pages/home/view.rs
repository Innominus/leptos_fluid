use leptos::prelude::*;

use crate::components::common::PageShell;

#[component]
pub fn Home() -> impl IntoView {
    view! {
        <PageShell>
            <div class="inline-block w-full h-full text-center text-white bg-teal-500">"Home"</div>
        </PageShell>
    }
}
