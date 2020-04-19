use yarte::Template;

#[derive(Template)]
#[template(src = "{{# unless !foo }}{{/unless }}")]
struct Test{
    foo: bool
}

fn main() {}
