use std::fs::read_to_string;
use yarte::Template;
use yarte_helpers::config::Config;

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

#[derive(Template)]
#[template(path = "issues/issue-224")]
struct I224 {
    url: String,
}

#[test]
fn i_224() {
    let t = I224 {
        url: "foo".to_string(),
    };
    let conf = Config::new("");
    let path = conf.get_dir().clone().join("issues/issue-224.txt");
    let expect = read_to_string(path).unwrap();
    assert_eq!(expect.trim(), t.call().unwrap());
}
