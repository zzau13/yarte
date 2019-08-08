use yarte::Template;

#[derive(Template)]
#[template(path = "for.hbs")]
struct ForTemplate<'a> {
    strings: Vec<&'a str>,
}

#[test]
fn test_for() {
    let s = ForTemplate {
        strings: vec!["foo", "bar", "baz"],
    };
    assert_eq!(s.call().unwrap(), "0. foo(first)1. bar2. baz");
}

#[derive(Template)]
#[template(path = "nested-for.hbs")]
struct NestedForTemplate<'a> {
    seqs: &'a [&'a [&'a str]],
}

#[test]
fn test_nested_for() {
    let alpha: &[&str] = &["foo", "bar", "baz"];
    let numbers: &[&str] = &["bar", "baz"];
    let seqs: &[&[&str]] = &[alpha, numbers];
    let s = NestedForTemplate { seqs };
    assert_eq!(s.call().unwrap(), "1\n  0foo1bar2baz2\n  0bar1baz");
}

#[derive(Template)]
#[template(path = "precedence-for.hbs")]
struct PrecedenceTemplate<'a> {
    strings: &'a [&'a str],
}

#[test]
fn test_precedence_for() {
    let strings: &[&str] = &["foo", "bar", "baz"];
    let s = PrecedenceTemplate { strings };
    assert_eq!(s.call().unwrap(), "0 ~ foo2bar1 ~ bar42 ~ baz6")
}

#[derive(Template)]
#[template(path = "for-range.hbs")]
struct ForRangeTemplate {
    init: i32,
    end: i32,
}

#[test]
fn test_for_range() {
    let s = ForRangeTemplate { init: -1, end: 1 };
    assert_eq!(s.call().unwrap(), "foo\nfoo\nbar\nbar\nfoo\nbar\nbar\n");
}
