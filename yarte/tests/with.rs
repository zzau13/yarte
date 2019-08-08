#![allow(clippy::into_iter_on_ref)]

use yarte::Template;

struct Holder {
    foo: usize,
    bar: usize,
}

#[derive(Template)]
#[template(path = "with.hbs")]

struct WithTemplate {
    hold: Holder,
}

#[test]
fn test_with() {
    let hello = WithTemplate {
        hold: Holder { foo: 0, bar: 1 },
    };
    assert_eq!("0 1", hello.call().unwrap());
}

struct DeepHold {
    deep: Holder,
}

#[derive(Template)]
#[template(path = "with-each.hbs")]

struct WithEachTemplate {
    hold: Vec<DeepHold>,
}

#[test]
fn test_with_each() {
    let hello = WithEachTemplate {
        hold: vec![DeepHold {
            deep: Holder { foo: 0, bar: 1 },
        }],
    };
    assert_eq!("0 1", hello.call().unwrap());
}
