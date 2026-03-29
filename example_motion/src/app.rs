use leptos::prelude::*;

use crate::components::{
    AutoLayoutSection, FlipBoardSection, FlipCardSection, FooterSection, HeroSection, PerfSection,
    SpringFollowSection, StyleLabSection, TimelineStudioSection, WrapperGallerySection,
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
            <SpringFollowSection />
            <FlipCardSection />
            <FlipBoardSection />
            <PerfSection />
            <FooterSection />
        </main>
    }
}
