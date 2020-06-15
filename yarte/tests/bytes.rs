#![cfg(feature = "byts_buff")]
#![allow(clippy::uninit_assumed_init)]

use bytes::Bytes;

use yarte::TemplateBytes;

trait ToBytes {
    fn byteb(self) -> Bytes;
}

impl ToBytes for &'static str {
    fn byteb(self) -> Bytes {
        Bytes::from(self)
    }
}

#[derive(TemplateBytes)]
#[template(path = "simple")]
struct VariablesTemplate<'a> {
    strvar: &'a str,
    num: i64,
    i18n: String,
}

#[test]
fn test_variables() {
    let s = VariablesTemplate {
        strvar: "foo",
        num: 42,
        i18n: "Iñtërnâtiônàlizætiøn".to_string(),
    };
    assert_eq!(
        s.call(128).unwrap(),
        "hello world, foo\nwith number: 42\nIñtërnâtiônàlizætiøn is important\nin vars too: \
         Iñtërnâtiônàlizætiøn"
            .byteb()
    );
}

#[derive(TemplateBytes)]
#[template(path = "hello")]
struct EscapeTemplate<'a> {
    name: &'a str,
}

#[test]
fn test_escape() {
    let s = EscapeTemplate { name: "<>&\"" };
    assert_eq!(s.call(64).unwrap(), "Hello, &lt;&gt;&amp;&quot;!".byteb());
}

#[derive(TemplateBytes)]
#[template(path = "for")]
struct ForTemplate<'a> {
    strings: Vec<&'a str>,
}

#[test]
fn test_for() {
    let s = ForTemplate {
        strings: vec!["foo", "bar", "baz"],
    };
    assert_eq!(s.call(64).unwrap(), ". foo(first)1. bar2. baz".byteb());
}

#[derive(TemplateBytes)]
#[template(path = "nested-for")]
struct NestedForTemplate<'a> {
    seqs: &'a [&'a [&'a str]],
}

#[test]
fn test_nested_for() {
    let alpha: &[&str] = &["foo", "bar", "baz"];
    let numbers: &[&str] = &["bar", "baz"];
    let seqs: &[&[&str]] = &[alpha, numbers];
    let s = NestedForTemplate { seqs };
    assert_eq!(
        s.call(64).unwrap(),
        "1\n  0foo1bar2baz2\n  0bar1baz".byteb()
    );
}
