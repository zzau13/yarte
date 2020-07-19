use std::collections::HashMap;
use std::io::{stdout, Write};
use std::thread;

use bytes::BytesMut;
use std::mem::MaybeUninit;
use yarte::{ywrite, Template, TemplateBytesMin, TemplateFixedMin, TemplateMin};

#[derive(Template)]
#[template(path = "index")]
struct IndexTemplate<'a> {
    query: &'a HashMap<&'static str, &'static str>,
}

#[derive(TemplateMin)]
#[template(path = "index")]
struct IndexTemplateMin<'a> {
    query: &'a HashMap<&'static str, &'static str>,
}

#[derive(TemplateFixedMin)]
#[template(path = "index_fixed")]
struct IndexTemplateF<'a> {
    query: &'a HashMap<&'static str, &'static str>,
}

#[derive(TemplateBytesMin)]
#[template(path = "index_bytes")]
struct IndexTemplateB<'a> {
    query: Option<(&'a str, &'a str)>,
}

fn main() {
    let mut query = HashMap::new();
    query.insert("name", "new");
    query.insert("lastname", "user");

    println!("Fmt:\n{}", IndexTemplate { query: &query });
    println!("\nFmt Min:\n{}", IndexTemplateMin { query: &query });

    unsafe {
        TemplateFixedMin::call(
            &IndexTemplateF { query: &query },
            &mut [MaybeUninit::uninit(); 2048],
        )
    }
    .and_then(|b| {
        println!("\nFixed Min:");
        stdout().lock().write_all(b).ok()?;
        println!();
        Some(())
    })
    .unwrap();

    let buf = TemplateBytesMin::ccall::<BytesMut>(
        IndexTemplateB {
            query: query
                .get("name")
                .and_then(|name| query.get("lastname").map(|lastname| (*name, *lastname))),
        },
        2048,
    );
    thread::spawn(move || {
        println!("\nBytes Min:");
        stdout().lock().write_all(&buf).unwrap();
        println!();
    })
    .join()
    .unwrap();

    let mut buf = BytesMut::with_capacity(2048);
    let query = query
        .get("name")
        .and_then(|name| query.get("lastname").map(|lastname| (*name, *lastname)));
    ywrite!(buf, "{{> index_bytes }}");
    println!("\nywrite:");
    stdout().lock().write_all(&buf.freeze()).unwrap();
    println!();
}
