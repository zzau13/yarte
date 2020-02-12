use wasm_bindgen::prelude::*;

use yarte::Template;

use model::Fortune;

#[derive(Template)]
#[template(path = "fortune.hbs", mode = "wasm", print = "code")]
#[msg(pub enum Msg {
    Clear,
    Add,
    Delete(u32),
})]
pub struct Test {
    fortunes: Vec<Fortune>,
    head: String,
    count: u32,
    black_box: <Self as Template>::BlackBox,
}

fn clear(app: &mut Test, _addr: &yarte::Addr<Test>) {
    app.fortunes.clear();
    // TODO: macro
    app.black_box.t_root |= 1u8;
}

fn delete(app: &mut Test, id: u32, _addr: &yarte::Addr<Test>) {
    let index = app.fortunes.iter().position(|x| x.id == id).unwrap();
    app.fortunes.remove(index);
    // TODO: macro
    // TODO: when index
    app.black_box.__ytable__1.remove(index);
}

fn add(app: &mut Test, _addr: &yarte::Addr<Test>) {
    let id = app.count;
    app.count += 1;
    app.fortunes.push(Fortune {id, ..Default::default() } );
    // TODO: macro
    app.black_box.t_root |= 1u8;
}

#[wasm_bindgen(start)]
pub fn start() {
    let app = Test::start_default();
    // Safe when only is used one time
    unsafe {
        app.hydrate();
    }
}
