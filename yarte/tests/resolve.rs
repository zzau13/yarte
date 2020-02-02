#![allow(dead_code)]

use yarte::Template;

struct Foo {
    bar: usize,
}

#[derive(Template)]
#[template(src = "{{#with foo}}{{? bar }}{{/with}}")]
struct Resolve {
    foo: Foo,
}

#[test]
fn test() {
    assert_eq!(
        "self . foo . bar",
        Resolve {
            foo: Foo { bar: 0 }
        }
        .call()
        .unwrap()
    )
}
