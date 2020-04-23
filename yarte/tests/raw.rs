#![cfg(feature = "html-min")]
use yarte::TemplateMin;

#[derive(TemplateMin)]
#[template(path = "raw")]
struct RawTemplate;

#[test]
fn test_raw() {
    let raw = RawTemplate;
    assert_eq!("{{#each example}}{{/each}}", raw.call().unwrap());
}

#[derive(TemplateMin)]
#[template(path = "raw-partial")]
struct RawPartialTemplate;

#[test]
fn test_raw_partial() {
    let raw = RawPartialTemplate;
    assert_eq!("{{&gt; partial }}", raw.call().unwrap());
}
