#![allow(clippy::uninit_assumed_init)]
use std::collections::HashMap;
use std::io::{stdout, Write};
use std::thread;

use bytes::BytesMut;

use yarte::{Template, TemplateFixedMin, TemplateMin};

#[derive(Template)]
#[template(path = "index")]
struct IndexTemplate {
    query: HashMap<&'static str, &'static str>,
}

#[derive(TemplateMin)]
#[template(path = "index")]
struct IndexTemplateMin {
    query: HashMap<&'static str, &'static str>,
}

#[derive(TemplateFixedMin)]
#[template(path = "index_fixed")]
struct IndexTemplateF {
    query: HashMap<&'static str, &'static str>,
}

fn main() {
    let mut query = HashMap::new();
    query.insert("name", "new");
    query.insert("lastname", "user");

    println!(
        "Fmt:\n{}",
        IndexTemplate {
            query: query.clone()
        }
    );
    println!(
        "\nFmt Min:\n{}",
        IndexTemplateMin {
            query: query.clone()
        }
    );

    let mut buf = BytesMut::with_capacity(2048);
    unsafe {
        // Maybe uninit
        buf.set_len(2048);
        let size = IndexTemplateF { query }.call(&mut buf).unwrap();
        // bound init data
        buf.set_len(size);
    }
    // Freeze
    let buf = buf.freeze();
    // Send to another thread
    thread::spawn(move || {
        println!("\nFixed Min:");
        let _ = stdout().lock().write(&buf);
        println!()
    })
    .join()
    .unwrap();
}
