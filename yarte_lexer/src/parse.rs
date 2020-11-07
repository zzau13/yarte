use std::fmt::Debug;

use crate::error::{ErrorMessage, PError};
use crate::source_map::Span;
use crate::strnom::{is_ws, Cursor, LexError, PResult};
use crate::{Kinder, SNode};

pub trait Ki: Kinder + Debug + PartialEq + Clone {}
impl<T: Kinder + Debug + PartialEq + Clone> Ki for T {}

pub fn parse<K: Ki>(i: Cursor) -> Result<Vec<SNode<K>>, ErrorMessage<PError>> {
    let (c, res) = eat(i)?;
    if c.is_empty() {
        Ok(res)
    } else {
        Err(ErrorMessage {
            message: PError::Uncompleted,
            span: Span::from_len(c, 1),
        })
    }
}

fn eat<K: Ki>(i: Cursor) -> PResult<Vec<SNode<K>>> {
    Err(LexError::Fail(PError::Uncompleted, Span::from_len(i, 1)))
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
