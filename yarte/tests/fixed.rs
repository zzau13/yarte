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
    let s = EscapeTemplate { name: "<>&\"" };
    let mut buf: [u8; 64] = unsafe { MaybeUninit::uninit().assume_init() };
    let b = unsafe { s.call(&mut buf) }.unwrap();
    assert_eq!(&buf[..b], b"Hello, &lt;&gt;&amp;&quot;!");
}

#[derive(TemplateFixed)]
#[template(path = "for")]
struct ForTemplate<'a> {
    strings: Vec<&'a str>,
}

#[test]
fn test_for() {
    let s = ForTemplate {
        strings: vec!["foo", "bar", "baz"],
    };
    let mut buf: [u8; 64] = unsafe { MaybeUninit::uninit().assume_init() };
    let b = unsafe { s.call(&mut buf) }.unwrap();
    assert_eq!(&buf[..b], b"0. foo(first)1. bar2. baz");
}

#[derive(TemplateFixed)]
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
    let mut buf: [u8; 64] = unsafe { MaybeUninit::uninit().assume_init() };
    let b = unsafe { s.call(&mut buf) }.unwrap();
    assert_eq!(&buf[..b], b"1\n  0foo1bar2baz2\n  0bar1baz");
}

#[derive(TemplateFixed)]
#[template(path = "precedence-for")]
struct PrecedenceTemplate<'a> {
    strings: &'a [&'a str],
}

#[test]
fn test_precedence_for() {
    let strings: &[&str] = &["foo", "bar", "baz"];
    let s = PrecedenceTemplate { strings };
    let mut buf: [u8; 64] = unsafe { MaybeUninit::uninit().assume_init() };
    let b = unsafe { s.call(&mut buf) }.unwrap();
    assert_eq!(&buf[..b], b"0 ~ foo2bar1 ~ bar42 ~ baz6")
}

#[derive(TemplateFixed)]
#[template(path = "for-range")]
struct ForRangeTemplate {
    init: i32,
    end: i32,
}

#[test]
fn test_for_range() {
    let s = ForRangeTemplate { init: -1, end: 1 };
    let mut buf: [u8; 64] = unsafe { MaybeUninit::uninit().assume_init() };
    let b = unsafe { s.call(&mut buf) }.unwrap();
    assert_eq!(&buf[..b], b"foo\nfoo\nbar\nbar\nfoo\nbar\nbar\n");
}

const OUT_L3: usize = 50 * 1024;
#[derive(TemplateFixed)]
#[template(src = "{{# each 0..OUT_L3 }}a{{ super::f }}{{ !super::f }}{{/ each}} ")]
struct UnAlignedBool {
    f: bool,
}

#[test]
fn test_unaligned() {
    let s = UnAlignedBool { f: true };
    let mut buf: [u8; OUT_L3 * 10] = unsafe { MaybeUninit::uninit().assume_init() };
    let b = unsafe { s.call(&mut buf) }.unwrap();
    assert_eq!(buf[..b].to_vec(), "atruefalse".repeat(OUT_L3).into_bytes());
}
