use yarte::Template;

#[derive(Template)]
#[template(src = "{{ while foo {} }}")]
struct Test {
    foo: bool
}

fn main() {}
