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

    assert_eq!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" \
         content=\"width=device-width, initial-scale=1.0\"><title>HTML5 Test \
         Page</title></head><body><div id=\"top\" class=\"page\" role=\"document\"><header \
         role=\"banner\"><h1>foo</h1><p>bar</p></header><nav role=\"navigation\"><ul><li><a \
         href=\"#text\">Text</a> <ul><li><a \
         href=\"#bar\">Bar</a></li></ul></li></ul></nav></div></body></html>",
        t.call().unwrap()
    );
}

#[derive(Template)]
#[template(path = "html/article.hbs")]
struct ArticleTemplate {
    flag: bool,
}

#[test]
fn test_article() {
    let t = ArticleTemplate { flag: true };

    assert_eq!(
        "<article id=\"text__code\"><header><h1>Code</h1></header><div><p><strong>Keyboard \
         input:</strong> <kbd>Cmd</kbd></p><p><strong>Inline code:</strong> \
         <code>&lt;div&gt;code&lt;/div&gt;</code></p><p><strong>Sample output:</strong> \
         <samp>This is sample output from a computer program.</samp></p><h2>Pre-formatted \
         text</h2></div><footer><p><a href=\"#top\">[Top]</a></p></footer></article>",
        t.call().unwrap()
    );
}

#[derive(Template)]
#[template(path = "html/pre.hbs")]
struct PreTemplate {
    flag: bool,
}

#[test]
fn test_pre() {
    let t = PreTemplate { flag: false };
    assert_eq!(
        "<pre>P R E F O R M A T T E D T E X T\n  ! \" # $ % &amp; \' ( ) * + , - . /\n  0 1 2 3 4 \
         5 6 7 8 9 : ; &lt; = &gt; ?\n  @ A B C D E F G H I J K L M N O\n  P Q R S T U V W X Y Z \
         [ \\ ] ^ _\n  ` a b c d e f g h i j k l m n o\n  p q r s t u v w x y z { | } ~ </pre>",
        t.call().unwrap()
    )
}

#[derive(Template)]
#[template(path = "html/section.hbs")]
struct SectionTemplate {
    flag: bool,
}

#[test]
fn test_section() {
    let t = SectionTemplate { flag: false };
    let expected = "<section id=\"embedded\"><header><h1>Embedded content</h1></header>\
        <article id=\"embedded__images\"><header><h2>Images</h2></header><div>\
        <h3>No <code>&lt;figure&gt;</code> element</h3><p>\
        <img src=\"http://placekitten.com/480/480\" alt=\"Image alt text\"></p>\
        <h3>Wrapped in a <code>&lt;figure&gt;</code> element, no <code>&lt;figcaption&gt;</code>\
        </h3><figure><img src=\"http://placekitten.com/420/420\" alt=\"Image alt text\"></figure>\
        <h3>Wrapped in a <code>&lt;figure&gt;</code> element, with a <code>&lt;figcaption&gt;\
        </code></h3><figure><img src=\"http://placekitten.com/420/420\" alt=\"Image alt text\">\
        <figcaption>Here is a caption for this image.</figcaption></figure></div><footer><p>\
        <a href=\"#top\">[Top]</a></p></footer></article><article id=\"embedded__audio\"><header>\
        <h2>Audio</h2></header><div><audio controls>audio</audio></div><footer><p><a href=\"#top\">\
        [Top]</a></p></footer></article><article id=\"embedded__video\"><header><h2>Video</h2>\
        </header><div><video controls>video</video></div><footer><p><a href=\"#top\">[Top]</a>\
        </p></footer></article><article id=\"embedded__canvas\"><header><h2>Canvas</h2></header>\
        <div><canvas>canvas</canvas></div><footer><p><a href=\"#top\">[Top]</a></p></footer>\
        </article><article id=\"embedded__meter\"><header><h2>Meter</h2></header><div>\
        <meter value=\"2\" min=\"0\" max=\"10\">2 out of 10</meter></div><footer><p>\
        <a href=\"#top\">[Top]</a></p></footer></article><article id=\"embedded__progress\">\
        <header><h2>Progress</h2></header><div><progress>progress</progress></div><footer><p>\
        <a href=\"#top\">[Top]</a></p></footer></article><article id=\"embedded__svg\"><header>\
        <h2>Inline SVG</h2></header><div><svg width=\"100px\" height=\"100px\">\
        <circle cx=\"100\" cy=\"100\" r=\"100\" fill=\"#1fa3ec\"></circle></svg></div><footer>\
        <p><a href=\"#top\">[Top]</a></p></footer></article><article id=\"embedded__iframe\">\
        <header><h2>IFrame</h2></header><div><iframe src=\"index.html\" height=\"300\"></iframe>\
        </div><footer><p><a href=\"#top\">[Top]</a></p></footer></article></section>";

    assert_eq!(expected, t.call().unwrap())
}
