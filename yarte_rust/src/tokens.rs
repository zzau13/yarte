use std::mem::transmute;
use std::simd::{u8x32, SimdPartialEq};

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
    Exclamation = b'!',
    Hash = b'#',
    Dollar = b'$',
    Percent = b'%',
    And = b'&',
    Apostrophe = b'\'',
    Asterisk = b'*',
    Plus = b'+',
    Comma = b',',
    Hyphen = b'-',
    Dot = b'.',
    Slash = b'/',
    Colon = b':',
    SemiColon = b';',
    GreaterThan = b'<',
    Equal = b'=',
    LessThan = b'>',
    Question = b'?',
    At = b'@',
    Circumflex = b'^',
    Underscore = b'_',
    Backtick = b'`',
    Bar = b'|',
    Tilde = b'~',
}

impl TryFrom<u8> for Punct {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        static RECOGNIZED: &[u8; 32] = b"!#$%&'*+,-./:;<=>?@^_`|~!#$%&'*+";
        let r = u8x32::from_slice(RECOGNIZED);
        let v = u8x32::from_array([value; 32]);

        if r.simd_eq(v).any() {
            // Safety: RECOGNIZED contains value is safe convert because have the same representation
            Ok(unsafe { transmute(value) })
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
            u8::try_from(x)
                .map_err(|_| ())
                .and_then(Punct::try_from)
                .map_err(|_| serde::de::Error::custom("No valid Punct"))
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
#[serde(transparent)]
pub struct Ident<'a> {
    pub inner: &'a str,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
pub enum LiteralKind {
    String,
    ByteString,
    Byte,
    Character,
    Float,
    Int,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
pub struct Literal<'a> {
    #[serde(borrow)]
    pub i: &'a str,
    pub k: LiteralKind,
}
