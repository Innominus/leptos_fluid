use leptos::prelude::*;

use crate::components::{
    CardsSection, FooterSection, HeroSection, IslandSection, PerfSection, PresenceSection,
    SpringFollowSection, StaggeredChipsSection, TabsSection,
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
            <PresenceSection />
            <IslandSection />
            <SpringFollowSection />
            <StaggeredChipsSection pulse />
            <PerfSection />
            <FooterSection />
        </main>
    }
}
