use crate::source_map::Span;
use crate::strnom::{get_chars, tac};
use crate::{Cursor, KiError, LexError, PResult};

/// Eat comment
pub fn comment<E: KiError>(i: Cursor) -> PResult<&str, E> {
    let (j, _) = tac(i, '!')?;
    let (c, expected) = if j.starts_with("--") {
        (j.adv(2), "--!}}")
    } else {
        (j, "!}}")
    };

    let ch = expected.chars().next().unwrap();
    let rest = &expected[1..];
    let mut at = 0;
    loop {
        if let Some(j) = c.adv_find(at, ch) {
            if c.adv_starts_with(at + j + 1, rest) {
                break Ok((c.adv(at + j + expected.len()), get_chars(c.rest, 0, at + j)));
            } else {
                at += j + 1;
            }
        } else {
            break Err(LexError::Next(E::COMMENTARY, Span::from_cursor(i, c)));
        }
    }
}
