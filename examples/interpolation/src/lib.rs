#![deny(warnings)]

pub mod app;
leptos_i18n::load_locales!();

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use app::App;
    use leptos::*;
    console_error_panic_hook::set_once();

    leptos::mount_to_body(move || {
        view! {  <App/> }
    });
}
