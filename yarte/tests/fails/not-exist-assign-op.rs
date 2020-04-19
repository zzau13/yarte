use yarte::Template;

#[derive(Template)]
#[template(src = "{{ {
    no_exist *= 0;
    no_exist
} }}")]
struct TestMul;

fn main() {}
