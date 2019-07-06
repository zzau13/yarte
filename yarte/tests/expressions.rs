use std::fmt::Error;
use yarte::{Result, Template};

#[derive(Template)]
#[template(src = "Hello, {{ name }}!", ext = "txt")]
struct HelloTemplate<'a> {
    name: &'a str,
}

#[test]
fn test_hello() {
    let t = HelloTemplate { name: "world" };
    assert_eq!("Hello, world!", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "hello.txt")]
struct HelloTxtTemplate<'a> {
    name: &'a str,
}

#[test]
fn test_hello_txt() {
    let t = HelloTxtTemplate { name: "world" };
    assert_eq!("Hello, world!", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "expr-ignore.hbs")]
struct IgnoreTemplate {
    cond: Option<bool>,
}

#[test]
fn test_ignore() {
    let t = IgnoreTemplate { cond: Some(false) };
    assert_eq!("foo", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "expr-closure.hbs")]
struct ClosureTemplate<'a> {
    name: &'a str,
}

#[test]
fn test_closure() {
    let t = ClosureTemplate { name: "world" };
    assert_eq!("true", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "expr-let-closure.hbs")]
struct LetClosureTemplate<'a> {
    name: &'a str,
}

#[test]
fn test_let_closure() {
    let t = LetClosureTemplate { name: "world" };
    assert_eq!("worldworld", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "expr-let-closure-scope.hbs")]
struct LetClosureScopeTemplate<'a> {
    name: &'a str,
}

#[test]
fn test_let_closure_scope() {
    let t = LetClosureScopeTemplate { name: "world" };
    assert_eq!("world", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "expr-let.hbs")]
struct LetTemplate<'a> {
    name: &'a str,
}

#[test]
fn test_let() {
    let t = LetTemplate { name: "world" };
    assert_eq!("Hello, world!", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "expr-withfields.hbs")]
struct WithFieldsTemplate<'a> {
    names: (Name<'a>, Name<'a>),
}

struct Name<'a> {
    first: &'a str,
    last: &'a str,
}

#[test]
fn test_with_fields() {
    let t = WithFieldsTemplate {
        names: (
            Name {
                first: "foo",
                last: "bar",
            },
            Name {
                first: "fOO",
                last: "bAR",
            },
        ),
    };
    assert_eq!("Hello, foo bar and fOO bAR!", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "expr-struct.hbs")]
struct StructTemplate;

#[test]
fn test_struct() {
    let t = StructTemplate;
    assert_eq!("foo &amp;", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "expr-letwith.hbs")]
struct LetWithTemplate<'a> {
    name: &'a str,
}

#[test]
fn test_let_with() {
    let t = LetWithTemplate { name: "world" };
    assert_eq!("Hello, worldworld!", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "expr-let-if.hbs")]
struct LetIfTemplate {
    cond: bool,
}

#[test]
fn test_let_if() {
    let t = LetIfTemplate { cond: true };
    assert_eq!("Hello, true false foo!", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "expr-let-if-some.hbs")]
struct LetIfSomeTemplate {
    cond: Option<bool>,
}

#[test]
fn test_let_if_some() {
    let t = LetIfSomeTemplate { cond: Some(false) };
    assert_eq!("Hello, bar!", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "expr-elif-some.hbs")]
struct LetElseIfSomeTemplate {
    cond: Option<bool>,
    check: Option<bool>,
}

#[test]
fn test_let_else_if_some() {
    let t = LetElseIfSomeTemplate {
        cond: Some(false),
        check: Some(false),
    };
    assert_eq!("Hello, bar!", t.call().unwrap());
    let t = LetElseIfSomeTemplate {
        cond: Some(true),
        check: Some(false),
    };
    assert_eq!("Hello, foo!", t.call().unwrap());
    let t = LetElseIfSomeTemplate {
        cond: None,
        check: Some(true),
    };
    assert_eq!("Hello, baa!", t.call().unwrap());
    let t = LetElseIfSomeTemplate {
        cond: None,
        check: Some(false),
    };
    assert_eq!("Hello, fun!", t.call().unwrap());
    let t = LetElseIfSomeTemplate {
        cond: None,
        check: None,
    };
    assert_eq!("Hello, None!", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "expr-let-elif-each-some.hbs")]
struct LetElseIfEachSomeTemplate {
    conditions: Vec<Cond>,
}

struct Cond {
    cond: Option<bool>,
    check: Option<bool>,
}

