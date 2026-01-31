use leptos::mount;

use crate::app::App;

mod app;
mod components;

pub fn main() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    mount::mount_to_body(App);
}
