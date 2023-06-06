use serde::Deserialize;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
pub enum Delimiter {
    Parenthesis, // ()
    Brace,       // {}
    Bracket,     // []
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
#[serde(transparent)]
pub struct Ident<'a> {
    pub inner: &'a str,
}

pub static RECOGNIZED: &str = "!#$%&'*+,-./:;<=>?@[]^_`{|}~";

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
pub enum Punct {
    _33 = 33,
    _35 = 35,
    _36 = 36,
    _37 = 37,
    _38 = 38,
    _39 = 39,
    _42 = 42,
    _43 = 43,
    _44 = 44,
    _45 = 45,
    _46 = 46,
    _47 = 47,
    _58 = 58,
    _59 = 59,
    _60 = 60,
    _61 = 61,
    _62 = 62,
    _63 = 63,
    _64 = 64,
    _91 = 91,
    _92 = 92,
    _93 = 93,
    _94 = 94,
    _95 = 95,
    _96 = 96,
    _123 = 123,
    _124 = 124,
    _125 = 125,
    _126 = 126,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
#[serde(transparent)]
pub struct Literal<'a> {
    pub inner: &'a str,
}
