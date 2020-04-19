use yarte::Template;

#[derive(Template)]
#[template(src = "{{ {
    no_exist = 0;
    no_exist
} }}")]
struct Test;

fn main() {}
