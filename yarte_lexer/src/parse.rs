use std::fmt::Debug;
use std::marker::PhantomData;

use crate::error::{ErrorMessage, PError};
use crate::source_map::Span;
use crate::strnom::{is_ws, Cursor, LexError, PResult};
use crate::{Comment, Lexer, Options, SNode};

#[derive(Debug, Copy, Clone)]
pub struct Parser<Kind> {
    opt: Options,
    _kind: PhantomData<Kind>,
}

pub fn build_parser<Kind>(opt: Options) -> Parser<Kind> {
    Parser {
        opt,
        _kind: PhantomData::default(),
    }
}

impl<Kind> Parser<Kind>
where
    Kind: Lexer + Comment + Debug + PartialEq + Clone,
{
    pub fn parse(self, i: Cursor) -> Result<Vec<SNode<Kind>>, ErrorMessage<PError>> {
        let (c, res) = Self::eat(i, self.opt)?;
        if c.is_empty() {
            Ok(res)
        } else {
            Err(ErrorMessage {
                message: PError::Uncompleted,
                span: Span::from_len(c, 1),
            })
        }
    }

    fn eat(i: Cursor, _opt: Options) -> PResult<Vec<SNode<Kind>>> {
        Err(LexError::Fail(PError::Uncompleted, Span::from_len(i, 1)))
    }
}

/// TODO: Define chars in path
/// Eat path at partial
/// Next white space close path
fn path(i: Cursor) -> PResult<&str> {
    take_while!(i, |i| !is_ws(i)).and_then(|(c, s)| {
        if s.is_empty() {
            Err(LexError::Fail(PError::PartialPath, Span::from(c)))
        } else {
            Ok((c, s))
        }
    })
}

pub fn trim(i: &str) -> (&str, &str, &str) {
    if i.is_empty() {
        return ("", "", "");
    }

    let b = i.as_bytes();

    if let Some(ln) = b.iter().position(|x| !is_ws((*x).into())) {
        let rn = b.iter().rposition(|x| !is_ws((*x).into())).unwrap();
        (
            safe_utf8(&b[..ln]),
            safe_utf8(&b[ln..=rn]),
            safe_utf8(&b[rn + 1..]),
        )
    } else {
        (i, "", "")
    }
}

/// Convert from bytes to str
/// Use when previous check bytes it's valid utf8
fn safe_utf8(s: &[u8]) -> &str {
    unsafe { ::std::str::from_utf8_unchecked(s) }
}
