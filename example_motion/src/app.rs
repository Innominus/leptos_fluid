use leptos::prelude::*;

use crate::components::{
    CardsSection, ControllerSection, FlipGroupSection, FlipHeroSection, FlipSection, FooterSection,
    HeroSection, IslandSection, PerfSection, SpringFollowSection, StaggeredChipsSection,
    TabsSection, TimelineSection,
};

#[component]
pub fn App() -> impl IntoView {
    let pulse = RwSignal::new(true);
    let card_focus = RwSignal::new(false);

    view! {
        <main class="page">
            <HeroSection pulse card_focus />
            <CardsSection card_focus />
            <ControllerSection />
            <TabsSection />
            <TimelineSection />
            <FlipSection />
            <FlipHeroSection />
            <FlipGroupSection />
            <IslandSection />
            <SpringFollowSection />
            <StaggeredChipsSection pulse />
            <PerfSection />
            <FooterSection />
        </main>
    }
}
