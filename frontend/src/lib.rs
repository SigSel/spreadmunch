use wasm_bindgen::prelude::*;

mod app;
mod cross_filter;
mod filter_panel;
mod settings;
mod table;

#[wasm_bindgen(start)]
fn main() {
    console_error_panic_hook::set_once();
    dwind::stylesheet();
    dominator::append_dom(&dominator::body(), app::App::render(app::App::new()));
}
