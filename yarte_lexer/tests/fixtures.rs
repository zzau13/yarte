use std::fmt::{self, Debug, Display, Formatter};
use std::path::Path;
use std::{fs::File, io, io::prelude::*};

use glob::glob;
use serde::Deserialize;

use std::error::Error;
use yarte_lexer::{parse, Cursor, KiError, Kinder, SNode};

fn read_file<P: AsRef<Path>>(path: P) -> Result<String, io::Error> {
    let mut f = File::open(path)?;
    let mut s = String::new();
    (f.read_to_string(&mut s))?;
    Ok(s.trim_end().to_string())
}

#[derive(Debug, Deserialize)]
struct Fixture<'a, Kind>
where
    Kind: PartialEq + Clone + Debug + Kinder,
{
    #[serde(borrow)]
    src: &'a str,
    #[serde(borrow)]
    exp: Vec<SNode<'a, Kind>>,
}

#[derive(Debug, Deserialize)]
struct FixturePanic<'a>(#[serde(borrow)] &'a str);

#[derive(Debug, Clone, PartialEq, Deserialize)]
enum Kind {
    Some,
}

impl Kinder for Kind {
    type Error = MyError;
    const OPEN: char = '{';
    const CLOSE: char = '}';
    const OPEN_EXPR: char = '{';
    const CLOSE_EXPR: char = '}';
    const OPEN_BLOCK: char = '{';
    const CLOSE_BLOCK: char = '}';
    const WS: char = '~';
    const WS_AFTER: bool = false;
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
        let src = read_file(name).expect("Valid file");
        let fixtures: Vec<Fixture<'_, Kind>> = ron::from_str(&src)
            .map_err(|e| eprintln!("{:?}", e))
            .expect("Valid Fixtures");

        for Fixture { src, exp } in fixtures {
            let res = parse::<Kind>(Cursor { rest: src, off: 0 }).expect("Valid parse");
            assert_eq!(res, exp);
        }
    }
}

#[test]
fn test_panic() {
    for entry in glob("./tests/fixtures/panic/**/*.ron").expect("Failed to read glob pattern") {
        let name = entry.expect("File name");
        let src = read_file(name).expect("Valid file");
        let fixtures: Vec<FixturePanic> = ron::from_str(&src)
            .map_err(|e| eprintln!("{:?}", e))
            .expect("Valid Fixtures");

        for FixturePanic(src) in fixtures {
            assert!(parse::<Kind>(Cursor { rest: src, off: 0 }).is_err());
        }
    }
}
