use leptos::prelude::*;

use crate::components::{
    CardsSection, FlipGroupSection, FlipSection, FooterSection, HeroSection, IslandSection,
    PerfSection, PresenceSection, PresenceSwapSection, SpringFollowSection, StaggeredChipsSection,
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
            <TabsSection />
            <TimelineSection />
            <FlipSection />
            <FlipGroupSection />
            <PresenceSection />
            <PresenceSwapSection />
            <IslandSection />
            <SpringFollowSection />
            <StaggeredChipsSection pulse />
            <PerfSection />
            <FooterSection />
        </main>
    }
}
