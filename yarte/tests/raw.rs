use yarte::Template;

#[derive(Template)]
#[template(path = "raw.hbs")]
struct RawTemplate;

#[test]
fn test_raw() {
    let raw = RawTemplate;
    assert_eq!("{{#each example}}{{/each}}", raw.call().unwrap());
}

#[derive(Template)]
#[template(path = "raw-partial.hbs", mode = "min")]
struct RawPartialTemplate;

#[test]
fn test_raw_partial() {
    let raw = RawPartialTemplate;
    assert_eq!("{{&gt; partial }}", raw.call().unwrap());
}
