use yarte::Template;

#[derive(Template)]
#[template(src = "<h1>{{ content }}</h1>")]
struct I125 {
    content: char,
}

#[test]
fn i_125() {
    let t = I125 { content: '<' };
    assert_eq!("<h1>&lt;</h1>", t.call().unwrap());
}
