#![allow(clippy::uninit_assumed_init)]
#![cfg(all(feature = "fixed", feature = "bytes-buf", feature = "html-min"))]

use std::collections::HashMap;
use std::mem::MaybeUninit;

use yarte::{Bytes, BytesMut, TemplateBytesMin, TemplateFixedMin, TemplateMin};

#[derive(TemplateMin)]
#[template(path = "example/index")]
struct IndexTemplateMin<'a> {
    query: &'a HashMap<&'static str, &'static str>,
}

#[derive(TemplateFixedMin)]
#[template(path = "example/index_fixed")]
struct IndexTemplateF<'a> {
    query: &'a HashMap<&'static str, &'static str>,
}

#[derive(TemplateBytesMin)]
#[template(path = "example/index_bytes")]
struct IndexTemplateB<'a> {
    query: Option<(&'a str, &'a str)>,
}

#[test]
fn main() {
    let expected =
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>Actix web</title></head><body><h1>Hi, new user!</h1><p id=\"hi\" class=\"welcome\">Welcome</p></body></html>";
    let mut query = HashMap::new();
    query.insert("name", "new");
    query.insert("lastname", "user");

    assert_eq!(IndexTemplateMin { query: &query }.to_string(), expected);

    assert_eq!(
        unsafe {
            TemplateFixedMin::call(
                &IndexTemplateF { query: &query },
                &mut [MaybeUninit::uninit(); 2048],
            )
        }
        .unwrap(),
        expected.as_bytes()
    );

    assert_eq!(
        TemplateBytesMin::ccall::<BytesMut>(
            IndexTemplateB {
                query: query
                    .get("name")
                    .and_then(|name| query.get("lastname").map(|lastname| (*name, *lastname))),
            },
            2048,
        ),
        Bytes::from(expected)
    )
}
