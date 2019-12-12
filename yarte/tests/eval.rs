use yarte::Template;

#[derive(Template)]
#[template(src = "{{ \"foo\" + \"bar\" }}")]
struct AddStrTemplate;

#[test]
fn test_add_str() {
    let t = AddStrTemplate;
    assert_eq!(t.call().unwrap(), "foobar");
}

#[derive(Template)]
#[template(src = "{{ \"foo\" + \"bar\" * 2 }}")]
struct MulStrTemplate;

#[test]
fn test_mul_str() {
    let t = MulStrTemplate;
    assert_eq!(t.call().unwrap(), "foobarbar");
}

#[derive(Template)]
#[template(src = "{{ (\"foo\" + \"bar\") * 2 }}")]
struct MulParenStrTemplate;

#[test]
fn test_mul_paren_str() {
    let t = MulParenStrTemplate;
    assert_eq!(t.call().unwrap(), "foobarfoobar");
}
