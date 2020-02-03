#![allow(clippy::into_iter_on_ref)]

use yarte::Template;

#[derive(Template)]
#[template(path = "with-partial.hbs")]
struct PartialTemplate<'a> {
    strs: &'a [&'a str],
}

#[test]
fn test_partial() {
    let strs = vec!["foo", "bar"];
    let s = PartialTemplate { strs: &strs };
    assert_eq!(s.call().unwrap(), "foo\nIn partial\n1bar\nIn partial\n2")
}

#[derive(Template)]
#[template(path = "with-partial-indir.hbs")]
struct PartialDirTemplate<'a> {
    strs: &'a [&'a str],
}

#[test]
fn test_partial_dir() {
    let strs = vec!["foo", "bar"];
    let s = PartialDirTemplate { strs: &strs };
    assert_eq!(
        s.call().unwrap(),
        "foo\npartial in subdirectory\n1bar\npartial in subdirectory\n2"
    )
}

#[derive(Template)]
#[template(path = "deep/with-partial.hbs")]
struct PartialDirDTemplate<'a> {
    strs: &'a [&'a str],
}

#[test]
fn test_partial_dir_d() {
    let strs = vec!["foo", "bar"];
    let s = PartialDirDTemplate { strs: &strs };
    assert_eq!(
        s.call().unwrap(),
        "foo\npartial in subdirectory\n1bar\npartial in subdirectory\n2"
    )
}

struct Scope<'a> {
    this: &'a str,
    index: usize,
}

#[derive(Template)]
#[template(path = "with-partial-scope.hbs")]
struct PartialScopeTemplate<'a> {
    scope: Scope<'a>,
}

#[test]
fn test_partial_scope() {
    let s = PartialScopeTemplate {
        scope: Scope {
            this: "foo",
            index: 0,
        },
    };
    assert_eq!(s.call().unwrap(), "foo\nIn partial\n0")
}

#[derive(Template)]
#[template(path = "with-partial-lit.hbs")]
struct PartialLitTemplate;

#[test]
fn test_partial_lit() {
    assert_eq!(PartialLitTemplate.call().unwrap(), "&amp;\nIn partial\n1")
}

#[derive(Template)]
#[template(path = "with-partial-lit-mix.hbs")]
struct PartialLitMixTemplate {
    index: usize,
}

#[test]
fn test_partial_lit_mix() {
    let t = PartialLitMixTemplate { index: 0 };
    assert_eq!(t.call().unwrap(), "&amp;\nIn partial\n0")
}

#[derive(Template)]
#[template(path = "with-partial-lit-scp.hbs")]
struct PartialLitScopeTemplate<'a> {
    scope: Scope<'a>,
}

#[test]
fn test_partial_lit_scope() {
    let t = PartialLitScopeTemplate {
        scope: Scope {
            this: "foo",
            index: 1,
        },
    };
    assert_eq!(t.call().unwrap(), "foo\nIn partial\n0")
}

#[derive(Template)]
#[template(path = "with-partial-compose.hbs")]
struct PartialComposeTemplate<'a> {
    scope: Scope<'a>,
}

#[test]
fn test_partial_compose() {
    let t = PartialComposeTemplate {
        scope: Scope {
            this: "foo",
            index: 1,
        },
    };
    assert_eq!(t.call().unwrap(), "&amp;\nIn partial\n0foo\nIn partial\n0")
}

#[derive(Template)]
#[template(path = "with-partial-self.hbs")]
struct PartialSelfTemplate<'a> {
    scope: Scope<'a>,
    this: &'a str,
    index: usize,
}

#[test]
fn test_partial_self() {
    let t = PartialSelfTemplate {
        scope: Scope {
            this: "foo",
            index: 1,
        },
        this: "bar",
        index: 2,
    };
    assert_eq!(t.call().unwrap(), "1foo1foo1foo\n2bar2bar1foo");
}

#[derive(Template)]
#[template(path = "with-partial-each.hbs")]
struct PartialEachTemplate<'a> {
    list: &'a [&'a str],
}

#[test]
fn test_partial_each() {
    let t = PartialEachTemplate {
        list: &["&", "foo"],
    };
    assert_eq!(t.call().unwrap(), "&amp;\nIn partial\n0foo\nIn partial\n1")
}

#[derive(Template)]
#[template(path = "with-partial-on.hbs")]
struct PartialOnTemplate<'a> {
    list: &'a [Scope<'a>],
    this: &'a str,
    index: usize,
}

#[test]
fn test_partial_on() {
    let t = PartialOnTemplate {
        list: &[
            Scope {
                this: "foo",
                index: 4,
            },
            Scope {
                this: "&",
                index: 5,
            },
        ],
        this: "bar",
        index: 6,
    };

    assert_eq!(
        t.call().unwrap(),
        "4foo4foo4foo5&amp;5&amp;5&amp;\n6bar0h4foo6bar1h5&amp;"
    )
}

