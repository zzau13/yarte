#![cfg(any(feature = "bytes-buf", feature = "bytes-buf-tokio2"))]
#![allow(clippy::uninit_assumed_init)]

use yarte::TemplateBytes;

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
        s.call::<String>(128),
        "hello world, foo\nwith number: 42\nIñtërnâtiônàlizætiøn is important\nin vars too: \
         Iñtërnâtiônàlizætiøn"
    );
    let mut b = String::with_capacity(128);
    s.write_call(&mut b);
    assert_eq!(
        b,
        "hello world, foo\nwith number: 42\nIñtërnâtiônàlizætiøn is important\nin vars too: \
         Iñtërnâtiônàlizætiøn"
    )
}

const LOOP: usize = 13;

#[derive(TemplateBytes)]
#[template(src = "{{# each 0..LOOP }}12{{ this }}1{{ this }}123{{/each }}987")]
struct Loop;

#[test]
fn bytes3() {
    assert_eq!(
        Loop.call::<String>(8),
        "12010123121111231221212312313123124141231251512312616123127171231281812312919123121011012312111111231212112123987"
    );

    let mut b = String::with_capacity(8);
    Loop.write_call(&mut b);
    assert_eq!(
        b,
        "12010123121111231221212312313123124141231251512312616123127171231281812312919123121011012312111111231212112123987"
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
    assert_eq!(s.call::<String>(64), "Hello, &lt;&gt;&amp;&quot;!");
    let mut b = String::with_capacity(64);
    s.write_call(&mut b);
    assert_eq!(b, "Hello, &lt;&gt;&amp;&quot;!")
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
    assert_eq!(s.call::<String>(64), "0. foo(first)1. bar2. baz");

    let mut b = String::with_capacity(64);
    s.write_call(&mut b);
    assert_eq!(b, "0. foo(first)1. bar2. baz")
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
    assert_eq!(s.call::<String>(64), "1\n  0foo1bar2baz2\n  0bar1baz");

    let mut b = String::with_capacity(64);
    s.write_call(&mut b);
    assert_eq!(b, "1\n  0foo1bar2baz2\n  0bar1baz")
}
