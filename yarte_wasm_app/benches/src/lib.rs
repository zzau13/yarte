#[macro_use]
extern crate serde_derive;

use wasm_bindgen::prelude::*;

use yarte_wasm_app::App;

use crate::app::NonKeyed;

mod app;
mod handler;
mod row;

#[wasm_bindgen(start)]
pub fn start() {
    let app = NonKeyed::start_default();
    app.hydrate();
}
