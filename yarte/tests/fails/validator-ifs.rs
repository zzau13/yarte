use yarte::Template;

#[derive(Template)]
#[template(src = "{{# if yield foo }}{{/if }}")]
struct Test {
    foo: bool
}

fn main() {}
