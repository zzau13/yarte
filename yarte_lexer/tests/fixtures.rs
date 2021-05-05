use std::fmt::{self, Debug, Display, Formatter};
use std::fs::read_to_string;
use std::result;

use glob::glob;
use serde::Deserialize;

use std::error::Error;
use yarte_lexer::error::{ErrorMessage, KiError, Result};
use yarte_lexer::pipes::{
    _false, _true, and_then, debug, important, is_empty, is_len, map, map_err, not, then,
};
use yarte_lexer::{
    _while, alt, ascii, asciis, do_parse, is_ws, path, pipes, tac, tag, ws, Ascii, Cursor, Ki,
    Kinder, LexError, LexResult, Lexer, SArm, SExpr, SLocal, SStr, SVExpr, Sink, Span, Ws, S,
};

// TODO: Visit trait
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum Token<'a, Kind>
where
    Kind: Kinder<'a>,
{
    Arm(Ws, SArm),
    ArmKind(Ws, Kind, SArm),
    Comment(#[serde(borrow)] &'a str),
    Safe(Ws, SExpr),
    Local(Ws, SLocal),
    Expr(Ws, SVExpr),
    ExprKind(Ws, Kind, SVExpr),
    Lit(
        #[serde(borrow)] &'a str,
        #[serde(borrow)] SStr<'a>,
        #[serde(borrow)] &'a str,
    ),
    Block(Ws, SVExpr),
    BlockKind(Ws, Kind, SVExpr),
    Error(SVExpr),
}

struct VecSink<'a, K: Ki<'a>>(Vec<S<Token<'a, K>>>);

impl<'a, K: Ki<'a>> Sink<'a, K> for VecSink<'a, K> {
    fn arm(&mut self, ws: Ws, arm: SArm, span: Span) -> LexResult<K::Error> {
        self.0.push(S(Token::Arm(ws, arm), span));
        Ok(())
    }

    fn arm_kind(&mut self, ws: Ws, kind: K, arm: SArm, span: Span) -> LexResult<K::Error> {
        self.0.push(S(Token::ArmKind(ws, kind, arm), span));
        Ok(())
    }

    fn block(&mut self, ws: Ws, expr: SVExpr, span: Span) -> LexResult<K::Error> {
        self.0.push(S(Token::Block(ws, expr), span));
        Ok(())
    }

    fn block_kind(&mut self, ws: Ws, kind: K, expr: SVExpr, span: Span) -> LexResult<K::Error> {
        self.0.push(S(Token::BlockKind(ws, kind, expr), span));
        Ok(())
    }

    fn comment(&mut self, src: &'a str, span: Span) -> LexResult<K::Error> {
        self.0.push(S(Token::Comment(src), span));
        Ok(())
    }

    fn error(&mut self, expr: SVExpr, span: Span) -> LexResult<K::Error> {
        self.0.push(S(Token::Error(expr), span));
        Ok(())
    }

    fn expr(&mut self, ws: Ws, expr: SVExpr, span: Span) -> LexResult<K::Error> {
        self.0.push(S(Token::Expr(ws, expr), span));
        Ok(())
    }

    fn expr_kind(&mut self, ws: Ws, kind: K, expr: SVExpr, span: Span) -> LexResult<K::Error> {
        self.0.push(S(Token::ExprKind(ws, kind, expr), span));
        Ok(())
    }

    fn lit(&mut self, left: &'a str, src: SStr<'a>, right: &'a str, span: Span) {
        self.0.push(S(Token::Lit(left, src, right), span));
    }

    fn local(&mut self, ws: Ws, local: SLocal, span: Span) -> LexResult<K::Error> {
        self.0.push(S(Token::Local(ws, local), span));
        Ok(())
    }

    fn safe(&mut self, ws: Ws, expr: SExpr, span: Span) -> LexResult<K::Error> {
        self.0.push(S(Token::Safe(ws, expr), span));
        Ok(())
    }

    fn end(&mut self) -> LexResult<K::Error> {
        Ok(())
    }
}

pub fn parse<'a, K: Ki<'a>>(
    i: Cursor<'a>,
) -> result::Result<Vec<S<Token<'a, K>>>, ErrorMessage<K::Error>> {
    Ok(Lexer::<K, VecSink<'a, K>>::new(VecSink(vec![])).feed(i)?.0)
}

#[derive(Debug, Deserialize)]
struct Fixture<'a, Kind: Ki<'a>> {
    #[serde(borrow)]
    src: &'a str,
    #[serde(borrow)]
    exp: Vec<S<Token<'a, Kind>>>,
}

#[derive(Debug, Deserialize)]
struct FixturePanic<'a>(#[serde(borrow)] &'a str);

fn comment<'a, K: Ki<'a>>(i: Cursor<'a>) -> Result<&'a str, K::Error> {
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
                    K::Error::UNCOMPLETED,
                    Span::from_cursor(i, c),
                ));
            }
        }
    }
}

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

    fn parse(i: Cursor<'a>) -> Result<Self, Self::Error> {
        const PARTIAL: Ascii = ascii!('>');

        let ws = |i| pipes!(i, ws: is_empty: _false);

        do_parse!(i,
            tac[PARTIAL]    =>
            ws              =>
            p= path         =>
            ws              =>
            (MyKindAfter::Partial(p))
        )
    }

    fn comment(i: Cursor<'a>) -> Result<&'a str, Self::Error> {
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

    fn parse(i: Cursor<'a>) -> Result<Self, Self::Error> {
        alt!(i, some | partial)
    }

    fn comment(i: Cursor<'a>) -> Result<&'a str, Self::Error> {
        comment::<Self>(i)
    }
}

fn partial(i: Cursor) -> Result<MyKind, MyError> {
    const PARTIAL: Ascii = ascii!('>');

    let ws = |i| pipes!(i, ws: _true);

    do_parse!(i,
        tac[PARTIAL]    =>
        ws              =>
        p= path         =>
        ws:important    =>
        (MyKind::Partial(p))
    )
}

fn some(i: Cursor) -> Result<MyKind, MyError> {
    const SOME: &[Ascii] = asciis!("some");

    let tag = |i| {
        pipes!(i,
            tag[SOME]:
            map_err::<_, MyError, _>[|_| MyError::Some]:
            then::<_, _, MyKind, _>[|_| Err(MyError::Some)]:
            and_then[|_| Ok(MyKind::Some)]
        )
    };
    let ws = |i| pipes!(i, ws:is_empty:_false:map[|_| MyKind::Some]);

    do_parse!(i, tag => ws => (MyKind::Some))
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
enum MyKindBlock<'a> {
    Partial(&'a str),
    Some,
    Str(&'a str),
}

impl<'a> Kinder<'a> for MyKindBlock<'a> {
    type Error = MyError;
    const OPEN: Ascii = ascii!('{');
    const CLOSE: Ascii = ascii!('}');
    const OPEN_EXPR: Ascii = ascii!('{');
    const CLOSE_EXPR: Ascii = ascii!('}');
    const OPEN_BLOCK: Ascii = ascii!('%');
    const CLOSE_BLOCK: Ascii = ascii!('%');
    const WS: Ascii = ascii!('~');
    const WS_AFTER: bool = true;

    fn parse(i: Cursor<'a>) -> Result<Self, Self::Error> {
        const PARTIAL: Ascii = ascii!('>');

        let ws_not_empty = |i| pipes!(i, ws: is_empty: _false);
        let ws_0 =
            |i| pipes!(i, _while[is_ws]:is_len[0]:debug["Len"]:_false:is_empty:_true:not:_false);

        // TODO: remove unnecessary []
        do_parse!(i,
            tac[PARTIAL]                    =>
            ws_not_empty                    =>
            p= path                         =>
            ws_not_empty[]:debug["after"]   =>
            ws_0[]:debug["ws_0"]            =>
            (MyKindBlock::Partial(p))
        )
    }

    fn comment(i: Cursor<'a>) -> Result<&'a str, Self::Error> {
        comment::<Self>(i)
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
    const UNCOMPLETED: Self = MyError::Some;
    const PATH: Self = MyError::Some;
    const WHITESPACE: Self = MyError::Some;

    fn str(s: &'static str) -> Self {
        MyError::Str(s)
    }

    fn char(_: char) -> Self {
        MyError::Some
    }

    fn string(s: String) -> Self {
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
    test_after_block_features: "./tests/fixtures/features/**/*.ron" MyKindBlock,
    test_after_block_features_a: "./tests/fixtures/features_a/**/*.ron" MyKindBlock,
    test_after_block: "./tests/fixtures/block/**/*.ron" MyKindBlock,
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
