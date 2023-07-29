#![cfg_attr(nightly, feature(proc_macro_hygiene, stmt_expr_attributes))]

use std::io::{stdout, Write};

use yarte::*;

struct Card<'a> {
    title: &'a str,
    body: &'a str,
}

fn _write_str(buffer: String) {
    stdout().lock().write_all(buffer.as_bytes()).unwrap();
}

fn main() {
    let my_card = Card {
        title: "My Title",
        body: "My Body",
    };
    let mut buffer = Vec::with_capacity(2048);
    #[html(buffer)]
    "{{> hello my_card }}";

    println!("Proc macro attribute");
    stdout().lock().write_all(&buffer).unwrap();
    println!();

    println!("Proc macro attribute auto");

    _write_str(
        #[html]
        "{{> hello my_card }}",
    );

    println!();

    let buffer: String = #[html]
    "{{> hello my_card }}";

    println!("Proc macro attribute auto");
    stdout().lock().write_all(buffer.as_bytes()).unwrap();
    println!();
}
