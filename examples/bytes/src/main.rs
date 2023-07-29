#![cfg_attr(nightly, feature(proc_macro_hygiene, stmt_expr_attributes))]
use std::collections::HashMap;
use std::io::{stdout, Write};

use yarte::*;

use bytes::BytesMut;

struct Card<'a> {
    title: &'a str,
    body: &'a str,
}

#[cfg(nightly)]
/// without comma or error
/// `message: stable/nightly mismatch`
fn _write_str(buffer: BytesMut) {
    stdout().lock().write_all(&buffer).unwrap();
}

#[cfg(nightly)]
fn nightly(my_card: &Card) {
    let mut buffer = BytesMut::new();
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

    let buffer: BytesMut = #[html]
    "{{> hello my_card }}";

    println!("Proc macro attribute auto");
    stdout().lock().write_all(&buffer).unwrap();
    println!();
}

fn main() {
    let mut query = HashMap::new();
    query.insert("name", "new");
    query.insert("lastname", "user");

    let query = query
        .get("name")
        .and_then(|name| query.get("lastname").map(|lastname| (*name, *lastname)));

    // Auto sized html minimal (Work in progress. Not use in production)
    let buf = auto!(ywrite!(BytesMut, "{{> index }}"));

    println!("Proc macro minimal");
    stdout().lock().write_all(&buf).unwrap();
    println!();

    let my_card = Card {
        title: "My Title",
        body: "My Body",
    };

    // Auto sized html
    let buf = auto!(ywrite_html!(BytesMut, "{{> hello my_card }}"));
    println!("Proc macro auto");
    stdout().lock().write_all(&buf).unwrap();
    println!();

    #[cfg(nightly)]
    nightly(&my_card);
}
