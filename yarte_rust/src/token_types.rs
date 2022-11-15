use serde::Deserialize;

#[derive(Clone, PartialEq, Eq, Debug, Deserialize)]
pub enum Delimiter {
    Parenthesis, // ()
    Brace,       // {}
    Bracket,     // []
    None,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize)]
pub struct Ident<'a> {
    pub inner: &'a str,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize)]
pub struct Punct {
    pub ch: char,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize)]
pub struct Literal<'a> {
    pub inner: &'a str,
}
