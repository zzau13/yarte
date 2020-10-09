use std::path::Path;
use std::{fs::File, io, io::prelude::*};

use glob::glob;
use serde::Deserialize;
use yarte_parser::{parse, Cursor, SNode};

fn read_file<P: AsRef<Path>>(path: P) -> Result<String, io::Error> {
    let mut f = File::open(path)?;
    let mut s = String::new();
    (f.read_to_string(&mut s))?;
    Ok(s.trim_end().to_string())
}

fn read_fixture(src: &str) -> Option<Vec<Fixture>> {
    ron::from_str(src).map_err(|e| eprintln!("{:?}", e)).ok()
}

#[derive(Debug, Deserialize)]
struct Fixture<'a> {
    #[serde(borrow)]
    src: &'a str,
    #[serde(borrow)]
    exp: Vec<SNode<'a>>,
}

#[test]
fn test_fixtures() {
    for entry in glob("./tests/fixtures/**/*.ron").expect("Failed to read glob pattern") {
        let name = entry.expect("File name");
        let file = read_file(name).expect("Valid file");
        let fixtures = read_fixture(&file).expect("Valid Fixtures");

        for Fixture { src, exp } in fixtures {
            let res = parse(Cursor { rest: src, off: 0 }).expect("Valid parse");
            assert_eq!(res, exp);
        }
    }
}
