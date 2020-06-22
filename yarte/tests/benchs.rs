#![cfg(all(feature = "html-min", feature = "fixed", feature = "bytes-buf"))]
#![allow(clippy::into_iter_on_ref)]
#![allow(clippy::uninit_assumed_init)]

use std::mem::MaybeUninit;

use bytes::Bytes;

use yarte::{
    TemplateBytesMin as TemplateBytes, TemplateFixedMin as TemplateFixed, TemplateMin as Template,
};

#[test]
fn big_table() {
    let size = 3;
    let mut table = Vec::with_capacity(size);
    for _ in 0..size {
        let mut inner = Vec::with_capacity(size);
        for i in 0..size {
            inner.push(i);
        }
        table.push(inner);
    }

    let table = BigTable { table };
    let expected =
        "<table><tr><td>0</td><td>1</td><td>2</td></tr><tr><td>0</td><td>1</td><td>2</td></\
         tr><tr><td>0</td><td>1</td><td>2</td></tr></table>";
    assert_eq!(Template::call(&table).unwrap(), expected);
    assert_eq!(
        unsafe { TemplateFixed::call(&table, &mut [MaybeUninit::uninit(); 256]) }.unwrap(),
        expected.as_bytes()
    );
    assert_eq!(TemplateBytes::ccall(table, 0), Bytes::from(expected))
}

#[derive(Template, TemplateFixed, TemplateBytes)]
#[template(path = "big-table")]
struct BigTable {
    table: Vec<Vec<usize>>,
}

#[test]
fn teams() {
    let teams = Teams {
        year: 2015,
        teams: vec![
            Team {
                name: "Jiangsu".into(),

                score: 43,
            },
            Team {
                name: "Beijing".into(),
                score: 27,
            },
            Team {
                name: "Guangzhou".into(),
                score: 22,
            },
            Team {
                name: "Shandong".into(),
                score: 12,
            },
        ],
    };
    let expected = "<html><head><title>2015</title></head><body><h1>CSL 2015</h1><ul><li \
         class=\"champion\"><b>Jiangsu</b>: 43</li><li class=\"\"><b>Beijing</b>: 27</li><li \
         class=\"\"><b>Guangzhou</b>: 22</li><li class=\"\"><b>Shandong</b>: \
         12</li></ul></body></html>";
    assert_eq!(Template::call(&teams).unwrap(), expected);
    assert_eq!(
        unsafe { TemplateFixed::call(&teams, &mut [MaybeUninit::uninit(); 256]) }.unwrap(),
        expected.as_bytes()
    );
    assert_eq!(TemplateBytes::ccall(teams, 0), Bytes::from(expected))
}

#[derive(Template, TemplateFixed, TemplateBytes)]
#[template(path = "teams")]
struct Teams {
    year: u16,
    teams: Vec<Team>,
}

struct Team {
    name: String,
    score: u8,
}
