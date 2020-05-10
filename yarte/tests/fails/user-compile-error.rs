use yarte::Template;

#[derive(Template)]
#[template(src = "{{#if true.is_some() }}
{{$ \"OMG! true is some\" }} {{/if }}")]
struct Test{
    foo: usize
}

fn main() {}
