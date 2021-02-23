use std::collections::HashMap;
use std::io::{stdout, Write};

use yarte::*;

struct Card<'a> {
    title: &'a str,
    body: &'a str,
}

fn main() {
    let mut query = HashMap::new();
    query.insert("name", "new");
    query.insert("lastname", "user");

    let query = query
        .get("name")
        .and_then(|name| query.get("lastname").map(|lastname| (*name, *lastname)));

    let buf = auto!(ywrite_min!(String, "{{> index_bytes }}"));

    stdout().lock().write_all(buf.as_bytes()).unwrap();
    println!();

    let my_card = Card {
        title: "My Title",
        body: "My Body",
    };

    // Auto sized html
    let buf = auto!(ywrite_html!(String, "{{> hello my_card }}"));

    stdout().lock().write_all(buf.as_bytes()).unwrap();
    println!();
}
