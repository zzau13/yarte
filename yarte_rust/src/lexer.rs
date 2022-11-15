// Adapted from: https://github.com/dtolnay/proc-macro2/blob/master/src/parse.rs

use yarte_strnom::{Cursor, LexError, Span};

use crate::error::{CResult, Error, Result};
use crate::literals::literal;
use crate::sink::{Sink, State};
use crate::token_types::{Delimiter, Ident, Punct};

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

pub fn token_stream<'a, S: Sink<'a>>(mut input: Cursor<'a>, sink: &mut S) -> CResult<'a> {
    while !input.rest.is_empty() {
        input = skip_whitespaces(input);

        let first = match input.bytes().next() {
            Some(first) => first,
            None => {
                return match sink.end() {
                    Ok(_) => Ok(input),
                    Err(_) => Err(LexError::Next(Error::StartToken, Span::from(input))),
                }
            }
        };

        if let Some(open_delimiter) = match first {
            b'(' => Some(Delimiter::Parenthesis),
            b'[' => Some(Delimiter::Bracket),
            b'{' => Some(Delimiter::Brace),
            _ => None,
        } {
            input = input.adv(1);
            match sink.open_group(open_delimiter)? {
                State::Continue => (),
                State::Stop => return Ok(input),
            }
        } else if let Some(close_delimiter) = match first {
            b')' => Some(Delimiter::Parenthesis),
            b']' => Some(Delimiter::Bracket),
            b'}' => Some(Delimiter::Brace),
            _ => None,
        } {
            input = input.adv(1);
            match sink.close_group(close_delimiter)? {
                State::Continue => (),
                State::Stop => return Ok(input),
            }
        } else {
            input = leaf_token(input, sink)?;
        }
    }
    match sink.end() {
        Ok(_) => Ok(input),
        Err(_) => Err(LexError::Fail(Error::SinkEnd, Span::from(input))),
    }
}

fn leaf_token<'a, S: Sink<'a>>(input: Cursor<'a>, sink: &mut S) -> CResult<'a> {
    if let Ok((input, l)) = literal(input) {
        match sink.literal(l)? {
            State::Continue => (),
            State::Stop => return Ok(input),
        }
        Ok(input)
    } else if let Ok((input, p)) = punct(input) {
        match sink.punct(p)? {
            State::Continue => (),
            State::Stop => return Ok(input),
        }
        Ok(input)
    } else if let Ok((input, i)) = ident(input) {
        match sink.ident(i)? {
            State::Continue => (),
            State::Stop => return Ok(input),
        }
        Ok(input)
    } else {
        Err(LexError::Next(Error::UnmatchedToken, Span::from(input)))
    }
}

fn punct(input: Cursor) -> Result<Punct> {
    /* this can by one of these symbols: ~!@#$%^&*-=+|;:,<.>/?'  */
    if input.starts_with("//") || input.starts_with("/*") {
        // Do not accept `/` of a comment as a punct.
        return Err(LexError::Next(Error::Punct, Span::from(input)));
    }

    let mut chars = input.chars();
    let first = match chars.next() {
        Some(ch) => ch,
        None => {
            return Err(LexError::Next(Error::Punct, Span::from(input)));
        }
    };
    let recognized = "~!@#$%^&*-=+|;:,<.>/?'";
    if recognized.contains(first) {
        Ok((input.adv(first.len_utf8()), Punct { ch: first }))
    } else {
        Err(LexError::Next(Error::Punct, Span::from(input)))
    }
}

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
