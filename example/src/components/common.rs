use leptos::prelude::*;

// pub const BASE_PADDING: &str = "py-2 px-2 sm:px-14";

#[component]
pub fn PageShell(#[prop(optional)] class: &'static str, children: Children) -> impl IntoView {
    let delayed_class = RwSignal::new("");
    Effect::new(move || request_animation_frame(move || delayed_class.set(class)));
    view! {
        <section class=move || {
            "flex flex-1 flex-col h-full w-full items-center scroll-bar ".to_string()
                + delayed_class.get()
        }>{children()}</section>
    }
}

/// This component has a display of flex column
#[component]
pub fn AnimatedContainer(
    #[prop(optional)] class: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="fade-in-transition w-full flex flex-col ".to_string() + class>{children()}</div>
    }
}

#[component]
pub fn LoadingSpinner(#[prop(default = "loading-lg")] class: &'static str) -> impl IntoView {
    view! { <span class="fade-in-transition loading loading-spinner ".to_string() + class></span> }
}
