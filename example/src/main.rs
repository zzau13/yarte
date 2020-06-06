#![allow(clippy::uninit_assumed_init)]
use std::collections::HashMap;
use std::io::{stdout, Write};
use std::mem::MaybeUninit;

use yarte::{Template, TemplateFixed, TemplateMin};

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

#[derive(TemplateFixed)]
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
    let mut buf: [u8; 2048] = unsafe { MaybeUninit::uninit().assume_init() };
    let size = unsafe { IndexTemplateF { query }.call(&mut buf) }.unwrap();
    println!("\nFixed:");
    let _ = stdout().lock().write(&buf[..size]);
    println!()
}
