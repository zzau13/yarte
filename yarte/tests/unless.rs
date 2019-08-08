#![allow(clippy::into_iter_on_ref)]

use yarte::Template;

#[derive(Template)]
#[template(path = "unless-template.hbs")]
struct UnlessTemplate {
    license: bool,
}

#[test]
fn test_unless() {
    let t = UnlessTemplate { license: true };
    assert_eq!("<div class=\"entry\"></div>", t.call().unwrap());

    let t = UnlessTemplate { license: false };
    assert_eq!(
        "<div class=\"entry\">\
         <h3 class=\"warning\">WARNING: This entry does not have a license!</h3>\
         </div>",
        t.call().unwrap()
    );
}

#[derive(Template)]
#[template(path = "people-template.hbs")]
struct PeopleTemplate<'a> {
    people: &'a [&'a str],
}

#[test]
fn test_people_template() {
    let s = &PeopleTemplate { people: &["foo"] };
    assert_eq!(
        s.call().unwrap(),
        "<ul class=\"people_list\"><li>foo</li></ul>"
    )
}

#[derive(Template)]
#[template(path = "paragraph-template.hbs")]
struct ParagraphTemplate<'a> {
    paragraphs: &'a [&'a str],
}

#[test]
fn test_paragraph_template() {
    let t = ParagraphTemplate {
        paragraphs: &["bar"],
    };
    assert_eq!(t.call().unwrap(), "<p>bar</p>");
    let t = ParagraphTemplate { paragraphs: &[] };
    assert_eq!(t.call().unwrap(), "<p class=\"empty\">No content</p>");
}

#[derive(Template)]
#[template(path = "array-template.hbs")]
struct ArrayTemplate<'a> {
    array: &'a [&'a str],
}

#[test]
fn test_array_template() {
    let t = ArrayTemplate {
        array: &["foo", "bar", "baz"],
    };
    assert_eq!(t.call().unwrap(), "1foo2bar3baz");
}

#[derive(Template)]
#[template(path = "title-template.hbs")]
struct AuthorTemplate<'a> {
    title: &'a str,
    author: Author<'a>,
}

struct Author<'a> {
    first_name: &'a str,
    last_name: &'a str,
}

#[test]
fn author_template() {
    let t = AuthorTemplate {
        title: "An Essay on the Psychology of Invention in the Mathematical Field",
        author: Author {
            first_name: "Jacques",
            last_name: "Hadamard",
        },
    };
    assert_eq!(t.call().unwrap(), "<div class=\"entry\"><h1>An Essay on the Psychology of Invention in the Mathematical Field</h1><h2>By Jacques Hadamard</h2></div>");
}