#[derive(Template)]
#[template(path = "partial-compile.hbs")]
struct PartialCompile {
    is_bar: bool,
    num: usize,
}

#[test]
fn test_partial_compile() {
    let t = PartialCompile {
        is_bar: true,
        num: 1,
    };

    assert_eq!(t.call().unwrap(), "foo");

    let t = PartialCompile {
        is_bar: false,
        num: 1,
    };

    assert_eq!(t.call().unwrap(), "");

    let t = PartialCompile {
        is_bar: true,
        num: 0,
    };

    assert_eq!(t.call().unwrap(), "");
}

#[derive(Template)]
#[template(path = "with-partial-compile.hbs", print = "code")]
struct WithPartialCompile;

#[test]
fn test_with_partial_compile() {
    let t = WithPartialCompile;

    assert_eq!(t.call().unwrap(), "foo");
}

#[derive(Template)]
#[template(path = "partial-compile-cond.hbs")]
struct PartialCompileCond {
    is_bar: bool,
    num: usize,
}

#[test]
fn test_partial_compile_cond() {
    let t = PartialCompileCond {
        is_bar: true,
        num: 1,
    };

    assert_eq!(t.call().unwrap(), "foofoo");

    let t = PartialCompileCond {
        is_bar: false,
        num: 1,
    };

    assert_eq!(t.call().unwrap(), "foobarfalse");

    let t = PartialCompileCond {
        is_bar: true,
        num: 0,
    };

    assert_eq!(t.call().unwrap(), "foobartruebar");
}

#[derive(Template)]
#[template(path = "with-partial-compile-cond.hbs", print = "code")]
struct WithPartialCompileCond;

#[test]
fn test_with_partial_compile_cond() {
    let t = WithPartialCompileCond;

    assert_eq!(t.call().unwrap(), "foofoofoobarfalsefoobartruebar");
}

#[derive(Template)]
#[template(src = "{{#> partial-block }}Foo{{/partial-block }}")]
struct PartialBlock;

#[test]
fn test_partial_block() {
    let t = PartialBlock;

    assert_eq!(t.call().unwrap(), "BarFooFol");
}

#[derive(Template)]
#[template(src = "{{#> with-partial-block }}Foo{{/with-partial-block }}")]
struct WithPartialBlock;

#[test]
fn test_with_partial_block() {
    let t = WithPartialBlock;

    assert_eq!(t.call().unwrap(), "fooBarBalFooForFolbar");
}

#[derive(Template)]
#[template(src = "{{#> partial-block-ctx a = \"bar\" }}Fol{{ a }}{{/partial-block-ctx }}")]
struct PartialBlockCtx {
    a: usize,
}

#[test]
fn test_partial_block_ctx() {
    let t = PartialBlockCtx { a: 0 };

    assert_eq!(t.call().unwrap(), "FoobarBarFol0");
}

#[derive(Template)]
#[template(src = "{{#> partial-block-ws }}
 Foo {{/partial-block-ws }}")]
struct PartialBlockWs;

#[test]
fn test_partial_block_ws() {
    let t = PartialBlockWs;

    assert_eq!(t.call().unwrap(), "foo   \n Foo \nbar");
}

#[derive(Template)]
#[template(src = "{{#> partial-block-ws ~}}
 Foo {{/partial-block-ws }}")]
struct PartialBlockWs1;

#[test]
fn test_partial_block_ws_1() {
    let t = PartialBlockWs1;

    assert_eq!(t.call().unwrap(), "foo   Foo \nbar");
}

#[derive(Template)]
#[template(src = "{{#> partial-block-ws }}
 Foo {{~/partial-block-ws }}")]
struct PartialBlockWs2;

#[test]
fn test_partial_block_ws_2() {
    let t = PartialBlockWs2;

    assert_eq!(t.call().unwrap(), "foo   \n Foo\nbar");
}

#[derive(Template)]
#[template(src = "{{#> partial-block }}
 Foo {{/partial-block }}")]
struct PartialBlockWs3;

#[test]
fn test_partial_block_ws_3() {
    let t = PartialBlockWs3;

    assert_eq!(t.call().unwrap(), "Bar\n Foo Fol");
}

#[derive(Template)]
#[template(src = "{{#> partial-block }}
 Foo {{~/partial-block }}")]
struct PartialBlockWs4;

#[test]
fn test_partial_block_ws_4() {
    let t = PartialBlockWs4;

    assert_eq!(t.call().unwrap(), "Bar\n FooFol");
}
