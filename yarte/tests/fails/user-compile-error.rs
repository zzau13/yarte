use yarte::Template;

#[derive(Template)]
#[template(src = "{{#if true}} {{$ \"foo\" }} {{/if }}")]
struct Test{
    foo: usize
}

fn main() {}
