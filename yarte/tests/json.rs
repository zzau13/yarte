// adapted from [`serde-json`](https://github.com/serde-rs/json)
#![cfg(feature = "json")]
#![allow(
    clippy::excessive_precision,
    clippy::float_cmp,
    clippy::unreadable_literal
)]

use std::collections::BTreeMap;
use std::f64;
use std::fmt::Debug;
use std::i64;
use std::string::ToString;
use std::u64;

use yarte::{to_bytes, Serialize};

macro_rules! treemap {
    () => {
        BTreeMap::new()
    };
    ($($k:expr => $v:expr),+) => {
        {
            let mut m = BTreeMap::new();
            $(
                m.insert($k, $v);
            )+
            m
        }
    };
}

#[derive(Clone, Debug, PartialEq, Serialize)]
enum Animal {
    Dog,
    Frog(String, Vec<isize>),
    Cat { age: usize, name: String },
    AntHive(Vec<String>),
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct Inner {
    a: (),
    b: usize,
    c: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct Outer {
    inner: Vec<Inner>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct Tuple {
    tuple: (u8, u8, u8, u8),
}

fn test_encode_ok<T>(errors: &[(T, &str)])
where
    T: PartialEq + Debug + Serialize,
{
    for &(ref value, out) in errors {
        let out = out.to_string();

        let s = to_bytes::<String, _>(value, 0);
        assert_eq!(s, out);
    }
}

#[test]
fn test_write_null() {
    let tests = &[((), "null")];
    test_encode_ok(tests);
}

#[test]
fn test_write_u64() {
    let tests = &[(3u64, "3"), (u64::MAX, &u64::MAX.to_string())];
    test_encode_ok(tests);
}

#[test]
fn test_write_i64() {
    let tests = &[
        (3i64, "3"),
        (-2i64, "-2"),
        (-1234i64, "-1234"),
        (i64::MIN, &i64::MIN.to_string()),
    ];
    test_encode_ok(tests);
}

#[test]
fn test_write_f64() {
    let tests = &[
        (3.0, "3.0"),
        (3.1, "3.1"),
        (-1.5, "-1.5"),
        (0.5, "0.5"),
        (f64::MIN, "-1.7976931348623157e308"),
        (f64::MAX, "1.7976931348623157e308"),
        (f64::EPSILON, "2.220446049250313e-16"),
    ];
    test_encode_ok(tests);
}

#[test]
fn test_encode_nonfinite_float_yields_null() {
    let e = String::from("null");
    let v = to_bytes::<String, _>(&f64::NAN, 0);
    assert_eq!(v, e);

    let v = to_bytes::<String, _>(&f64::INFINITY, 0);
    assert_eq!(v, e);

    let v = to_bytes::<String, _>(&f32::NAN, 0);
    assert_eq!(v, e);

    let v = to_bytes::<String, _>(&f32::INFINITY, 0);
    assert_eq!(v, e);
}

#[test]
fn test_write_str() {
    let tests = &[("", "\"\""), ("foo", "\"foo\"")];
    test_encode_ok(tests);
}

#[test]
fn test_write_bool() {
    let tests = &[(true, "true"), (false, "false")];
    test_encode_ok(tests);
}

#[test]
fn test_write_char() {
    let tests = &[
        ('n', "\"n\""),
        ('"', "\"\\\"\""),
        ('\\', "\"\\\\\""),
        ('/', "\"/\""),
        ('\x08', "\"\\b\""),
        ('\x0C', "\"\\f\""),
        ('\n', "\"\\n\""),
        ('\r', "\"\\r\""),
        ('\t', "\"\\t\""),
        ('\x0B', "\"\\u000b\""),
        ('\u{3A3}', "\"\u{3A3}\""),
    ];
    test_encode_ok(tests);
}

#[test]
fn test_write_object() {
    test_encode_ok(&[
        (treemap!(), "{}"),
        (treemap!("a".to_string() => true), "{\"a\":true}"),
        (
            treemap!(
                "a".to_string() => true,
                "b".to_string() => false
            ),
            "{\"a\":true,\"b\":false}",
        ),
    ]);

    test_encode_ok(&[
        (
            treemap![
                "a".to_string() => treemap![],
                "b".to_string() => treemap![],
                "c".to_string() => treemap![]
            ],
            "{\"a\":{},\"b\":{},\"c\":{}}",
        ),
        (
            treemap![
                "a".to_string() => treemap![
                    "a".to_string() => treemap!["a" => vec![1,2,3]],
                    "b".to_string() => treemap![],
                    "c".to_string() => treemap![]
                ],
                "b".to_string() => treemap![],
                "c".to_string() => treemap![]
            ],
            "{\"a\":{\"a\":{\"a\":[1,2,3]},\"b\":{},\"c\":{}},\"b\":{},\"c\":{}}",
        ),
        (
            treemap![
                "a".to_string() => treemap![],
                "b".to_string() => treemap![
                    "a".to_string() => treemap!["a" => vec![1,2,3]],
                    "b".to_string() => treemap![],
                    "c".to_string() => treemap![]
                ],
                "c".to_string() => treemap![]
            ],
            "{\"a\":{},\"b\":{\"a\":{\"a\":[1,2,3]},\"b\":{},\"c\":{}},\"c\":{}}",
        ),
        (
            treemap![
                "a".to_string() => treemap![],
                "b".to_string() => treemap![],
                "c".to_string() => treemap![
                    "a".to_string() => treemap!["a" => vec![1,2,3]],
                    "b".to_string() => treemap![],
                    "c".to_string() => treemap![]
                ]
            ],
            "{\"a\":{},\"b\":{},\"c\":{\"a\":{\"a\":[1,2,3]},\"b\":{},\"c\":{}}}",
        ),
    ]);

    test_encode_ok(&[(treemap!['c' => ()], "{\"c\":null}")]);
}

#[test]
fn test_write_tuple() {
    test_encode_ok(&[((5,), "[5]")]);
    test_encode_ok(&[((5, (6, "abc")), "[5,[6,\"abc\"]]")]);
}

#[test]
fn test_write_enum() {
    test_encode_ok(&[
        (Animal::Dog, "\"Dog\""),
        (
            Animal::Frog("Henry".to_string(), vec![]),
            "{\"Frog\":[\"Henry\",[]]}",
        ),
        (
            Animal::Frog("Henry".to_string(), vec![349]),
            "{\"Frog\":[\"Henry\",[349]]}",
        ),
        (
            Animal::Frog("Henry".to_string(), vec![349, 102]),
            "{\"Frog\":[\"Henry\",[349,102]]}",
        ),
        (
            Animal::Cat {
                age: 5,
                name: "Kate".to_string(),
            },
            "{\"Cat\":{\"age\":5,\"name\":\"Kate\"}}",
        ),
        (
            Animal::AntHive(vec!["Bob".to_string(), "Stuart".to_string()]),
            "{\"AntHive\":[\"Bob\",\"Stuart\"]}",
        ),
    ]);
}

#[test]
fn test_tuple() {
    test_encode_ok(&[(
        &Tuple {
            tuple: (0, 2, 0, 1),
        },
        "{\"tuple\":[0,2,0,1]}",
    )])
}

// Adapted from [`simd-json-derive`](https://github.com/simd-lite/simd-json-derive)
#[test]
fn unnamed1() {
    #[derive(Serialize, PartialEq, Debug)]
    struct Bla(u8);
    let b = Bla(1);
    let e = r#"1"#;
    test_encode_ok(&[(b, e)]);
}

#[test]
fn unnamed2() {
    #[derive(Serialize, PartialEq, Debug)]
    struct Bla(u8, u16);
    let b = Bla(1, 2);
    let e = r#"[1,2]"#;
    test_encode_ok(&[(b, e)]);
}

#[test]
fn named() {
    #[derive(Serialize, PartialEq, Debug)]
    struct Bla {
        f1: u8,
        f2: String,
    }

    let b = Bla {
        f1: 1,
        f2: "snot".into(),
    };

    let e = r#"{"f1":1,"f2":"snot"}"#;
    test_encode_ok(&[(b, e)]);
}

#[test]
fn unnamed1_lifetime() {
    #[derive(Serialize, PartialEq, Debug)]
    struct BlaU1L<'a>(&'a str);
    let b = BlaU1L("snot");

    let e = r#""snot""#;
    test_encode_ok(&[(b, e)]);
}
#[test]
fn unnamed2_lifetime() {
    #[derive(Serialize, PartialEq, Debug)]
    struct BlaU2L<'a, 'b>(&'a str, &'b str);
    let b = BlaU2L("hello", "world");

    let e = r#"["hello","world"]"#;
    test_encode_ok(&[(b, e)]);
}

#[test]
fn named_lifetime() {
    #[derive(Serialize, PartialEq, Debug)]
    struct BlaN2L<'a, 'b> {
        f1: &'a str,
        f2: &'b str,
    }

    let b = BlaN2L {
        f1: "snot",
        f2: "badger",
    };

    let e = r#"{"f1":"snot","f2":"badger"}"#;
    test_encode_ok(&[(b, e)]);
}
