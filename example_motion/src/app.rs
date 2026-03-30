use leptos::prelude::*;

use crate::components::{
    AutoLayoutSection, FlipBoardSection, FlipCardSection, FooterSection, HeroSection, PerfSection,
    SpringFollowSection, SpringShowcaseSection, StyleLabSection, TimelineStudioSection,
    WrapperGallerySection,
};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <main class="page">
            <HeroSection />
            <WrapperGallerySection />
            <StyleLabSection />
            <AutoLayoutSection />
            <TimelineStudioSection />
            <SpringShowcaseSection />
            <SpringFollowSection />
            <FlipCardSection />
            <FlipBoardSection />
            <PerfSection />
            <FooterSection />
        </main>
    }
}
