#![cfg_attr(nightly, feature(proc_macro_hygiene, stmt_expr_attributes))]
use std::collections::HashMap;
use std::io::{stdout, Write};

use yarte::*;

struct Card<'a> {
    title: &'a str,
    body: &'a str,
}

#[cfg(nightly)]
fn nightly(my_card: &Card) {
    let mut buffer= String::new();
    // TODO: auto when no pass args
    // TODO: bad statement when pass without let
    let _ = #[html(buffer)]
    "{{> hello my_card }}";

    println!("Proc macro attribute");
    stdout().lock().write_all(buffer.as_bytes()).unwrap();
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
    let buf = auto!(ywrite_min!(String, "{{> index }}"));

    println!("Proc macro minimal");
    stdout().lock().write_all(buf.as_bytes()).unwrap();
    println!();

    let my_card = Card {
        title: "My Title",
        body: "My Body",
    };

    // Auto sized html
    let buf = auto!(ywrite_html!(String, "{{> hello my_card }}"));
    println!("Proc macro auto");
    stdout().lock().write_all(buf.as_bytes()).unwrap();
    println!();

    #[cfg(nightly)]
    nightly(&my_card);
}
