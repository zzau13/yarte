// adapted from [`serde-json`](https://github.com/serde-rs/json)

#![allow(
    clippy::excessive_precision,
    clippy::float_cmp,
    clippy::unreadable_literal
)]

use serde::ser::{self};
use serde::{de, Serialize};
use serde_json::{json, to_value};

use std::collections::BTreeMap;
use std::f64;
use std::fmt::{self, Debug};
use std::marker::PhantomData;
use std::string::ToString;
use std::{i32, i64};
use std::{u32, u64};

use bytes::Bytes;

use yarte_helpers::helpers::to_bytes;

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

macro_rules! json_str {
    ([]) => {
        "[]"
    };
    ([ $e0:tt $(, $e:tt)* $(,)? ]) => {
        concat!("[",
            json_str!($e0),
            $(",", json_str!($e),)*
        "]")
    };
    ({}) => {
        "{}"
    };
    ({ $k0:tt : $v0:tt $(, $k:tt : $v:tt)* $(,)? }) => {
        concat!("{",
            stringify!($k0), ":", json_str!($v0),
            $(",", stringify!($k), ":", json_str!($v),)*
        "}")
    };
    (($other:tt)) => {
        $other
    };
    ($other:tt) => {
        stringify!($other)
    };
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
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

fn test_encode_ok<T>(errors: &[(T, &str)])
where
    T: PartialEq + Debug + ser::Serialize,
{
    for &(ref value, out) in errors {
        let out = Bytes::from(out.to_string());

        let s = to_bytes(0, value).unwrap();
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
    let e = Bytes::from("null");
    let v = to_bytes(0, &::std::f64::NAN).unwrap();
    assert_eq!(v, e);

    let v = to_bytes(0, &::std::f64::INFINITY).unwrap();
    assert_eq!(v, e);

    let v = to_bytes(0, &::std::f32::NAN).unwrap();
    assert_eq!(v, e);

    let v = to_bytes(0, &::std::f32::INFINITY).unwrap();
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
fn test_write_list() {
    test_encode_ok(&[
        (vec![], "[]"),
        (vec![true], "[true]"),
        (vec![true, false], "[true,false]"),
    ]);

    test_encode_ok(&[
        (vec![vec![], vec![], vec![]], "[[],[],[]]"),
        (vec![vec![1, 2, 3], vec![], vec![]], "[[1,2,3],[],[]]"),
        (vec![vec![], vec![1, 2, 3], vec![]], "[[],[1,2,3],[]]"),
        (vec![vec![], vec![], vec![1, 2, 3]], "[[],[],[1,2,3]]"),
    ]);

    let long_test_list = json!([false, null, ["foo\nbar", 3.5]]);

    test_encode_ok(&[(long_test_list, json_str!([false, null, ["foo\nbar", 3.5]]))]);
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
fn test_write_option() {
    test_encode_ok(&[(None, "null"), (Some("jodhpurs"), "\"jodhpurs\"")]);

    test_encode_ok(&[
        (None, "null"),
        (Some(vec!["foo", "bar"]), "[\"foo\",\"bar\"]"),
    ]);
}

#[test]
fn test_write_newtype_struct() {
    #[derive(Serialize, PartialEq, Debug)]
    struct Newtype(BTreeMap<String, i32>);

    let inner = Newtype(treemap!(String::from("inner") => 123));
    let outer = treemap!(String::from("outer") => to_value(&inner).unwrap());

    test_encode_ok(&[(inner, r#"{"inner":123}"#)]);

    test_encode_ok(&[(outer, r#"{"outer":{"inner":123}}"#)]);
}

#[test]
fn test_serialize_seq_with_no_len() {
    #[derive(Clone, Debug, PartialEq)]
    struct MyVec<T>(Vec<T>);

    impl<T> ser::Serialize for MyVec<T>
    where
        T: ser::Serialize,
    {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ser::Serializer,
        {
            use serde::ser::SerializeSeq;
            let mut seq = serializer.serialize_seq(None)?;
            for elem in &self.0 {
                seq.serialize_element(elem)?;
            }
            seq.end()
        }
    }

    struct Visitor<T> {
        marker: PhantomData<MyVec<T>>,
    }

    impl<'de, T> de::Visitor<'de> for Visitor<T>
    where
        T: de::Deserialize<'de>,
    {
        type Value = MyVec<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("array")
        }

        #[inline]
        fn visit_unit<E>(self) -> Result<MyVec<T>, E>
        where
            E: de::Error,
        {
            Ok(MyVec(Vec::new()))
        }

        #[inline]
        fn visit_seq<V>(self, mut visitor: V) -> Result<MyVec<T>, V::Error>
        where
            V: de::SeqAccess<'de>,
        {
            let mut values = Vec::new();

            while let Some(value) = visitor.next_element()? {
                values.push(value);
            }

            Ok(MyVec(values))
        }
    }

    impl<'de, T> de::Deserialize<'de> for MyVec<T>
    where
        T: de::Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<MyVec<T>, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            deserializer.deserialize_map(Visitor {
                marker: PhantomData,
            })
        }
    }

    let mut vec = Vec::new();
    vec.push(MyVec(Vec::new()));
    vec.push(MyVec(Vec::new()));
    let vec: MyVec<MyVec<u32>> = MyVec(vec);

    test_encode_ok(&[(vec, "[[],[]]")]);
}

#[test]
fn test_serialize_map_with_no_len() {
    #[derive(Clone, Debug, PartialEq)]
    struct MyMap<K, V>(BTreeMap<K, V>);

    impl<K, V> ser::Serialize for MyMap<K, V>
    where
        K: ser::Serialize + Ord,
        V: ser::Serialize,
    {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ser::Serializer,
        {
            use serde::ser::SerializeMap;
            let mut map = serializer.serialize_map(None)?;
            for (k, v) in &self.0 {
                map.serialize_entry(k, v)?;
            }
            map.end()
        }
    }

    struct Visitor<K, V> {
        marker: PhantomData<MyMap<K, V>>,
    }

    impl<'de, K, V> de::Visitor<'de> for Visitor<K, V>
    where
        K: de::Deserialize<'de> + Eq + Ord,
        V: de::Deserialize<'de>,
    {
        type Value = MyMap<K, V>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("map")
        }

        #[inline]
        fn visit_unit<E>(self) -> Result<MyMap<K, V>, E>
        where
            E: de::Error,
        {
            Ok(MyMap(BTreeMap::new()))
        }

        #[inline]
        fn visit_map<Visitor>(self, mut visitor: Visitor) -> Result<MyMap<K, V>, Visitor::Error>
        where
            Visitor: de::MapAccess<'de>,
        {
            let mut values = BTreeMap::new();

            while let Some((key, value)) = visitor.next_entry()? {
                values.insert(key, value);
            }

            Ok(MyMap(values))
        }
    }

    impl<'de, K, V> de::Deserialize<'de> for MyMap<K, V>
    where
        K: de::Deserialize<'de> + Eq + Ord,
        V: de::Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<MyMap<K, V>, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            deserializer.deserialize_map(Visitor {
                marker: PhantomData,
            })
        }
    }

    let mut map = BTreeMap::new();
    map.insert("a", MyMap(BTreeMap::new()));
    map.insert("b", MyMap(BTreeMap::new()));
    let map: MyMap<_, MyMap<u32, u32>> = MyMap(map);

    test_encode_ok(&[(map.clone(), "{\"a\":{},\"b\":{}}")]);
}

#[test]
// TODO: error know in compilation time
fn test_serialize_rejects_bool_keys() {
    let map = treemap!(
        true => 2,
        false => 4
    );

    let err = to_bytes(0, &map).unwrap_err();
    assert_eq!(err.to_string(), "key must be a string");
}

#[test]
fn test_serialize_rejects_adt_keys() {
    let map = treemap!(
        Some("a") => 2,
        Some("b") => 4,
        None => 6
    );

    let err = to_bytes(0, &map).unwrap_err();
    assert_eq!(err.to_string(), "key must be a string");
}

#[test]
fn test_integer_key() {
    // map with integer keys
    let map = treemap!(
        1 => 2,
        -1 => 6
    );
    let j = r#"{"-1":6,"1":2}"#;
    test_encode_ok(&[(&map, j)]);
}

#[test]
fn test_effectively_string_keys() {
    #[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Serialize)]
    enum Enum {
        One,
        Two,
    }
    let map = treemap! {
        Enum::One => 1,
        Enum::Two => 2
    };
    let expected = r#"{"One":1,"Two":2}"#;
    test_encode_ok(&[(&map, expected)]);

    #[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Serialize)]
    struct Wrapper(String);
    let map = treemap! {
        Wrapper("zero".to_owned()) => 0,
        Wrapper("one".to_owned()) => 1
    };
    let expected = r#"{"one":1,"zero":0}"#;
    test_encode_ok(&[(&map, expected)]);
}
