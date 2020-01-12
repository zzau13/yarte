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
    #[inline]
    fn run_n(&mut self, n: usize) {
        let update_n = min(n, self.data.len());

        // ?? #macro mark_data_update(for i in
        for (i, (dom, row)) in self
            .tbody_children
            .iter_mut()
            .zip(self.data.iter_mut())
            .enumerate()
            .take(update_n)
        {
            row.id = self.id + i as u32;
            row.label = (*ZIPPED.choose(&mut self.rng).unwrap()).to_string();
            // or #macro mark_data_
            dom.t_root = 0xFF;
            // /macro
        }
        // /macro

        for i in update_n..n {
            self.data.push(Row {
                id: self.id + i as u32,
                label: (*ZIPPED.choose(&mut self.rng).unwrap()).to_string(),
            });
        }

        // #macro mark_data
        self.t_root |= 0b0000_0001;
        // /macro

        self.id += n as u32;

        // #macro set_selected
        self.selected = None;
        self.t_root |= 0b0000_0010;
        // /macro
    }
}

pub struct Create;

impl Message for Create {}

impl Handler<Create> for NonKeyed {
    fn handle(&mut self, _msg: Create, _mb: &Addr<Self>) {
        self.run_n(1_000);
    }
}

pub struct Create10;

impl Message for Create10 {}

impl Handler<Create10> for NonKeyed {
    fn handle(&mut self, _msg: Create10, _mb: &Addr<Self>) {
        self.run_n(10_000);
    }
}

pub struct Append;

impl Message for Append {}

impl Handler<Append> for NonKeyed {
    fn handle(&mut self, _msg: Append, _mb: &Addr<Self>) {
        let n = 1000;
        for i in 0..n {
            self.data.push(Row {
                id: self.id + i,
                label: (*ZIPPED.choose(&mut self.rng).unwrap()).to_string(),
            })
        }
        // id its not present in template and haven't a marker
        self.id += n;
        // #macro mark_data
        self.t_root |= 0b0000_0001;
        // /macro
    }
}

pub struct Update;

impl Message for Update {}

impl Handler<Update> for NonKeyed {
    fn handle(&mut self, _msg: Update, _mb: &Addr<Self>) {
        for (row, dom) in self
            .data
            .iter_mut()
            .zip(self.tbody_children.iter_mut())
            .step_by(10)
        {
            row.label.push_str(" !!!");
            // #macro mark_data_label
            dom.t_root |= 0b0000_0001;
            // /macro
        }

        // #macro mark_data
        self.t_root |= 0b0000_0001;
        // /macro
    }
}

pub struct Clear;

impl Message for Clear {}

impl Handler<Clear> for NonKeyed {
    fn handle(&mut self, _msg: Clear, _mb: &Addr<Self>) {
        // #macro data_swap
        self.data.clear();
        self.t_root |= 0b0000_0001;
        // /macro
    }
}

pub struct Swap;

impl Message for Swap {}

impl Handler<Swap> for NonKeyed {
    fn handle(&mut self, _msg: Swap, _mb: &Addr<Self>) {
        if self.data.len() < 999 {
            return;
        }

        // #macro data_swap
        self.data.swap(1, 998);
        self.tbody_children[1].t_root = 0xFF;
        self.tbody_children[998].t_root = 0xFF;
        self.t_root |= 0b0000_0001;
        // /macro
    }
}

pub struct Select(pub u32);

impl Message for Select {}

impl Handler<Select> for NonKeyed {
    fn handle(&mut self, msg: Select, _mb: &Addr<Self>) {
        if let Some(t) = self.selected {
            if t == msg.0 {
                // #macro set_selected
                self.selected = None;
                self.t_root |= 0b0000_0010;
                // /macro
                return;
            }
        }

        // #macro set_selected
        self.selected = Some(msg.0);
        self.t_root |= 0b0000_0010;
        // /macro
    }
}

pub struct Delete(pub u32);

impl Message for Delete {}

impl Handler<Delete> for NonKeyed {
    fn handle(&mut self, msg: Delete, _mb: &Addr<Self>) {
        if let Some(selected) = self.selected {
            if msg.0 == selected {
                // #macro set_selected
                self.selected = None;
                self.t_root |= 0b0000_0010;
                // /macro
            }
        }
        self.data
            .iter()
            .position(|x| x.id == msg.0)
            .map(|x| {
                // #macro remove_data(_self, i: usize)
                self.tbody.remove_child(&self.tbody_children.remove(x).root).unwrap_throw();
                self.data.remove(x);
                self.t_root |= 0b0000_0001;
                // /macro
            })
            .unwrap_throw();
    }
}
