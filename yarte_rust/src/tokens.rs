use std::mem::transmute;

use serde::{Deserialize, Deserializer};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
pub enum Delimiter {
    Parenthesis, // ()
    Brace,       // {}
    Bracket,     // []
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Punct {
    Exclamation = 33, // !
    Hash = 35,        // #
    Dollar = 36,      // $
    Percent = 37,     // %
    And = 38,         // &
    Apostrophe = 39,  // '
    Asterisk = 42,    // *
    Plus = 43,        // +
    Comma = 44,       // ,
    Hyphen = 45,      // -
    Dot = 46,         // .
    Slash = 47,       // /
    Colon = 58,       // :
    SemiColon = 59,   // ;
    GreaterThan = 60, // <
    Equal = 61,       // =
    LessThan = 62,    // >
    Question = 63,    // ?
    At = 64,          // @
    Circumflex = 94,  // ^
    Underscore = 95,  // _
    Backtick = 96,    // `
    Bar = 124,        // |
    Tilde = 126,      // ~
}

impl TryFrom<char> for Punct {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        static RECOGNIZED: &str = "!#$%&'*+,-./:;<=>?@^_`|~";

        if RECOGNIZED.contains(value) {
            // Safety: RECOGNIZED contains value is safe convert because have the same representation
            Ok(unsafe { transmute(value as u8) })
        } else {
            Err(())
        }
    }
}

impl<'de> Deserialize<'de> for Punct {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <char>::deserialize(deserializer).and_then(|x| {
            Punct::try_from(x).map_err(|_| serde::de::Error::custom("No valid Punct"))
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
#[serde(transparent)]
pub struct Ident<'a> {
    pub inner: &'a str,
}

// TODO:
#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
#[serde(transparent)]
pub struct Literal<'a> {
    pub inner: &'a str,
}
