use yarte::Template;

#[derive(Template)]
#[template(
    src = "Hello, {{#each this~}}
        {{#each this.as_bytes() ~}}
            {{ super::index0 }} {{ super::super::this[0] }}
        {{~/each }}{{ super::this[0] }}
    {{~/each}}!",
    ext = "txt"
)]
struct HelloSuperTemplate<'a> {
    this: &'a [&'a str],
}

#[test]
fn test_hello() {
    let t = HelloSuperTemplate { this: &["world"] };
    assert_eq!(
        "Hello, 0 world0 world0 world0 world0 worldworld!",
        t.call().unwrap()
    );
}

#[derive(Template)]
#[template(
    src = "Hello, {{#each this~}}
            {{#with this}}{{ super::hold }}{{ hold }}{{/with}}
    {{~/each}}!",
    ext = "txt"
)]
struct WithSuperTemplate<'a> {
    this: &'a [Holder],
}

struct Holder {
    hold: usize,
}

#[test]
fn test_with() {
    let t = WithSuperTemplate {
        this: &[Holder { hold: 127 }],
    };
    assert_eq!("Hello, 127127!", t.call().unwrap());
}

#[derive(Template)]
#[template(
    src = "Hello, {{#each this~}}
            {{#with this}}{{ super::hold }}{{ hold }}{{ super::index }}{{/with}}
    {{~/each}}!",
    ext = "txt"
)]
struct WithSuperIndexTemplate<'a> {
    this: &'a [Holder],
}

#[test]
fn test_with_index() {
    let t = WithSuperIndexTemplate {
        this: &[Holder { hold: 127 }],
    };
    assert_eq!("Hello, 1271271!", t.call().unwrap());
}
