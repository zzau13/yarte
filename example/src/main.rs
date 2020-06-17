#![allow(clippy::uninit_assumed_init)]
use std::collections::HashMap;
use std::io::{stdout, Write};
use std::thread;

use std::mem::MaybeUninit;
use yarte::{Template, TemplateBytesMin, TemplateFixedMin, TemplateMin};

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

#[derive(TemplateBytesMin)]
#[template(path = "index_fixed")]
struct IndexTemplateB {
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

    unsafe {
        IndexTemplateF {
            query: query.clone(),
        }
        .call(&mut [MaybeUninit::uninit(); 2048])
    }
    .and_then(|b| {
        println!("\nFixed Min:");
        stdout().lock().write_all(b).ok()?;
        println!();
        Some(())
    })
    .unwrap();

    let buf = IndexTemplateB { query }.call(2048).unwrap();
    thread::spawn(move || {
        println!("\nBytes Min:");
        stdout().lock().write_all(&buf).unwrap();
        println!();
    })
    .join()
    .unwrap();
}
