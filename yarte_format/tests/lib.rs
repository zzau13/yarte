#![cfg(nightly)]
#![feature(proc_macro_hygiene)]

use yarte_format::{yformat, yformat_html};

#[test]
fn test() {
    let world = "World";
    let res = yformat!("Hello {{ world }}!");

    eprintln!("{}", res);
    assert_eq!(res, "Hello World!")
}

#[test]
fn test_hello() {
    let name = "World";
    let res = yformat!("{{> hello }}");

    assert_eq!(res, "Hello, World!")
}

#[test]
fn test_brackets() {
    assert_eq!("{{}", yformat!("{{}"));
}

#[test]
fn test_brackets2() {
    assert_eq!("{{{}", yformat!("{{{}"));
}

#[test]
fn test_variables() {
    let strvar = "foo";
    let num = 42;
    let i18n = "Iñtërnâtiônàlizætiøn";
    assert_eq!(
        "hello world, foo\nwith number: 42\nIñtërnâtiônàlizætiøn is important\nin vars too: \
         Iñtërnâtiônàlizætiøn",
        yformat!("{{> simple }}")
    );
}

#[test]
fn test_escape() {
    let name = "<>&\"'/";

    assert_eq!(
        "Hello, &lt;&gt;&amp;&quot;&#x27;&#x2f;!",
        yformat_html!("{{> hello }}")
    );
}

#[test]
fn test_if() {
    let cond = true;
    assert_eq!("true", yformat!("{{> if }}"));
}

#[test]
fn test_else_false() {
    let cond = false;
    assert_eq!("     \n    false\n", yformat!("{{> else }}"));
}

#[test]
fn test_else_true() {
    let cond = true;
    assert_eq!("     \n true", yformat!("{{> else }}"));
}

#[test]
fn test_else_if() {
    let cond = false;
    let check = true;
    assert_eq!(" checked ", yformat!("{{> else-if }}"));
}

#[test]
fn test_comment() {
    assert_eq!("", yformat!("{{> comment }}"));
}

#[test]
fn test_noescape() {
    let a = "&";

    assert_eq!("&", yformat!("{{ a }}"));
}

#[test]
#[cfg(feature = "json")]
fn test_json() {
    use serde::Serialize;
    use serde_json::{to_string, to_string_pretty};
    #[derive(Serialize)]
    struct Json {
        f: usize,
    }

    let val = Json { f: 1 };
    assert_eq!(to_string(&val).unwrap(), yformat!("{{ @json val }}"));
    assert_eq!(
        to_string_pretty(&val).unwrap(),
        yformat!("{{ @json_pretty val }}")
    );
}
