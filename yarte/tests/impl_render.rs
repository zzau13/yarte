use std::fmt::{self, Display, Formatter};
use yarte::{Render, Template};

struct Rendered;

// Safe text
static HELLO: &str = "Hello World!";

impl Render for Rendered {
    fn render(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        HELLO.fmt(f)
    }
}

#[derive(Template)]
#[template(src = "{{ rendered }}")]
struct RenderTemplate {
    rendered: Rendered,
}

#[test]
fn test_impl_render() {
    let s = RenderTemplate { rendered: Rendered };
    assert_eq!(HELLO, s.call().unwrap());
}
