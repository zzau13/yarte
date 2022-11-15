// Adapted from: https://github.com/dtolnay/proc-macro2/blob/master/src/parse.rs
use crate::error::{CResult, Error, Result};
use crate::token_types::Literal;
use yarte_strnom::{do_parse, tac, tag, Cursor, LexError, Span};

pub(crate) fn literal(input: Cursor) -> Result<Literal> {
    let rest = literal_nocapture(input)?;
    let end = input.len() - rest.len();
    Ok((
        rest,
        Literal {
            inner: &input.rest[..end],
        },
    ))
}

fn literal_nocapture(input: Cursor) -> CResult {
    if let Ok(ok) = string(input) {
        Ok(ok)
    } else if let Ok(ok) = byte_string(input) {
        Ok(ok)
    } else if let Ok(ok) = byte(input) {
        Ok(ok)
    } else if let Ok(ok) = character(input) {
        Ok(ok)
    } else if let Ok(ok) = float(input) {
        Ok(ok)
    } else if let Ok(ok) = int(input) {
        Ok(ok)
    } else {
        Err(LexError::Next(Error::Literal, Span::from(input)))
    }
}

pub(crate) fn is_ident_start(c: char) -> bool {
    c == '_' || unicode_ident::is_xid_start(c)
}

pub(crate) fn is_ident_continue(c: char) -> bool {
    unicode_ident::is_xid_continue(c)
}

fn ident_not_raw(input: Cursor) -> Result<&str> {
    let mut chars = input.rest.char_indices();

    match chars.next() {
        Some((_, ch)) if is_ident_start(ch) => {}
        _ => return Err(LexError::Next(Error::Literal, Span::from(input))),
    }

    let mut end = input.len();
    for (i, ch) in chars {
        if !is_ident_continue(ch) {
            end = i;
            break;
        }
    }

    Ok((input.adv(end), &input.rest[..end]))
}

fn literal_suffix(input: Cursor) -> Cursor {
    match ident_not_raw(input) {
        Ok((input, _)) => input,
        Err(_) => input,
    }
}

fn string(input: Cursor) -> CResult {
    if let Ok((input, _)) = tac::<Error>(input, b'\"') {
        cooked_string(input)
    } else if let Ok((input, _)) = tac::<Error>(input, b'r') {
        raw_string(input)
    } else {
        Err(LexError::Next(Error::Literal, Span::from(input)))
    }
}

fn cooked_string(input: Cursor) -> CResult {
    let mut chars = input.rest.char_indices().peekable();

    while let Some((i, ch)) = chars.next() {
        match ch {
            '"' => {
                let input = input.adv(i + 1);
                return Ok(literal_suffix(input));
            }
            '\r' => match chars.next() {
                Some((_, '\n')) => {}
                _ => break,
            },
            '\\' => match chars.next() {
                Some((_, 'x')) => {
                    if !backslash_x_char(&mut chars) {
                        break;
                    }
                }
                Some((_, 'n')) | Some((_, 'r')) | Some((_, 't')) | Some((_, '\\'))
                | Some((_, '\'')) | Some((_, '"')) | Some((_, '0')) => {}
                Some((_, 'u')) => {
                    if !backslash_u(&mut chars) {
                        break;
                    }
                }
                Some((_, ch @ '\n')) | Some((_, ch @ '\r')) => {
                    let mut last = ch;
                    loop {
                        if last == '\r' && chars.next().map_or(true, |(_, ch)| ch != '\n') {
                            return Err(LexError::Next(Error::Literal, Span::from(input)));
                        }
                        match chars.peek() {
                            Some((_, ch)) if ch.is_whitespace() => {
                                last = *ch;
                                chars.next();
                            }
                            _ => break,
                        }
                    }
                }
                _ => break,
            },
            _ch => {}
        }
    }
    Err(LexError::Next(Error::Literal, Span::from(input)))
}

fn byte_string(input: Cursor) -> CResult {
    if let Ok((input, _)) = tac::<Error>(input, b'"') {
        cooked_byte_string(input)
    } else if let Ok((input, _)) = tag::<Error>(input, "br") {
        raw_string(input)
    } else {
        Err(LexError::Next(Error::Literal, Span::from(input)))
    }
}

