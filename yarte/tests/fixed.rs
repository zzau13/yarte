#![cfg(feature = "fixed")]
#![allow(clippy::uninit_assumed_init)]

use std::mem::MaybeUninit;

use yarte::TemplateFixed;

#[derive(TemplateFixed)]
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
    let mut buf: [u8; 128] = unsafe { MaybeUninit::uninit().assume_init() };
    let b = unsafe { s.call(&mut buf) }.unwrap();
    assert_eq!(
        &buf[..b],
        "hello world, foo\nwith number: 42\nIñtërnâtiônàlizætiøn is important\nin vars too: \
         Iñtërnâtiônàlizætiøn"
            .as_bytes()
    );
}

#[derive(TemplateFixed)]
#[template(path = "hello")]
struct EscapeTemplate<'a> {
    name: &'a str,
}

#[test]
fn test_escape() {
    let s = EscapeTemplate { name: "<>&\"'/" };
    let mut buf: [u8; 64] = unsafe { MaybeUninit::uninit().assume_init() };
    let b = unsafe { s.call(&mut buf) }.unwrap();
    assert_eq!(
        &buf[..b],
        "Hello, &lt;&gt;&amp;&quot;&#x27;&#x2f;!".as_bytes()
    );
}
