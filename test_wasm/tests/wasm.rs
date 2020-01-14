use yarte::Template;

#[derive(Template)]
#[template(src = "fortunes.hbs", print = "code", mode = "wasm")]
struct Test;

#[test]
fn test() {
    let app = Test::start_default();
}

