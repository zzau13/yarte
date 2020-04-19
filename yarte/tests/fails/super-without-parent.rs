use yarte::Template;

#[derive(Template)]
#[template(src = "{{ super::foo }}")]
struct Test;

fn main() {}
