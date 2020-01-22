#![allow(warnings)]
use serde::Deserialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use yarte::Template;

#[derive(Default, Deserialize)]
struct Fortune {
    id: i32,
    message: String,
}

#[derive(Template)]
#[template(path = "fortune.hbs", print = "code", mode = "wasm")]
#[msg(enum Msg {
    #[func]
    Any,
})]
struct Test {
    fortunes: Vec<Fortune>,
    head: String,
    black_box: <Self as Template>::BlackBox,
}

fn func(_app: &mut Test, _addr: &yarte::Addr<Test>) {}

#[wasm_bindgen_test]
fn test() {
    let app = Test::start_default();
}
