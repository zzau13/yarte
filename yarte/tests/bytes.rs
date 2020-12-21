#![cfg(any(feature = "bytes-buf", feature = "bytes-buf-tokio3"))]
#![allow(clippy::uninit_assumed_init)]

use yarte::{Bytes, BytesMut, TemplateBytes};

trait ToBytes {
    fn byteb(self) -> Bytes;
}

impl ToBytes for &'static str {
    fn byteb(self) -> Bytes {
        Bytes::from(self)
    }
}

#[derive(TemplateBytes)]
#[template(path = "simple", print = "code")]
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
        s.call::<BytesMut>(128),
        "hello world, foo\nwith number: 42\nIñtërnâtiônàlizætiøn is important\nin vars too: \
         Iñtërnâtiônàlizætiøn"
            .byteb()
    );
    let mut b = BytesMut::with_capacity(128);
    s.write_call(&mut b);
    assert_eq!(
        b.freeze(),
        "hello world, foo\nwith number: 42\nIñtërnâtiônàlizætiøn is important\nin vars too: \
         Iñtërnâtiônàlizætiøn"
            .byteb()
    )
}

const LOOP: usize = 13;

#[derive(TemplateBytes)]
#[template(src = "{{# each 0..LOOP }}12{{ this }}1{{ this }}123{{/each }}987")]
struct Loop;

#[test]
fn bytes3() {
    assert_eq!(
        Loop.call::<BytesMut>(8),
        "12010123121111231221212312313123124141231251512312616123127171231281812312919123121011012312111111231212112123987"
            .byteb()
    );

    let mut b = BytesMut::with_capacity(8);
    Loop.write_call(&mut b);
    assert_eq!(
        b.freeze(),
        "12010123121111231221212312313123124141231251512312616123127171231281812312919123121011012312111111231212112123987"
            .byteb()
    )
}

#[derive(TemplateBytes)]
#[template(path = "hello")]
struct EscapeTemplate<'a> {
    name: &'a str,
}

#[test]
fn test_escape() {
    let s = EscapeTemplate { name: "<>&\"" };
    assert_eq!(
        s.call::<BytesMut>(64),
        "Hello, &lt;&gt;&amp;&quot;!".byteb()
    );
    let mut b = BytesMut::with_capacity(64);
    s.write_call(&mut b);
    assert_eq!(b.freeze(), "Hello, &lt;&gt;&amp;&quot;!".byteb())
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
    assert_eq!(s.call::<BytesMut>(64), "0. foo(first)1. bar2. baz".byteb());

    let mut b = BytesMut::with_capacity(64);
    s.write_call(&mut b);
    assert_eq!(b.freeze(), "0. foo(first)1. bar2. baz".byteb())
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
        s.call::<BytesMut>(64),
        "1\n  0foo1bar2baz2\n  0bar1baz".byteb()
    );

    let mut b = BytesMut::with_capacity(64);
    s.write_call(&mut b);
    assert_eq!(b.freeze(), "1\n  0foo1bar2baz2\n  0bar1baz".byteb())
}
