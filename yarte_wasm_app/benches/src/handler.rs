use std::cmp::min;

use wasm_bindgen::UnwrapThrowExt;

use rand::seq::SliceRandom;
use yarte_wasm_app::*;

use codegen::zip_with_spaces;

use crate::{app::NonKeyed, row::Row};

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

impl NonKeyed {
    fn run_n(&mut self, n: usize) {
        let update_n = min(n, self.data.len());

        for i in 0..update_n {
            let mut row = &mut self.data[i];
            row.id = self.id + i as u32;
            row.label = ZIPPED.choose(&mut self.rng).unwrap().to_string();
            self.tbody_children[i].t_root |= 0b0000_0011
        }

        self.id += update_n as u32;

        for i in update_n..n {
            self.data.push(Row {
                id: self.id + i as u32,
                label: ZIPPED.choose(&mut self.rng).unwrap().to_string(),
            });
        }

        self.id += (n - update_n) as u32;
        self.selected = None;
        self.t_root |= 0b0000_0011;
    }
}

pub struct Create;

impl Message for Create {}

impl Handler<Create> for NonKeyed {
    fn handle(&mut self, _msg: Create, _mb: &Mailbox<Self>) {
        self.run_n(1_000);
    }
}

pub struct Create10;

impl Message for Create10 {}

impl Handler<Create10> for NonKeyed {
    fn handle(&mut self, _msg: Create10, _mb: &Mailbox<Self>) {
        self.run_n(10_000);
    }
}

pub struct Append;

impl Message for Append {}

impl Handler<Append> for NonKeyed {
    fn handle(&mut self, _msg: Append, _mb: &Mailbox<Self>) {
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
    fn handle(&mut self, _msg: Update, _mb: &Mailbox<Self>) {
        let step = 10;
        for i in (0..(self.data.len() / step)).map(|x| x * step) {
            self.data[i].label.push_str(" !!!");
            self.tbody_children[i].t_root |= 0b0000_0001;
        }

        self.t_root |= 0b0000_0001;
    }
}

pub struct Clear;

impl Message for Clear {}

impl Handler<Clear> for NonKeyed {
    fn handle(&mut self, _msg: Clear, _mb: &Mailbox<Self>) {
        self.data.clear();
        self.t_root |= 0b0000_0001;
    }
}

pub struct Swap;

impl Message for Swap {}

impl Handler<Swap> for NonKeyed {
    fn handle(&mut self, _msg: Swap, _mb: &Mailbox<Self>) {
        if self.data.len() < 999 {
            return;
        }

        self.data.swap(1, 998);
        self.tbody_children[1].t_root = 0xFF;
        self.tbody_children[998].t_root= 0xFF;
        self.t_root |= 0b0000_0001;
    }
}

pub struct Select(pub u32);

impl Message for Select {}

impl Handler<Select> for NonKeyed {
    fn handle(&mut self, msg: Select, _mb: &Mailbox<Self>) {
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
    fn handle(&mut self, msg: Delete, _mb: &Mailbox<Self>) {

        if let Some(selected) = self.selected {
            if msg.0 == selected {
                self.selected = None;
                self.t_root |= 0b0000_0010;
            }
        }
        self.data
            .iter()
            .position(|x| x.id == msg.0)
            .map(|x| {
                self.data.remove(x);
                for x in self.tbody_children[x..].iter_mut() {
                    x.t_root = 0xFF;
                }
            })
            .unwrap_throw();
        self.t_root |= 0b0000_0001;
    }
}
