use dominator::{append_dom, body, html};
use dwind::prelude::*;
use dwind_macros::dwclass;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
fn main() {
    console_error_panic_hook::set_once();
    dwind::stylesheet();

    append_dom(
        &body(),
        html!("div", {
            .dwclass!("w-full h-screen flex items-center justify-center bg-gray-900")
            .child(html!("h1", {
                .dwclass!("text-4xl font-bold text-white")
                .text("Spreadmunch")
            }))
        }),
    );
}
