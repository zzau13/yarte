// Adapted from: https://github.com/dtolnay/proc-macro2/blob/master/src/parse.rs
use yarte_strnom::source_map::{spanned, S};
use yarte_strnom::{Cursor, LexError, Span};

use crate::error::{CResult, Error, Result};
use crate::literals::literal;
use crate::sink::{Sink, State};
use crate::tokens::{Delimiter, Ident, Punct};

#[inline]
pub fn token_stream<'a, O: Sink<'a>>(mut input: Cursor<'a>, sink: &mut O) -> CResult<'a> {
    macro_rules! go {
        ($sink:expr => $next:expr) => {
            match $sink {
                State::Continue => input = $next,
                State::Stop => return Ok($next),
                State::Back => return Ok(input),
            };
        };
    }

    loop {
        input = skip_whitespaces(input);

        let first = match input.bytes().next() {
            Some(first) => first,
            None => break,
        };

        if let Some(open_delimiter) = match first {
            b'(' => Some(Delimiter::Parenthesis),
            b'[' => Some(Delimiter::Bracket),
            b'{' => Some(Delimiter::Brace),
            _ => None,
        } {
            let span = Span::from(input);
            go!(sink.open_group(S(open_delimiter, span))? => input.adv(1));
        } else if let Some(close_delimiter) = match first {
            b')' => Some(Delimiter::Parenthesis),
            b']' => Some(Delimiter::Bracket),
            b'}' => Some(Delimiter::Brace),
            _ => None,
        } {
            let span = Span::from(input);
            go!(sink.close_group(S(close_delimiter, span))? => input.adv(1));
        } else if let Ok((next, l)) = spanned(input, literal) {
            go!(sink.literal(l)? => next);
        } else if let Ok((next, p)) = spanned(input, punct) {
            go!(sink.punct(p)? => next);
        } else if let Ok((next, i)) = spanned(input, ident) {
            go!(sink.ident(i)? => next);
        } else {
            return Err(LexError::Next(Error::UnmatchedToken, Span::from(input)));
        }
    }
    match sink.end() {
        Ok(_) => Ok(input),
        // TODO:
        Err(_) => Err(LexError::Fail(Error::SinkEnd, Span::from(input))),
    }
}

#[inline]
fn punct(input: Cursor) -> Result<Punct> {
    /* this can by one of these symbols: ~!@#$%^&*-=+|;:,<.>/?'  */
    if input.starts_with("//") || input.starts_with("/*") {
        // Do not accept `/` of a comment as a punct.
        return Err(LexError::Next(Error::Punct, Span::from(input)));
    }

    Ok((
        input.adv(1),
        match input.bytes().next() {
            Some(ch) => ch,
            None => {
                return Err(LexError::Next(Error::Punct, Span::from(input)));
            }
        }
        .try_into()
        .map_err(|_| LexError::Next(Error::Punct, Span::from(input)))?,
    ))
}

#[inline]
fn ident(mut input: Cursor) -> Result<Ident> {
    /* This is a name: true/false, function, reserved word, variable, struct, trait, enum, etc*/
    if ["r\"", "r#\"", "r##", "b\"", "b\'", "br\"", "br#"]
        .iter()
        .any(|prefix| input.starts_with(prefix))
    {
        Err(LexError::Next(Error::Ident, Span::from(input)))
    } else {
        let mut cont = 0;
        for byte in input.bytes() {
            match byte {
                b if (0x30..0x4A).contains(&b) => {
                    // Number
                    if cont == 0 {
                        return Err(LexError::Next(Error::Ident, Span::from(input)));
                    }
                    cont += 1;
                }
                b if (0x41..0x5B).contains(&b) => cont += 1, // Caps
                b if (0x61..0x7B).contains(&b) => cont += 1, // Uncap
                b'_' => cont += 1,
                _ => {
                    let i = Ident {
                        inner: &input.rest[..cont],
                    };
                    input = input.adv(cont);
                    return Ok((input, i));
                }
            }
        }
        let i = Ident {
            inner: &input.rest[..cont],
        };
        input = input.adv(cont); // Corresponds to end of string check if cont+1 makes sense
        Ok((input, i))
    }
}

#[inline]
fn skip_whitespaces(mut s: Cursor) -> Cursor {
    while !s.is_empty() {
        let ch = s.chars().next().unwrap();
        if ch.is_whitespace() || ch == '\u{200e}' || ch == '\u{200f}' {
            s = s.adv(ch.len_utf8());
            continue;
        }
        return s;
    }
    s
}
