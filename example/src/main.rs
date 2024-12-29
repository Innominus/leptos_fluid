use leptos::mount;
use leptos_fluid_example::app::App;

pub fn main() {
    console_error_panic_hook::set_once();

    mount::mount_to_body(App);
}
