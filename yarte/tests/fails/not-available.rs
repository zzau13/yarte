use yarte::Template;

#[derive(Template)]
#[template(src = "{{ { yield foo } }}")]
struct Test;

fn main() {}
