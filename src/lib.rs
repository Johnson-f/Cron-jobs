pub mod app;
pub mod context;
pub mod client;
pub mod config;
pub mod ui;

// #[cfg(not(target_arch = "wasm32"))]
pub mod server;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
