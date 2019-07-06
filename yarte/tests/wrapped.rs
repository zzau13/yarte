use yarte::Template;

#[derive(Template)]
#[template(path = "wrapped-if.hbs")]
struct IfTemplate {
    cond: bool,
}

#[test]
fn test_if() {
    let t = IfTemplate { cond: true };
    assert_eq!("&amp;", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "wrapped-index.hbs")]
struct IndexTemplate<'a> {
    arr: Vec<&'a str>,
}

#[test]
fn test_index() {
    let t = IndexTemplate { arr: vec!["&"] };
    assert_eq!("&amp;", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "wrapped-slice.hbs")]
struct SliceTemplate<'a> {
    arr: &'a [&'a str],
}

#[test]
fn test_slice() {
    let arr: &[&str] = &vec!["&"];
    let t = SliceTemplate { arr };
    assert_eq!("&amp;", t.call().unwrap());
}

fn repeat(s: &str, i: usize) -> String {
    s.repeat(i)
}

#[derive(Template)]
#[template(path = "wrapped-call.hbs")]
struct CallTemplate<'a> {
    s: &'a str,
}

#[test]
fn test_call() {
    let t = CallTemplate { s: "&" };
    assert_eq!("&amp;&amp;", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "wrapped-array.hbs")]
struct ArrayTemplate;

#[test]
fn test_array() {
    let t = ArrayTemplate;
    assert_eq!("&amp;", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "wrapped-tuple.hbs")]
struct TupleTemplate;

#[test]
fn test_tuple() {
    let t = TupleTemplate;
    assert_eq!("&amp;", t.call().unwrap());
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
#[template(src = "{{{ debug }}}", ext = "hbs")]
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

#[derive(Template)]
#[template(
    src = "{{ cond }}{{ 1 + num }}{{ num }}{{ cond || true }}{{ 1.0 }}{{ [true][0..1][0] }}",
    ext = "hbs",
    print = "code"
)]
struct WrappedDebug {
    cond: bool,
    num: usize,
}

#[test]
fn test_wrapped_debug() {
    let t = WrappedDebug { cond: true, num: 0 };
    assert_eq!("true10true1true", t.call().unwrap());
}
