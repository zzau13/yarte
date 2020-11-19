use std::fmt::{self, Debug, Display, Formatter};
use std::fs::read_to_string;

use glob::glob;
use serde::Deserialize;

use std::error::Error;
use yarte_lexer::{
    ascii, asciis, do_parse, is_empty, is_ws, not_true, parse, tac, take_while, ws, Ascii, Cursor,
    Ki, KiError, Kinder, LexError, PResult, SToken, Span,
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
enum MyKindAfter<'a> {
    Partial(&'a str),
    Some,
    Str(&'a str),
}

impl<'a> Kinder<'a> for MyKindAfter<'a> {
    type Error = MyError;
    const OPEN: Ascii = ascii!('{');
    const CLOSE: Ascii = ascii!('}');
    const OPEN_EXPR: Ascii = ascii!('{');
    const CLOSE_EXPR: Ascii = ascii!('}');
    const OPEN_BLOCK: Ascii = ascii!('{');
    const CLOSE_BLOCK: Ascii = ascii!('}');
    const WS: Ascii = ascii!('~');
    const WS_AFTER: bool = true;

    fn parse(i: Cursor<'a>) -> PResult<Self, Self::Error> {
        const PARTIAL: Ascii = ascii!('>');
        let (i, partial) = do_parse!(i,
            tac[PARTIAL]                    =>
            ws:is_empty:not_true            =>
            p= take_while[|x| !is_ws(x)]    =>
            (p)
        )?;
        Ok((i, MyKindAfter::Partial(partial)))
    }

    fn comment(i: Cursor<'a>) -> PResult<&'a str, Self::Error> {
        comment::<Self>(i)
    }
}

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
            tac[PARTIAL]                    =>
            ws:is_empty:not_true            =>
            p= take_while[|x| !is_ws(x)]    =>
            (p)
        )?;
        Ok((i, MyKind::Partial(partial)))
    }

    fn comment(i: Cursor<'a>) -> PResult<&'a str, Self::Error> {
        comment::<Self>(i)
    }
}

fn comment<'a, K: Ki<'a>>(i: Cursor<'a>) -> PResult<&'a str, K::Error> {
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
                break Err(LexError::Next(
                    K::Error::COMMENTARY,
                    Span::from_cursor(i, c),
                ));
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum MyError {
    Some,
    Str(&'static str),
    StrO(String),
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

    fn expr(s: String) -> Self {
        MyError::StrO(s)
    }
}

macro_rules! features {
    ($name:ident: $path:literal $kind:ty) => {
        #[test]
        fn $name() {
            for entry in glob($path).expect("Failed to read glob pattern") {
                let name = entry.expect("File name");
                eprintln!("\n{:?}\n", name);
                let src = read_to_string(name).expect("Valid file");
                let fixtures: Vec<Fixture<'_, $kind>> =
                    ron::from_str(&src).expect("Valid Fixtures");

                for (i, Fixture { src, exp }) in fixtures.into_iter().enumerate() {
                    let res = parse::<$kind>(unsafe { Cursor::new(src, 0) }).expect("Valid parse");
                    eprintln!("{:2}:\nBASE {:?} \nEXPR {:?}", i, exp, res);
                    assert_eq!(res, exp);
                }
            }
        }
    };
    ($name:ident: $path:literal $kind:ty, $($t:tt)*) => {
        features!($name: $path $kind);
        features!($($t)*);
    };
    () => {}
}

features!(
    test_after_same_features: "./tests/fixtures/features/**/*.ron" MyKindAfter,
    test_after_same_features_a: "./tests/fixtures/features_a/**/*.ron" MyKindAfter,
    test_same_features: "./tests/fixtures/features/**/*.ron" MyKind,
    test_same_features_b: "./tests/fixtures/features_b/**/*.ron" MyKind,
);

#[test]
fn test_panic() {
    for entry in glob("./tests/fixtures/panic/**/*.ron").expect("Failed to read glob pattern") {
        let name = entry.expect("File name");
        let src = read_to_string(name).expect("Valid file");
        let fixtures: Vec<FixturePanic> = ron::from_str(&src)
            .map_err(|e| eprintln!("{:?}", e))
            .expect("Valid Fixtures");

        for FixturePanic(src) in fixtures {
            assert!(parse::<MyKind>(unsafe { Cursor::new(src, 0) }).is_err());
        }
    }
}
