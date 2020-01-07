use yarte::Template;

#[derive(Template)]
#[template(path = "title.hbs")]
struct TitleTemplate<'a> {
    title: &'a str,
}

#[test]
fn test_title() {
    let t = TitleTemplate { title: "foo" };
    assert_eq!("<h1>foo</h1>", t.call().unwrap());
}

#[derive(Template)]
#[template(path = "article-title-template.hbs")]
struct ArticleTitleTemplate<'a> {
    article: Article<'a>,
}

struct Article<'a> {
    title: &'a str,
}

#[test]
fn test_article_title() {
    let t = ArticleTitleTemplate {
        article: Article { title: "bar" },
    };
    assert_eq!("<h1>bar</h1> <h2>bar</h2>", t.call().unwrap());
}
