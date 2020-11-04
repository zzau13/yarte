use std::fmt::Debug;
use std::path::Path;
use std::{fs::File, io, io::prelude::*};

use glob::glob;
use serde::Deserialize;

use yarte_lexer::{build_parser, Comment, Cursor, Lexer, Options, PResult, SNode};

fn read_file<P: AsRef<Path>>(path: P) -> Result<String, io::Error> {
    let mut f = File::open(path)?;
    let mut s = String::new();
    (f.read_to_string(&mut s))?;
    Ok(s.trim_end().to_string())
}

#[derive(Debug, Deserialize)]
struct Fixture<'a, Kind>
where
    Kind: PartialEq + Clone + Debug + Lexer + Comment,
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

impl Lexer for Kind {
    fn parse(_c: Cursor) -> PResult<'_, Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

impl Comment for Kind {
    fn open(_c: Cursor) -> PResult<'_, Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }

    fn close(_c: Cursor) -> PResult<'_, Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }

    fn inline(_c: Cursor) -> PResult<'_, Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

static OPT: Options = Options {
    open: '{',
    close: '}',
    ws: '~',
    ws_after: false,
};

#[test]
fn test() {
    for entry in glob("./tests/fixtures/features/**/*.ron").expect("Failed to read glob pattern") {
        let name = entry.expect("File name");
        let src = read_file(name).expect("Valid file");
        let fixtures: Vec<Fixture<'_, Kind>> = ron::from_str(&src)
            .map_err(|e| eprintln!("{:?}", e))
            .expect("Valid Fixtures");

        for Fixture { src, exp } in fixtures {
            let res = build_parser::<Kind>(OPT)
                .parse(Cursor { rest: src, off: 0 })
                .expect("Valid parse");
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
            assert!(build_parser::<Kind>(OPT)
                .parse(Cursor { rest: src, off: 0 })
                .is_err());
        }
    }
}