fn cooked_byte_string(mut input: Cursor) -> CResult {
    let mut bytes = input.bytes().enumerate();
    while let Some((offset, b)) = bytes.next() {
        match b {
            b'"' => {
                let input = input.adv(offset + 1);
                return Ok(literal_suffix(input));
            }
            b'\r' => match bytes.next() {
                Some((_, b'\n')) => {}
                _ => break,
            },
            b'\\' => match bytes.next() {
                Some((_, b'x')) => {
                    if !backslash_x_byte(&mut bytes) {
                        break;
                    }
                }
                Some((_, b'n')) | Some((_, b'r')) | Some((_, b't')) | Some((_, b'\\'))
                | Some((_, b'0')) | Some((_, b'\'')) | Some((_, b'"')) => {}
                Some((newline, b @ b'\n')) | Some((newline, b @ b'\r')) => {
                    let mut last = b as char;
                    let rest = input.adv(newline + 1);
                    let mut chars = rest.rest.char_indices();
                    loop {
                        if last == '\r' && chars.next().map_or(true, |(_, ch)| ch != '\n') {
                            return Err(LexError::Next(Error::Literal, Span::from(input)));
                        }
                        match chars.next() {
                            Some((_, ch)) if ch.is_whitespace() => last = ch,
                            Some((offset, _)) => {
                                input = rest.adv(offset);
                                bytes = input.bytes().enumerate();
                                break;
                            }
                            None => return Err(LexError::Next(Error::Literal, Span::from(input))),
                        }
                    }
                }
                _ => break,
            },
            b if b < 0x80 => {}
            _ => break,
        }
    }
    Err(LexError::Next(Error::Literal, Span::from(input)))
}

fn raw_string(input: Cursor) -> CResult {
    let mut chars = input.rest.char_indices();
    let mut n = 0;
    for (i, ch) in &mut chars {
        match ch {
            '"' => {
                n = i;
                break;
            }
            '#' => {}
            _ => return Err(LexError::Next(Error::Literal, Span::from(input))),
        }
    }
    while let Some((i, ch)) = chars.next() {
        match ch {
            '"' if input.rest[i + 1..].starts_with(&input.rest[..n]) => {
                let rest = input.adv(i + 1 + n);
                return Ok(literal_suffix(rest));
            }
            '\r' => match chars.next() {
                Some((_, '\n')) => {}
                _ => break,
            },
            _ => {}
        }
    }
    Err(LexError::Next(Error::Literal, Span::from(input)))
}

fn byte(input: Cursor) -> CResult {
    let input = tag(input, "b'")?.0;
    let mut bytes = input.bytes().enumerate();
    let ok = match bytes.next().map(|(_, b)| b) {
        Some(b'\\') => match bytes.next().map(|(_, b)| b) {
            Some(b'x') => backslash_x_byte(&mut bytes),
            Some(b'n') | Some(b'r') | Some(b't') | Some(b'\\') | Some(b'0') | Some(b'\'')
            | Some(b'"') => true,
            _ => false,
        },
        b => b.is_some(),
    };
    if !ok {
        return Err(LexError::Next(Error::Literal, Span::from(input)));
    }
    let (offset, _) = bytes
        .next()
        .ok_or(LexError::Next(Error::Literal, Span::from(input)))?;
    if !input.chars().as_str().is_char_boundary(offset) {
        return Err(LexError::Next(Error::Literal, Span::from(input)));
    }
    let input = do_parse!(input.adv(offset), tac[b'\''] => ())?.0;
    Ok(literal_suffix(input))
}

fn character(input: Cursor) -> CResult {
    let input = do_parse!(input, tac[b'\''] => ())?.0;
    let mut chars = input.rest.char_indices();
    let ok = match chars.next().map(|(_, ch)| ch) {
        Some('\\') => match chars.next().map(|(_, ch)| ch) {
            Some('x') => backslash_x_char(&mut chars),
            Some('u') => backslash_u(&mut chars),
            Some('n') | Some('r') | Some('t') | Some('\\') | Some('0') | Some('\'') | Some('"') => {
                true
            }
            _ => false,
        },
        ch => ch.is_some(),
    };
    if !ok {
        return Err(LexError::Next(Error::Literal, Span::from(input)));
    }
    let (idx, _) = chars
        .next()
        .ok_or(LexError::Next(Error::Literal, Span::from(input)))?;
    let input = do_parse!(input.adv(idx), tac[b'\''] => ())?.0;
    Ok(literal_suffix(input))
}

macro_rules! next_ch {
    ($chars:ident @ $pat:pat_param $(| $rest:pat)*) => {
        match $chars.next() {
            Some((_, ch)) => match ch {
                $pat $(| $rest)* => ch,
                _ => return false,
            },
            None => return false,
        }
    };
}

fn backslash_x_char<I>(chars: &mut I) -> bool
where
    I: Iterator<Item = (usize, char)>,
{
    next_ch!(chars @ '0'..='7');
    next_ch!(chars @ '0'..='9' | 'a'..='f' | 'A'..='F');
    true
}

