#![allow(clippy::into_iter_on_ref)]
use yarte::Template;

pub struct Fortune {
    id: i32,
    message: String,
}

#[derive(Template)]
#[template(path = "html/fortune.hbs")]
pub struct FortunesTemplate {
    fortunes: Vec<Fortune>,
}

#[test]
fn test_fortune() {
    let t = FortunesTemplate {
        fortunes: vec![
            Fortune {
                id: 0,
                message: "foo".to_string(),
            },
            Fortune {
                id: 1,
                message: "bar".to_string(),
            },
        ],
    };
    assert_eq!(
        "<!DOCTYPE html><html><head><title>Fortunes</title></head><body><table><tr><th>id</\
         th><th>message</th></tr><tr><td>0</td><td>foo</td></tr><tr><td>1</td><td>bar</td></tr></\
         table></body></html>",
        t.call().unwrap()
    );
}
