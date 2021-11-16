use yarte::Template;

#[derive(Template)]
#[template(path = "hello")]
struct Card<'a> {
    title: &'a str,
    body: &'a str,
}

fn main() {
    println!(
        "{}",
        Card {
            title: "My Title",
            body: "My Body",
        }
    );
}
