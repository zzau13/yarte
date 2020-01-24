#![cfg(target_arch = "wasm32")]

use wasm_bindgen::prelude::*;

use yarte::Template;

use model::Fortune;

#[derive(Template)]
#[template(path = "fortune.hbs", print = "code", mode = "wasm")]
#[msg(pub enum Msg {
    AnyMsg(usize),
    NewPoint { a: usize, b: usize },
    Ping,
    #[path::func]
    Other,
})]
pub struct Test {
    fortunes: Vec<Fortune>,
    head: String,
    #[inner]
    bar: usize,
    #[inner("foo")]
    barr: usize,
    black_box: <Self as Template>::BlackBox,
}

#[inline]
fn foo() -> usize {
    1
}

#[inline]
fn any_msg(_app: &mut Test, _0: usize, _addr: &yarte::Addr<Test>) {}
#[inline]
fn new_point(_app: &mut Test, _a: usize, _b: usize, _addr: &yarte::Addr<Test>) {}
#[inline]
fn ping(_app: &mut Test, _addr: &yarte::Addr<Test>) {}
mod path {
    use super::*;
    #[inline]
    pub(super) fn func(_app: &mut Test, _addr: &yarte::Addr<Test>) {}
}
