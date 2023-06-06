use serde::{Deserialize, Deserializer};
use std::mem::transmute;

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

static RECOGNIZED: &str = "!#$%&'*+,-./:;<=>?@[]^_`{|}~";
// Symbol	Decimal
//   !	    33
//   #	    35
//   $	    36
//   %	    37
//   &	    38
//   '	    39
//   *	    42
//   +	    43
//   ,	    44
//   -	    45
//   .	    46
//   /	    47
//   :	    58
//   ;	    59
//   <	    60
//   =	    61
//   >	    62
//   ?	    63
//   @	    64
//   [	    91
//   ]	    93
//   ^	    94
//   _	    95
//   `	    96
//   {	    123
//   |	    124
//   }	    125
//   ~	    126
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
    _93 = 93,
    _94 = 94,
    _95 = 95,
    _96 = 96,
    _123 = 123,
    _124 = 124,
    _125 = 125,
    _126 = 126,
}

impl TryFrom<char> for Punct {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
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
pub struct Literal<'a> {
    pub inner: &'a str,
}
