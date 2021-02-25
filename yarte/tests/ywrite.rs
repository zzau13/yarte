#![cfg(any(feature = "bytes-buf", feature = "bytes-buf-tokio2"))]
use yarte::{auto, yformat, yformat_html, ywrite, ywrite_html, BytesMut};

#[test]
fn test() {
    let world = "World";
    let res = yformat!("Hello {{ world }}!");

    assert_eq!(res, "Hello World!")
}

#[test]
fn test_w() {
    let mut bytes_mut = BytesMut::new();

    let world = "World";
    ywrite!(bytes_mut, "Hello {{ world }}!");

    assert_eq!(&bytes_mut[..], b"Hello World!");

    let mut bytes_mut = BytesMut::new();

    let world = 1;
    ywrite!(bytes_mut, "Hello {{ world }}!",);

    assert_eq!(&bytes_mut[..], b"Hello 1!")
}

#[test]
fn test_auto() {
    let world = "World";
    let bytes = auto!(ywrite!(BytesMut, "Hello {{ world }}!"));

    assert_eq!(&bytes[..], b"Hello World!");
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
    let name = "<>&\"";

    assert_eq!("Hello, &lt;&gt;&amp;&quot;!", yformat_html!("{{> hello }}"));
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

struct Card<'a> {
    title: &'a str,
    body: &'a str,
}

#[test]
fn resolve_partial_scope() {
    let my_card = Card {
        title: "My Title",
        body: "My Body",
    };

    let result = r#"<div class="entry">
  <h1>My Title</h1>
  <div class="body">
    My Body
  </div>
</div>"#;

    // Auto sized html
    let buf = auto!(ywrite_html!(String, "{{> hello_ex my_card }}"));
    assert_eq!(buf, result);

    let mut buf = String::new();
    ywrite_html!(buf, r#"{{> hello_ex my_card }}"#);
    assert_eq!(buf, result);
}

#[test]
fn resolve_partial_scope_overridden() {
    let my_card = Card {
        title: "My Title",
        body: "My Body",
    };

    // Auto sized html
    let buf = auto!(ywrite_html!(
        String,
        r#"{{> hello_ex my_card, body="foo" }}"#
    ));
    assert_eq!(
        buf,
        r#"<div class="entry">
  <h1>My Title</h1>
  <div class="body">
    foo
  </div>
</div>"#
    )
}
