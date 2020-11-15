use std::fmt::{self, Debug, Display, Formatter};
use std::fs::read_to_string;

use glob::glob;
use serde::Deserialize;

use std::error::Error;
use yarte_lexer::{
    ascii, asciis, do_parse, is_ws, parse, tac, take_while, ws, Ascii, Cursor, Ki, KiError, Kinder,
    LexError, PResult, SToken, Span,
};

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
    Partial(&'a str),
    Some,
    Str(&'a str),
}

impl<'a> Kinder<'a> for MyKind<'a> {
    type Error = MyError;
    const OPEN: Ascii = ascii!('{');
    const CLOSE: Ascii = ascii!('}');
    const OPEN_EXPR: Ascii = ascii!('{');
    const CLOSE_EXPR: Ascii = ascii!('}');
    const OPEN_BLOCK: Ascii = ascii!('{');
    const CLOSE_BLOCK: Ascii = ascii!('}');
    const WS: Ascii = ascii!('~');
    const WS_AFTER: bool = false;

    fn parse(i: Cursor<'a>) -> PResult<Self, Self::Error> {
        const PARTIAL: Ascii = ascii!('>');
        let (i, partial) = do_parse!(i,
            tac(PARTIAL) >>
            ws() >>
            p: take_while(|x| !is_ws(x)) >>
            (p)
        )?;
        Ok((i, MyKind::Partial(partial)))
    }

    fn comment(i: Cursor<'a>) -> PResult<&'a str, Self::Error> {
        const E: Ascii = ascii!('!');
        const B: &[Ascii] = asciis!("--");
        const END_B: &[Ascii] = asciis!("--}}");
        const END_A: &[Ascii] = asciis!("}}");

        let (c, _) = tac(i, E)?;
        let (c, expected) = if c.starts_with(B) {
            (c.adv_ascii(B), END_B)
        } else {
            (c, END_A)
        };

        let mut at = 0;
        loop {
            if c.adv_starts_with(at, expected) {
                break Ok((c.adv(at + expected.len()), &c.rest[..at]));
            } else {
                at += 1;
                if at >= c.len() {
                    break Err(LexError::Next(MyError::Some, Span::from_cursor(i, c)));
                }
            }
        }
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
    const EXPR: Self = MyError::Some;

    fn tag(s: &'static str) -> Self {
        MyError::Str(s)
    }

    fn tac(_: char) -> Self {
        MyError::Some
    }
}

#[test]
fn test() {
    const _A: Ascii = ascii!('-');
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
