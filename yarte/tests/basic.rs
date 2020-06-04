#![allow(clippy::blacklisted_name)]

use std::collections::HashMap;

use yarte::{Template, TemplateText};

#[derive(Template)]
#[template(path = "hello")]
struct HelloTemplate<'a> {
    name: &'a str,
}

#[test]
fn test_hello() {
    let hello = HelloTemplate { name: "world" };
    assert_eq!("Hello, world!", hello.call().unwrap());
}

#[derive(TemplateText)]
#[template(src = "{{}")]
struct BracketsTemplate;

#[test]
fn test_brackets() {
    let hello = BracketsTemplate;
    assert_eq!("{{}", hello.call().unwrap());
}

#[derive(TemplateText)]
#[template(src = "{{{}")]
struct Brackets2Template;

#[test]
fn test_brackets2() {
    let hello = Brackets2Template;
    assert_eq!("{{{}", hello.call().unwrap());
}

#[derive(Template)]
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
        s.call().unwrap(),
        "hello world, foo\nwith number: 42\nIñtërnâtiônàlizætiøn is important\nin vars too: \
         Iñtërnâtiônàlizætiøn"
    );
}

#[derive(Template)]
#[template(path = "hello")]
struct EscapeTemplate<'a> {
    name: &'a str,
}

#[test]
fn test_escape() {
    let s = EscapeTemplate { name: "<>&\"'/" };

    assert_eq!(s.call().unwrap(), "Hello, &lt;&gt;&amp;&quot;&#x27;&#x2f;!");
}

#[derive(TemplateText)]
#[template(path = "simple-no-escape")]
struct VariablesTemplateNoEscape<'a> {
    strvar: &'a str,
    num: i64,
    i18n: String,
}

#[test]
fn test_variables_no_escape() {
    let s = VariablesTemplateNoEscape {
        strvar: "foo",
        num: 42,
        i18n: "Iñtërnâtiônàlizætiøn".to_string(),
    };
    assert_eq!(
        s.call().unwrap(),
        "hello world, foo\nwith number: 42\nIñtërnâtiônàlizætiøn is important\nin vars too: \
         Iñtërnâtiônàlizætiøn"
    );
}

#[derive(Template)]
#[template(path = "if")]
struct IfTemplate {
    cond: bool,
}

#[test]
fn test_if() {
    let s = IfTemplate { cond: true };
    assert_eq!(s.call().unwrap(), "true");
}

#[derive(Template)]
#[template(path = "else")]
struct ElseTemplate {
    cond: bool,
}

#[test]
fn test_else_false() {
    let s = ElseTemplate { cond: false };
    assert_eq!(s.call().unwrap(), "     \n    false\n");
}

#[test]
fn test_else_true() {
    let s = ElseTemplate { cond: true };
    assert_eq!(s.call().unwrap(), "     \n true");
}

#[derive(Template)]
#[template(path = "else-if")]
struct ElseIfTemplate {
    cond: bool,
    check: bool,
}

#[test]
fn test_else_if() {
    let s = ElseIfTemplate {
        cond: false,
        check: true,
    };
    assert_eq!(s.call().unwrap(), " checked ");
}

#[derive(Template)]
#[template(path = "comment")]
struct CommentTemplate {}

#[test]
fn test_comment() {
    let t = CommentTemplate {};
    assert_eq!(t.call().unwrap(), "");
}

#[derive(Template)]
#[template(path = "negation")]
struct NegationTemplate {
    foo: bool,
}

#[test]
fn test_negation() {
    let t = NegationTemplate { foo: false };
    assert_eq!(t.call().unwrap(), "Hello");
}

#[derive(Template)]
#[template(path = "minus")]
struct MinusTemplate {
    foo: i8,
}

#[test]
fn test_minus() {
    let t = MinusTemplate { foo: 1 };
    assert_eq!(t.call().unwrap(), "Hello");
}

#[derive(Template)]
#[template(path = "index")]
struct IndexTemplate {
    foo: HashMap<String, String>,
}

#[test]
fn test_index() {
    let mut foo = HashMap::new();
    foo.insert("bar".into(), "baz".into());
    let t = IndexTemplate { foo };
    assert_eq!(t.call().unwrap(), "baz");
}

#[derive(Template)]
#[template(path = "tuple-attr")]
struct TupleAttrTemplate<'a>(&'a str, &'a str);

#[test]
fn test_tuple_attr() {
    let t = TupleAttrTemplate("foo", "bar");
    assert_eq!(t.call().unwrap(), "foobar");
}

struct Holder {
    a: usize,
}

struct NestedHolder {
    holder: Holder,
}

#[derive(Template)]
#[template(path = "nested-attr")]
struct NestedAttrTemplate {
    inner: NestedHolder,
}

#[test]
fn test_nested_attr() {
    let t = NestedAttrTemplate {
        inner: NestedHolder {
            holder: Holder { a: 5 },
        },
    };
    assert_eq!(t.call().unwrap(), "5");
}

#[derive(Template)]
#[template(path = "literals")]
struct LiteralsTemplate {}

#[test]
fn test_literals() {
    let s = LiteralsTemplate {};
    assert_eq!(s.call().unwrap(), "a");
}

#[derive(Template)]
#[template(path = "empty")]
struct Empty;

#[test]
fn test_empty() {
    assert_eq!(Empty.call().unwrap(), "foo");
}

struct Foo {
    a: usize,
}

#[derive(Template)]
#[template(path = "attr")]
struct AttrTemplate {
    inner: Foo,
}

#[test]
fn test_attr() {
    let t = AttrTemplate {
        inner: Foo { a: 1 },
    };
    assert_eq!(t.call().unwrap(), "1");
}

#[derive(Template)]
#[template(path = "option")]
struct OptionTemplate {
    var: Option<usize>,
}

#[test]
fn test_option() {
    let t = OptionTemplate { var: Some(1) };
    assert_eq!(t.call().unwrap(), "some: 1");
}

#[derive(Template)]
#[template(src = "{{{ string }}}")]
struct Unwrapped {
    string: String,
}

#[test]
fn test_unwrapped() {
    let t = Unwrapped {
        string: String::from("&"),
    };
    assert_eq!("&", t.call().unwrap());
}

#[derive(Template)]
#[template(src = "{{#if false }} {{$ \"foo\" }} {{/if }}")]
struct CompileError;

#[test]
fn test_compile_error() {
    assert_eq!("", CompileError.call().unwrap());
}

#[cfg(feature = "wasm")]
mod sd {
    use serde::Serialize;
    use yarte::Template;

    // Clone only for test not need in template
    #[derive(Serialize, Clone)]
    struct Foo {
        bar: String,
    }

    #[derive(Template)]
    #[template(src = "{{{ serde_json::to_string(&foo).map_err(|_| yarte::Error)? }}}")]
    struct Json<S: Serialize> {
        foo: S,
    }

    #[test]
    fn test_serialize_json() {
        let foo = Foo {
            bar: "foo".to_string(),
        };
        let t = Json { foo: foo.clone() };
        assert_eq!(serde_json::to_string(&foo).unwrap(), t.call().unwrap());
    }
}
