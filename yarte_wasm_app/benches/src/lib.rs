use wasm_bindgen_test::*;
use web_sys::console;

use yarte_wasm_app::*;

#[derive(Default)]
struct Bench;
impl App for Bench {
    type BlackBox = ();
}

struct Msg;
impl Message for Msg {}

impl Handler<Msg> for Bench {
    fn handle(&mut self, _: Msg, _: &Mailbox<Self>) {}
}

struct MsgR(usize);
impl Message for MsgR {}
impl Handler<MsgR> for Bench {
    fn handle(&mut self, msg: MsgR, mb: &Mailbox<Self>) {
        for _ in 0..msg.0 {
            mb.send(Msg);
        }
    }
}

pub struct Timer<'a> {
    name: &'a str,
}

impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Timer<'a> {
        console::time_with_label(name);
        Timer { name }
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        console::time_end_with_label(self.name);
    }
}

#[wasm_bindgen_test]
fn test() {
    let app = Bench::start_default();
    let _timer = Timer::new("1_000 sends in:");
    for _ in 0..1_000 {
        app.send(Msg);
    }
}

#[wasm_bindgen_test]
fn test_inner() {
    let app = Bench::start_default();
    let _timer = Timer::new("1_000 sends inner in:");
    app.send(MsgR(1_000));
}
