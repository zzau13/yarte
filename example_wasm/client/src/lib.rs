use wasm_bindgen::prelude::*;

use yarte::Template;

use model::Fortune;

#[derive(Template)]
#[template(path = "fortune.hbs", mode = "wasm", print = "code")]
#[msg(pub enum Msg {
    Clear,
})]
pub struct Test {
    fortunes: Vec<Fortune>,
    head: String,
    black_box: <Self as Template>::BlackBox,
}

fn clear(app: &mut Test, _addr: &yarte::Addr<Test>) {
    app.fortunes.clear();
    app.black_box.t_root = 0xFF;
}

#[wasm_bindgen(start)]
pub fn start() {
    let app = Test::start_default();
    app.send(Msg::Clear);
    unsafe { app.hydrate(); }
}
