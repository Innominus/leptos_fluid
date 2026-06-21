use leptos::prelude::*;

use crate::components::{
    ColorMorphSection, CounterSection, FooterSection, HorizontalGallerySection,
    ImageRevealSection, MagneticCtaSection, PerspectiveTiltSection, StickyHeroSection,
    StaggerGridSection, TextMaskRevealSection, VelocityMarqueeSection,
};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <main class="page">
            <StickyHeroSection />
            <HorizontalGallerySection />
            <TextMaskRevealSection />
            <StaggerGridSection />
            <CounterSection />
            <ImageRevealSection />
            <PerspectiveTiltSection />
            <VelocityMarqueeSection />
            <ColorMorphSection />
            <MagneticCtaSection />
            <FooterSection />
        </main>
    }
}