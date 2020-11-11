use crate::parse::close_block;
use crate::source_map::Span;
use crate::strnom::{get_chars, tac};
use crate::{Cursor, Ki, KiError, LexError, PResult};

/// Eat comment
pub fn comment<'a, K: Ki<'a>>(i: Cursor<'a>) -> PResult<&'a str, K::Error> {
    let (c, _) = tac(i, '!')?;
    let (c, expected) = if c.starts_with("--") {
        (c.adv(2), "--")
    } else {
        (c, "")
    };

    let mut at = 0;
    loop {
        let next = c.adv(at);
        if next.is_empty() {
            break Err(LexError::Next(
                K::Error::COMMENTARY,
                Span::from_cursor(i, c),
            ));
        }

        match close_block::<'a, K>(next) {
            Ok((cur, _)) => {
                if let Some(pre) = at.checked_sub(expected.len()) {
                    let end = get_chars(c.rest, pre, at);
                    if end == expected {
                        return Ok((cur, get_chars(c.rest, 0, pre)));
                    }
                }

                at += 1
            }
            Err(_) => at += 1,
        }
    }
}
