use std::cmp::min;

use rand::seq::SliceRandom;
use yarte_wasm_app::*;

use codegen::zip_with_spaces;

use crate::{app::NonKeyed, row::Row};

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

fn run_n(app: &mut NonKeyed, n: usize) {
    let update_n = min(n, app.data.len());

    // ?? #macro mark_data_update(for i in
    for (i, (dom, row)) in app
        .tbody_children
        .iter_mut()
        .zip(app.data.iter_mut())
        .enumerate()
        .take(update_n)
    {
        row.id = app.id + i as usize;
        row.label = (*ZIPPED.choose(&mut app.rng).unwrap()).to_string();
        // or #macro mark_data_
        dom.t_root = 0xFF;
        // /macro
    }
    // /macro

    for i in update_n..n {
        app.data.push(Row {
            id: app.id + i as usize,
            label: (*ZIPPED.choose(&mut app.rng).unwrap()).to_string(),
        });
    }

    // #macro mark_data
    app.t_root |= 0b0000_0001;
    // /macro

    app.id += n as usize;

    // #macro set_selected
    app.selected = None;
    app.t_root |= 0b0000_0010;
    // /macro
}

pub enum Msg {
    Append,
    Clear,
    Create,
    Create10,
    Delete(usize),
    Select(usize),
    Swap,
    Update,
}

#[inline]
pub fn create(app: &mut NonKeyed, _mb: &Addr<NonKeyed>) {
    run_n(app, 1_000);
}

#[inline]
pub fn create_10(app: &mut NonKeyed, _mb: &Addr<NonKeyed>) {
    run_n(app, 10_000);
}

#[inline]
pub fn append(app: &mut NonKeyed, _mb: &Addr<NonKeyed>) {
    let n = 1000;
    for i in 0..n {
        app.data.push(Row {
            id: app.id + i,
            label: (*ZIPPED.choose(&mut app.rng).unwrap()).to_string(),
        })
    }
    // id its not present in template and haven't a marker
    app.id += n;
    // #macro mark_data
    app.t_root |= 0b0000_0001;
    // /macro
}

#[inline]
pub fn update(app: &mut NonKeyed, _mb: &Addr<NonKeyed>) {
    for (row, dom) in app
        .data
        .iter_mut()
        .zip(app.tbody_children.iter_mut())
        .step_by(10)
    {
        row.label.push_str(" !!!");
        // #macro mark_data_label
        dom.t_root |= 0b0000_0001;
        // /macro
    }

    // #macro mark_data
    app.t_root |= 0b0000_0001;
    // /macro
}

#[inline]
pub fn clear(app: &mut NonKeyed, _mb: &Addr<NonKeyed>) {
    // #macro data_swap
    app.data.clear();
    app.t_root |= 0b0000_0001;
    // /macro
}

#[inline]
pub fn swap(app: &mut NonKeyed, _mb: &Addr<NonKeyed>) {
    if app.data.len() < 999 {
        return;
    }

    // #macro data_swap
    app.data.swap(1, 998);
    app.tbody_children[1].t_root = 0xFF;
    app.tbody_children[998].t_root = 0xFF;
    app.t_root |= 0b0000_0001;
    // /macro
}

#[inline]
pub fn select(app: &mut NonKeyed, msg: usize, _mb: &Addr<NonKeyed>) {
    if let Some(t) = app.selected {
        if t == msg {
            // #macro set_selected
            app.selected = None;
            app.t_root |= 0b0000_0010;
            // /macro
            return;
        }
    }

    // #macro set_selected
    app.selected = Some(msg);
    app.t_root |= 0b0000_0010;
    // /macro
}

#[inline]
pub fn delete(app: &mut NonKeyed, msg: usize, _mb: &Addr<NonKeyed>) {
    if let Some(position) = app.data.iter().position(|x| x.id == msg) {
        // #macro remove_data(_app, i: usize)
        app.data.remove(position);
        app.tbody_children.remove(position).root.remove();
        // TODO: if index
        app.t_root |= 0b0000_0001;
        // /macro

        // deselect
        if let Some(selected) = app.selected {
            if msg == selected {
                // No propagate change to render
                app.selected = None;
            }
        }
    }
}
