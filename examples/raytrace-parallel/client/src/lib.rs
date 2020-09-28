#![feature(extern_types, box_syntax)]
#![allow(clippy::manual_non_exhaustive)]
use std::fmt;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement};
use yarte_wasm_app::{run, App, A};

use utils::console_log;

use crate::app::RayTracing;
use crate::handler::{
    enable_interface, end_render, error, paint, start_render, unsafe_paint, update_concurrency,
    update_time,
};
use crate::scene::{Img, UnsafeImg};

mod app;
mod handler;
mod scene;

enum Msg {
    UpdateConcurrency(String),
    EndRender(f64),
    Error(Box<dyn fmt::Display>),
    Paint(Img),
    StartRender,
    UnsafePaint(UnsafeImg),
    UpdateTime(f64),
}

impl App for RayTracing {
    type BlackBox = ();
    type Message = Msg;

    fn __hydrate(&mut self, addr: A<Self>) {
        let cl = Closure::wrap(Box::new(move |event: Event| {
            event.prevent_default();
            addr.send(Msg::StartRender)
        }) as Box<dyn Fn(Event)>);
        self.button
            .add_event_listener_with_callback("click", cl.as_ref().unchecked_ref())
            .unwrap();
        cl.forget();

        let cl = Closure::wrap(Box::new(move |event: Event| {
            let target = event.target().unwrap().unchecked_into::<HtmlInputElement>();
            addr.send(Msg::UpdateConcurrency(target.value()))
        }) as Box<dyn Fn(Event)>);
        self.concurrency
            .add_event_listener_with_callback("input", cl.as_ref().unchecked_ref())
            .unwrap();
        cl.forget();
        self.button.set_inner_text("Render!");
        enable_interface(self);
        console_log!("wasm app is ready");
    }

    fn __dispatch(&mut self, msg: Self::Message, addr: A<Self>) {
        match msg {
            Msg::EndRender(start) => end_render(self, start),
            Msg::Error(s) => error(self, s),
            Msg::Paint(image) => paint(self, image),
            Msg::StartRender => start_render(self, addr),
            Msg::UnsafePaint(image) => unsafe { unsafe_paint(self, image) },
            Msg::UpdateConcurrency(val) => update_concurrency(self, val),
            Msg::UpdateTime(start) => update_time(self, start),
        }
    }
}

#[wasm_bindgen]
pub fn start() {
    console_error_panic_hook::set_once();
    let _addr = run!(RayTracing);
}