#[test]
fn test_let_else_if_each_some() {
    let mut conditions = vec![];
    for _ in 0..5 {
        conditions.push(Cond {
            cond: Some(false),
            check: Some(false),
        })
    }

    let t = LetElseIfEachSomeTemplate { conditions };
    assert_eq!("Hello, barbarbarbarbar!", t.call().unwrap());

    let mut conditions = vec![];
    for _ in 0..5 {
        conditions.push(Cond {
            cond: None,
            check: None,
        })
    }

    let t = LetElseIfEachSomeTemplate { conditions };
    assert_eq!("Hello, NoneNoneNoneNoneNone!", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "expr-letloop.hbs")]
struct LetLoopTemplate<'a> {
    name: &'a str,
}

#[test]
fn test_let_loop() {
    let t = LetLoopTemplate { name: "&foo" };
    assert_eq!("&amp;foo!&amp;foo!", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "expr-letcollect.hbs")]
struct LetCollectTemplate {
    a: Vec<usize>,
}

#[test]
fn test_let_collect() {
    let t = LetCollectTemplate { a: vec![0, 1] };
    assert_eq!("13", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "expr-try.hbs")]
struct TryTemplate {
    a: yarte::Result<usize>,
}

#[test]
fn test_try() {
    let t = TryTemplate { a: Err(Error) };
    assert!(t.call().is_err());

    let t = TryTemplate { a: Ok(1) };
    assert_eq!("1", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "expr-trymethod.hbs")]
struct TryMethodTemplate {
    some: bool,
}

impl TryMethodTemplate {
    fn not_is(&self, some: bool) -> Result<bool> {
        if some {
            Ok(false)
        } else {
            Err(Error)
        }
    }
}

#[test]
fn test_try_method() {
    let t = TryMethodTemplate { some: false };
    assert!(t.call().is_err());

    let t = TryMethodTemplate { some: true };
    assert_eq!("foo", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "expr-unsafe.hbs")]
struct UnsafeTemplate {
    s: Vec<usize>,
}

#[test]
fn test_unsafe() {
    let t = UnsafeTemplate { s: vec![0] };
    assert_eq!("0", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "expr-assign.hbs")]
struct AssignAtLoopTemplate;

#[test]
fn test_assign_at_loop() {
    let t = AssignAtLoopTemplate;
    assert_eq!("1", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "expr-assign-op.hbs")]
struct AssignOpAtLoopTemplate;

#[test]
fn test_assign_op_at_loop() {
    let t = AssignOpAtLoopTemplate;
    assert_eq!("1", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "expr-macros.hbs")]
struct MacrosTemplate;

#[test]
fn test_macro() {
    let template = MacrosTemplate {};
    assert_eq!("Hello, world!", template.call().unwrap());
}

#[derive(Template)]
#[template(path = "expr-for-loop.hbs")]
struct ForLoopTemplate {
    name: String,
}

#[test]
fn test_for_loop() {
    let t = ForLoopTemplate {
        name: "&foo".to_owned(),
    };
    assert_eq!("16", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "expr-match.hbs")]
struct MatchTemplate<'a> {
    a: yarte::Result<&'a str>,
}

#[test]
fn test_match() {
    let t = MatchTemplate { a: Err(Error) };
    assert_eq!("&amp;", t.call().unwrap());

    let t = MatchTemplate { a: Ok("1") };
    assert_eq!("1", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "self-method.hbs")]
struct SelfMethodTemplate<'a> {
    s: &'a str,
}

impl<'a> SelfMethodTemplate<'a> {
    fn get_s(&self) -> &str {
        self.s
    }
}

#[test]
fn test_self_method() {
    let t = SelfMethodTemplate { s: "foo" };
    assert_eq!(t.call().unwrap(), "foo");
}

#[derive(Template)]
#[template(path = "nested-self-method.hbs")]
struct NestedSelfMethodTemplate<'a> {
    t: SelfMethodTemplate<'a>,
}

impl<'a> NestedSelfMethodTemplate<'a> {
    fn get_s(&self) -> &str {
        "bar"
    }
}

#[test]
fn test_nested() {
    let t = NestedSelfMethodTemplate {
        t: SelfMethodTemplate { s: "foo" },
    };
    assert_eq!(t.call().unwrap(), "bar foo");
}

static RAW_HBS: &str = "{{R }} {{/R}}";

#[derive(Template)]
#[template(src = "{{ RAW_HBS }}")]
struct RawStaticTemplate;

#[test]
fn test_raw_static() {
    let t = RawStaticTemplate;
    assert_eq!(t.call().unwrap(), "{{R }} {{&#x2f;R}}");
}

fn hello() -> &'static str {
    "&"
}

#[derive(Template)]
#[template(path = "expr-call.hbs")]
struct CallTemplate;

#[test]
fn test_call() {
    let t = CallTemplate;
    assert_eq!(t.call().unwrap(), "Hello, &amp;!");
}
