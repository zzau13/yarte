use serde::Deserialize;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
pub enum Delimiter {
    Parenthesis, // ()
    Brace,       // {}
    Bracket,     // []
    None,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
#[serde(transparent)]
pub struct Ident<'a> {
    pub inner: &'a str,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
#[serde(transparent)]
pub struct Punct {
    pub ch: char,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
#[serde(transparent)]
pub struct Literal<'a> {
    pub inner: &'a str,
}
