#![allow(clippy::into_iter_on_ref)]
use yarte::Template;
use yarte_config::{read_config_file, Config};

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

#[derive(Template)]
#[template(path = "html/raw/index.html")]
struct RawIndexTemplate;

#[test]
fn test_raw_index() {
    let config = read_config_file();
    let config = Config::new(&config);
    let (_, expected) = config.get_template("html/raw/index-expected.html");
    let t = RawIndexTemplate.call().unwrap();

    assert_eq!(t, expected);
}

#[allow(dead_code)]
enum Mode {
    Text,
    Embedded,
    Else,
}

struct Item {
    href: String,
    name: String,
}

struct Header {
    role: String,
    title: String,
    description: String,
}

#[derive(Template)]
#[template(path = "html/header.hbs")]
struct HeaderTemplate {
    mode: Mode,
    header: Header,
    item: Vec<Item>,
}

#[test]
fn test_header() {
    let t = HeaderTemplate {
        mode: Mode::Text,
        header: Header {
            role: "banner".to_string(),
            title: "foo".to_string(),
            description: "bar".to_string(),
        },
        item: vec![Item {
            href: "bar".to_string(),
            name: "Bar".to_string(),
        }],
    };

    // TODO: when minimize
    t.call().unwrap();
}
