#![allow(clippy::into_iter_on_ref)]

use yarte::Template;

#[derive(Template)]
#[template(path = "simple-iterator.hbs")]
struct SimpleIterator<'a> {
    title: &'a str,
    story: Story<'a>,
    comments: Vec<Comment<'a>>,
}

struct Story<'a> {
    intro: &'a str,
    body: &'a str,
}

struct Comment<'a> {
    subject: &'a str,
    body: &'a str,
}

#[test]
fn test_simple_iterator() {
    let t = SimpleIterator {
        title: "foo",
        story: Story {
            intro: "bar",
            body: "baz",
        },
        comments: vec![
            Comment {
                subject: "foo",
                body: "bar",
            },
            Comment {
                subject: "baz",
                body: "FOO",
            },
        ],
    };

    assert_eq!("entry\nfoo\n  \n   foo\n    bar\n  bar\n    baz\n  \nbaz\n    foofoo\n      bar\n    foobaz\n      FOO", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "with-simple-iterator.hbs")]

struct WhitSimpleIterator<'a> {
    title: &'a str,
    story: History<'a>,
}

struct History<'a> {
    intro: &'a str,
    body: &'a str,
}

#[test]
fn test_whit_simple_iterator() {
    let t = WhitSimpleIterator {
        title: "foo",
        story: History {
            intro: "bar",
            body: "baz",
        },
    };

    assert_eq!(
        "entryfoo\n\n    intro\n    bar\n    body\n    baz\n",
        t.call().unwrap()
    );
}
