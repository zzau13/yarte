use yarte::Template;

#[derive(Template)]
#[template(src = "{{ @foo }}")]
struct Test{
    foo: usize
}

fn main() {}