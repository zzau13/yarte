use yarte::Template;

#[derive(Template)]
#[template(src = "{{ super }}")]
struct Test {
    foo: bool
}

fn main() {}
