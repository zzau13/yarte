use std::fmt::{self, Debug, Display, Formatter};
use std::fs::read_to_string;

use glob::glob;
use serde::Deserialize;

use std::error::Error;
use yarte_lexer::{handlebars, parse, Cursor, Ki, KiError, Kinder, PResult, SToken};

#[derive(Debug, Deserialize)]
struct Fixture<'a, Kind: Ki<'a>> {
    #[serde(borrow)]
    src: &'a str,
    #[serde(borrow)]
    exp: Vec<SToken<'a, Kind>>,
}

#[derive(Debug, Deserialize)]
struct FixturePanic<'a>(#[serde(borrow)] &'a str);

#[derive(Debug, Clone, PartialEq, Deserialize)]
enum MyKind<'a> {
    Some,
    Str(&'a str),
}

impl<'a> Kinder<'a> for MyKind<'a> {
    type Error = MyError;
    const OPEN: char = '{';
    const CLOSE: char = '}';
    const OPEN_EXPR: char = '{';
    const CLOSE_EXPR: char = '}';
    const OPEN_BLOCK: char = '{';
    const CLOSE_BLOCK: char = '}';
    const WS: char = '~';
    const WS_AFTER: bool = false;
    fn parse(i: Cursor<'a>) -> PResult<Self, Self::Error> {
        Ok((i, MyKind::Str(i.rest)))
    }
    fn comment(i: Cursor<'a>) -> PResult<&'a str, Self::Error> {
        handlebars::comment::<Self>(i)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum MyError {
    Some,
    Str(&'static str),
}

impl Display for MyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for MyError {}

impl KiError for MyError {
    const EMPTY: Self = MyError::Some;
    const PATH: Self = MyError::Some;
    const UNCOMPLETED: Self = MyError::Some;
    const WHITESPACE: Self = MyError::Some;
    const COMMENTARY: Self = MyError::Some;
    const CLOSE_BLOCK: Self = MyError::Some;

    fn tag(s: &'static str) -> Self {
        MyError::Str(s)
    }

    fn tac(_: char) -> Self {
        MyError::Some
    }
}

#[test]
fn test() {
    for entry in glob("./tests/fixtures/features/**/*.ron").expect("Failed to read glob pattern") {
        let name = entry.expect("File name");
        let src = read_to_string(name).expect("Valid file");
        let fixtures: Vec<Fixture<'_, MyKind>> = ron::from_str(&src)
            .map_err(|e| eprintln!("{:?}", e))
            .expect("Valid Fixtures");

        for Fixture { src, exp } in fixtures {
            let res = parse::<MyKind>(Cursor { rest: src, off: 0 }).expect("Valid parse");
            eprintln!("BASE {:?} \nEXPR {:?}", exp, res);
            assert_eq!(res, exp);
        }
    }
}

#[test]
fn test_panic() {
    for entry in glob("./tests/fixtures/panic/**/*.ron").expect("Failed to read glob pattern") {
        let name = entry.expect("File name");
        let src = read_to_string(name).expect("Valid file");
        let fixtures: Vec<FixturePanic> = ron::from_str(&src)
            .map_err(|e| eprintln!("{:?}", e))
            .expect("Valid Fixtures");

        for FixturePanic(src) in fixtures {
            assert!(parse::<MyKind>(Cursor { rest: src, off: 0 }).is_err());
        }
    }
}
