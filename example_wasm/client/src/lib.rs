extern crate wee_alloc;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::HtmlInputElement;

use model::Fortune;
use yarte_wasm_app::{Addr, App, run};

#[derive(App)]
#[template(path = "fortune", print = "code")]
#[msg(pub enum Msg {
    Clear,
    Add,
    Add10,
    Add20,
    Delete(u32),
})]
pub struct Test {
    fortunes: Vec<Fortune>,
    head: String,
    count: u32,
    // TODO: in template with update function
    #[inner("build_foo")]
    foo: HtmlInputElement,
    black_box: <Self as App>::BlackBox,
}

// TODO: in template with update function
#[inline]
fn build_foo() -> HtmlInputElement {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("foo")
        .unwrap()
        .unchecked_into()
}

#[inline]
fn clear(app: &mut Test, _addr: &Addr<Test>) {
    app.fortunes.clear();
    // TODO: macro
    app.black_box.t_root |= 1u8;
}

#[inline]
fn delete(app: &mut Test, id: u32, _addr: &Addr<Test>) {
    let index = app.fortunes.iter().position(|x| x.id == id).unwrap();
    app.fortunes.remove(index);
    // TODO: macro
    // TODO: when index
    app.black_box.__ytable__1.remove(index);
}

#[inline]
fn add(app: &mut Test, _addr: &Addr<Test>) {
    let message = app.foo.value();
    let id = app.count;
    app.count += 1;
    app.fortunes.push(Fortune {
        id,
        message,
        ..Default::default()
    });
    // TODO: macro
    app.black_box.t_root |= 1u8;
}

#[inline]
fn add10(app: &mut Test, _addr: &Addr<Test>) {
    for _ in 0..10 {
        let id = app.count;
        app.count += 1;
        app.fortunes.push(Fortune {
            id,
            ..Default::default()
        });
    }
    // TODO: macro
    app.black_box.t_root |= 1u8;
}

#[inline]
fn add20(app: &mut Test, _addr: &Addr<Test>) {
    for _ in 0..20 {
        let id = app.count;
        app.count += 1;
        app.fortunes.push(Fortune {
            id,
            ..Default::default()
        });
    }
    // TODO: macro
    app.black_box.t_root |= 1u8;
}

#[wasm_bindgen(start)]
pub fn start() {
    let _app = run!(Test);
}
