use leptos::prelude::*;

use crate::components::{
    FooterSection, HeroSection, OnceRevealSection, PureCallbackSection, ScrubCardSection,
    TimelineScrubSection, TimelineToggleSection,
};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <main class="page">
            <HeroSection />
            <ScrubCardSection />
            <TimelineToggleSection />
            <TimelineScrubSection />
            <OnceRevealSection />
            <PureCallbackSection />
            <FooterSection />
        </main>
    }
}