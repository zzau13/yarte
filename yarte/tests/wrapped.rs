#![cfg(feature = "fixed")]
#![allow(clippy::uninit_assumed_init)]

use std::mem::MaybeUninit;

use yarte::{Template, TemplateFixed};

#[derive(Template, TemplateFixed)]
#[template(path = "wrapped-if")]
struct IfTemplate {
    cond: bool,
}

#[test]
fn test_if() {
    let t = IfTemplate { cond: true };
    assert_eq!("&amp;", Template::call(&t).unwrap());
    let mut buf: [u8; 64] = unsafe { MaybeUninit::uninit().assume_init() };
    let b = unsafe { TemplateFixed::call(&t, &mut buf) }.unwrap();
    assert_eq!(b"&amp;", &buf[..b]);
}

#[derive(Template, TemplateFixed)]
#[template(path = "wrapped-index")]
struct IndexTemplate<'a> {
    arr: Vec<&'a str>,
}

#[test]
fn test_index() {
    let t = IndexTemplate { arr: vec!["&"] };
    assert_eq!("&amp;", Template::call(&t).unwrap());
    let mut buf: [u8; 64] = unsafe { MaybeUninit::uninit().assume_init() };
    let b = unsafe { TemplateFixed::call(&t, &mut buf) }.unwrap();
    assert_eq!(b"&amp;", &buf[..b]);
}

#[derive(Template, TemplateFixed)]
#[template(path = "wrapped-slice")]
struct SliceTemplate<'a> {
    arr: &'a [&'a str],
}

#[test]
fn test_slice() {
    let arr: &[&str] = &["&"];
    let t = SliceTemplate { arr };
    assert_eq!("&amp;", Template::call(&t).unwrap());
    let mut buf: [u8; 64] = unsafe { MaybeUninit::uninit().assume_init() };
    let b = unsafe { TemplateFixed::call(&t, &mut buf) }.unwrap();
    assert_eq!(b"&amp;", &buf[..b]);
}

fn repeat(s: &str, i: usize) -> String {
    s.repeat(i)
}

#[derive(Template, TemplateFixed)]
#[template(path = "wrapped-call")]
struct CallTemplate<'a> {
    s: &'a str,
}

#[test]
fn test_call() {
    let t = CallTemplate { s: "&" };
    assert_eq!("&amp;&amp;", Template::call(&t).unwrap());
    let mut buf: [u8; 64] = unsafe { MaybeUninit::uninit().assume_init() };
    let b = unsafe { TemplateFixed::call(&t, &mut buf) }.unwrap();
    assert_eq!(b"&amp;&amp;", &buf[..b]);
}

#[derive(Template, TemplateFixed)]
#[template(path = "wrapped-array")]
struct ArrayTemplate;

#[test]
fn test_array() {
    let t = ArrayTemplate;
    assert_eq!("&amp;", Template::call(&t).unwrap());
    let mut buf: [u8; 64] = unsafe { MaybeUninit::uninit().assume_init() };
    let b = unsafe { TemplateFixed::call(&t, &mut buf) }.unwrap();
    assert_eq!(b"&amp;", &buf[..b]);
}

#[derive(Template, TemplateFixed)]
#[template(path = "wrapped-tuple")]
struct TupleTemplate;

#[test]
fn test_tuple() {
    let t = TupleTemplate;
    assert_eq!("&amp;", Template::call(&t).unwrap());
    let mut buf: [u8; 64] = unsafe { MaybeUninit::uninit().assume_init() };
    let b = unsafe { TemplateFixed::call(&t, &mut buf) }.unwrap();
    assert_eq!(b"&amp;", &buf[..b]);
}

struct Debuggable<T>(T)
where
    T: std::fmt::Debug;

impl<T> std::fmt::Display for Debuggable<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

#[derive(Template)]
#[template(src = "{{{ debug }}}")]
struct DebugTemplate {
    debug: Debuggable<Vec<usize>>,
}

#[test]
fn test_debug() {
    let t = DebugTemplate {
        debug: Debuggable(vec![0]),
    };
    assert_eq!("[0]", t.call().unwrap());
}
