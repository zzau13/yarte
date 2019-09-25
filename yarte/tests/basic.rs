#![allow(clippy::blacklisted_name)]

use std::collections::HashMap;

use yarte::Template;

#[derive(Template)]
#[template(path = "hello.hbs")]

struct HelloTemplate<'a> {
    name: &'a str,
}

#[test]
fn test_hello() {
    let hello = HelloTemplate { name: "world" };
    assert_eq!("Hello, world!", hello.call().unwrap());
}

#[derive(Template)]
#[template(src = "{{}", ext = "txt")]

struct BracketsTemplate;

#[test]
fn test_brackets() {
    let hello = BracketsTemplate;
    assert_eq!("{{}", hello.call().unwrap());
}

#[derive(Template)]
#[template(src = "{{{}}", ext = "txt")]

struct Brackets2Template;

#[test]
fn test_brackets2() {
    let hello = Brackets2Template;
    assert_eq!("{{{}}", hello.call().unwrap());
}
#[derive(Template)]
#[template(path = "simple.hbs")]
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
        "hello world, foo\n\
         with number: 42\n\
         Iñtërnâtiônàlizætiøn is important\n\
         in vars too: Iñtërnâtiônàlizætiøn"
    );
    assert_eq!(VariablesTemplate::mime(), "text/html; charset=utf-8");
}

#[derive(Template)]
#[template(path = "hello.hbs")]
struct EscapeTemplate<'a> {
    name: &'a str,
}

#[test]
fn test_escape() {
    let s = EscapeTemplate { name: "<>&\"'/" };

    assert_eq!(s.call().unwrap(), "Hello, &lt;&gt;&amp;&quot;&#x27;&#x2f;!");
}

#[derive(Template)]
#[template(path = "simple-no-escape.txt")]
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
        "hello world, foo\n\
         with number: 42\n\
         Iñtërnâtiônàlizætiøn is important\n\
         in vars too: Iñtërnâtiônàlizætiøn"
    );
}

#[derive(Template)]
#[template(path = "if.hbs")]
struct IfTemplate {
    cond: bool,
}

#[test]
fn test_if() {
    let s = IfTemplate { cond: true };
    assert_eq!(s.call().unwrap(), "true");
}

#[derive(Template)]
#[template(path = "else.hbs")]
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
#[template(path = "else-if.hbs")]
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
#[template(path = "comment.hbs")]
struct CommentTemplate {}

#[test]
fn test_comment() {
    let t = CommentTemplate {};
    assert_eq!(t.call().unwrap(), "");
}

#[derive(Template)]
#[template(path = "negation.hbs")]
struct NegationTemplate {
    foo: bool,
}

#[test]
fn test_negation() {
    let t = NegationTemplate { foo: false };
    assert_eq!(t.call().unwrap(), "Hello");
}

#[derive(Template)]
#[template(path = "minus.hbs")]
struct MinusTemplate {
    foo: i8,
}

#[test]
fn test_minus() {
    let t = MinusTemplate { foo: 1 };
    assert_eq!(t.call().unwrap(), "Hello");
}

#[derive(Template)]
#[template(path = "index.hbs")]
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
#[template(path = "tuple-attr.hbs")]
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
#[template(path = "nested-attr.hbs")]
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
#[template(path = "literals.hbs")]
struct LiteralsTemplate {}

#[test]
fn test_literals() {
    let s = LiteralsTemplate {};
    assert_eq!(s.call().unwrap(), "a");
}

#[derive(Template)]
#[template(path = "empty.hbs")]
struct Empty;

#[test]
fn test_empty() {
    assert_eq!(Empty.call().unwrap(), "foo");
}

struct Foo {
    a: usize,
}

#[derive(Template)]
#[template(path = "attr.hbs")]
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
#[template(path = "option.hbs")]
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