fn backslash_x_byte<I>(chars: &mut I) -> bool
where
    I: Iterator<Item = (usize, u8)>,
{
    next_ch!(chars @ b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F');
    next_ch!(chars @ b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F');
    true
}

fn backslash_u<I>(chars: &mut I) -> bool
where
    I: Iterator<Item = (usize, char)>,
{
    next_ch!(chars @ '{');
    let mut value = 0;
    let mut len = 0;
    for (_, ch) in chars {
        let digit = match ch {
            '0'..='9' => ch as u8 - b'0',
            'a'..='f' => 10 + ch as u8 - b'a',
            'A'..='F' => 10 + ch as u8 - b'A',
            '_' if len > 0 => continue,
            '}' if len > 0 => return char::from_u32(value).is_some(),
            _ => return false,
        };
        if len == 6 {
            return false;
        }
        value *= 0x10;
        value += u32::from(digit);
        len += 1;
    }
    false
}

fn word_break(input: Cursor) -> CResult {
    match input.chars().next() {
        Some(ch) if is_ident_continue(ch) => Err(LexError::Next(Error::Literal, Span::from(input))),
        Some(_) | None => Ok(input),
    }
}

fn float(input: Cursor) -> CResult {
    let mut rest = float_digits(input)?;
    if let Some(ch) = rest.chars().next() {
        if is_ident_start(ch) {
            rest = ident_not_raw(rest)?.0;
        }
    }
    word_break(rest)
}

fn float_digits(input: Cursor) -> CResult {
    let mut chars = input.chars().peekable();
    match chars.next() {
        Some(ch) if ('0'..='9').contains(&ch) => {}
        _ => return Err(LexError::Next(Error::Literal, Span::from(input))),
    }

    let mut len = 1;
    let mut has_dot = false;
    let mut has_exp = false;
    while let Some(&ch) = chars.peek() {
        match ch {
            '0'..='9' | '_' => {
                chars.next();
                len += 1;
            }
            '.' => {
                if has_dot {
                    break;
                }
                chars.next();
                if chars
                    .peek()
                    .map_or(false, |&ch| ch == '.' || is_ident_start(ch))
                {
                    return Err(LexError::Next(Error::Literal, Span::from(input)));
                }
                len += 1;
                has_dot = true;
            }
            'e' | 'E' => {
                chars.next();
                len += 1;
                has_exp = true;
                break;
            }
            _ => break,
        }
    }

    if !(has_dot || has_exp) {
        return Err(LexError::Next(Error::Literal, Span::from(input)));
    }

    if has_exp {
        let token_before_exp = if has_dot {
            Ok(input.adv(len - 1))
        } else {
            Err(LexError::Next(Error::Literal, Span::from(input)))
        };
        let mut has_sign = false;
        let mut has_exp_value = false;
        while let Some(&ch) = chars.peek() {
            match ch {
                '+' | '-' => {
                    if has_exp_value {
                        break;
                    }
                    if has_sign {
                        return token_before_exp;
                    }
                    chars.next();
                    len += 1;
                    has_sign = true;
                }
                '0'..='9' => {
                    chars.next();
                    len += 1;
                    has_exp_value = true;
                }
                '_' => {
                    chars.next();
                    len += 1;
                }
                _ => break,
            }
        }
        if !has_exp_value {
            return token_before_exp;
        }
    }

    Ok(input.adv(len))
}

fn int(input: Cursor) -> CResult {
    let mut rest = digits(input)?;
    if let Some(ch) = rest.chars().next() {
        if is_ident_start(ch) {
            rest = ident_not_raw(rest)?.0;
        }
    }
    word_break(rest)
}

fn digits(mut input: Cursor) -> CResult {
    let base = if input.starts_with("0x") {
        input = input.adv(2);
        16
    } else if input.starts_with("0o") {
        input = input.adv(2);
        8
    } else if input.starts_with("0b") {
        input = input.adv(2);
        2
    } else {
        10
    };

    let mut len = 0;
    let mut empty = true;
    for b in input.bytes() {
        match b {
            b'0'..=b'9' => {
                let digit = (b - b'0') as u64;
                if digit >= base {
                    return Err(LexError::Next(Error::Literal, Span::from(input)));
                }
            }
            b'a'..=b'f' => {
                let digit = 10 + (b - b'a') as u64;
                if digit >= base {
                    break;
                }
            }
            b'A'..=b'F' => {
                let digit = 10 + (b - b'A') as u64;
                if digit >= base {
                    break;
                }
            }
            b'_' => {
                if empty && base == 10 {
                    return Err(LexError::Next(Error::Literal, Span::from(input)));
                }
                len += 1;
                continue;
            }
            _ => break,
        };
        len += 1;
        empty = false;
    }
    if empty {
        Err(LexError::Next(Error::Literal, Span::from(input)))
    } else {
        Ok(input.adv(len))
    }
}
