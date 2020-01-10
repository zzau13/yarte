use rand::seq::SliceRandom;
use yarte_wasm_app::*;

use codegen::zip_with_spaces;

use crate::{app::NonKeyed, row::Row};
use std::mem;
use wasm_bindgen::UnwrapThrowExt;

// TODO:

#[rustfmt::skip]
zip_with_spaces!(
    [
    "pretty", "large", "big", "small", "tall", "short", "long", "handsome", "plain",
    "quaint","clean", "elegant", "easy", "angry", "crazy", "helpful", "mushy", "odd",
    "unsightly", "adorable", "important", "inexpensive", "cheap", "expensive", "fancy",
    ],
    [
    "red", "yellow", "blue", "green", "pink", "brown", "purple", "brown", "white", "black",
    "orange",
    ],
    [
    "table", "chair", "house", "bbq", "desk", "car", "pony", "cookie", "sandwich", "burger",
    "pizza", "mouse", "keyboard",
    ]
);

pub struct Create;

impl Message for Create {}

impl Handler<Create> for NonKeyed {
    fn handle(&mut self, _msg: Create, mb: &Mailbox<Self>) {
        for _ in 0..1000 {
            let id = self.id;
            self.id += 1;
            self.data.push(Row {
                id,
                label: ZIPPED.choose(&mut self.rng).unwrap().to_string(),
            })
        }
        self.t_root |= 0b0000_0001;
    }
}

pub struct Create10;

impl Message for Create10 {}

impl Handler<Create10> for NonKeyed {
    fn handle(&mut self, _msg: Create10, mb: &Mailbox<Self>) {
        for _ in 0..10_000 {
            let id = self.id;
            self.id += 1;
            self.data.push(Row {
                id,
                label: ZIPPED.choose(&mut self.rng).unwrap().to_string(),
            })
        }
        self.t_root |= 0b0000_0001;
    }
}

pub struct Append;

impl Message for Append {}

impl Handler<Append> for NonKeyed {
    fn handle(&mut self, _msg: Append, mb: &Mailbox<Self>) {
        for _ in 0..1000 {
            let id = self.id;
            self.id += 1;
            self.data.push(Row {
                id,
                label: ZIPPED.choose(&mut self.rng).unwrap().to_string(),
            })
        }
        self.t_root |= 0b0000_0001;
    }
}

pub struct Update;

impl Message for Update {}

impl Handler<Update> for NonKeyed {
    fn handle(&mut self, _msg: Update, mb: &Mailbox<Self>) {}
}

pub struct Clear;

impl Message for Clear {}

impl Handler<Clear> for NonKeyed {
    fn handle(&mut self, _msg: Clear, mb: &Mailbox<Self>) {
        self.data.clear();
        self.t_root |= 0b0000_0001;
    }
}

pub struct Swap;

impl Message for Swap {}

impl Handler<Swap> for NonKeyed {
    fn handle(&mut self, _msg: Swap, mb: &Mailbox<Self>) {
        if self.data.len() < 999 {
            return;
        }

        self.data.swap(1, 998);
        self.tbody_children[1].update(&self.data[1], mb);
        self.tbody_children[998].update(&self.data[998], mb);
    }
}

pub struct Select(pub u32);

impl Message for Select {}

impl Handler<Select> for NonKeyed {
    fn handle(&mut self, msg: Select, mb: &Mailbox<Self>) {
        self.t_root |= 0b0000_0010;

        if let Some(t) = self.selected {
            if t == msg.0 {
                self.selected = None;
                return;
            }
        }

        self.selected = Some(msg.0);
    }
}

pub struct Delete(pub u32);

impl Message for Delete {}

impl Handler<Delete> for NonKeyed {
    fn handle(&mut self, msg: Delete, mb: &Mailbox<Self>) {
        self.t_root |= 0b0000_0011;

        if let Some(selected) = self.selected {
            if msg.0 == selected {
                self.selected = None;
            }
        }
        self.data
            .iter()
            .position(|x| x.id == msg.0)
            .map(|x| self.data.remove(x))
            .unwrap_throw();
    }
}
