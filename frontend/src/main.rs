mod app;
mod db;
mod pages;
mod state;
mod store;

use app::App;
use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    _ = console_log::init_with_level(log::Level::Warn);
    mount_to_body(App);
}
