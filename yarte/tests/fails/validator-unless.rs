use yarte::Template;

#[derive(Template)]
#[template(src = "{{# unless yield foo }}{{/unless }}")]
struct Test {
    foo: bool
}

fn main() {}
