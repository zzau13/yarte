use yarte::Template;

#[derive(Template)]
#[template(src = "{{# each yield foo }}{{/each }}")]
struct Test {
    foo: Vec<usize>
}

fn main() {}
