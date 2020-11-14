//! Adapted from [`proc-macro2`](https://github.com/alexcrichton/proc-macro2).

use std::iter::once;
use std::str::Chars;

use crate::error::{KiError, LexError, PResult};
use crate::source_map::Span;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Cursor<'a> {
    pub rest: &'a str,
    pub off: u32,
}

// TODO: this do a multiple chars counts can improve changing
impl<'a> Cursor<'a> {
    #[inline]
    pub fn adv(&self, amt: usize) -> Cursor<'a> {
        let (front, rest) = self.rest.split_at(amt);
        Cursor {
            rest,
            off: self.off + front.chars().count() as u32,
        }
    }

    #[inline]
    pub fn adv_ascii(&self, s: &[Ascii]) -> Cursor<'a> {
        Cursor {
            rest: &self.rest[s.len()..],
            off: self.off + s.len() as u32,
        }
    }

    #[inline]
    pub fn find(&self, p: Ascii) -> Option<usize> {
        find_ascii(self.as_bytes(), p)
    }

    #[inline]
    pub fn adv_find(&self, amt: usize, p: Ascii) -> Option<usize> {
        find_ascii(&self.as_bytes()[amt..], p)
    }

    #[inline]
    pub fn adv_starts_with(&self, amt: usize, s: &[Ascii]) -> bool {
        start_with_ascii(&self.as_bytes()[amt..], s)
    }

    #[inline]
    pub fn starts_with(&self, s: &[Ascii]) -> bool {
        start_with_ascii(self.as_bytes(), s)
    }

    #[inline]
    pub fn next_is(&self, c: Ascii) -> bool {
        next_is_ascii(self.as_bytes(), c)
    }

    #[inline]
    pub fn adv_next_is(&self, amt: usize, c: Ascii) -> bool {
        next_is_ascii(&self.as_bytes()[amt..], c)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.rest.is_empty()
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.rest.as_bytes()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.rest.len()
    }

    #[inline]
    pub fn chars(&self) -> Chars<'a> {
        self.rest.chars()
    }
}

#[inline]
fn find_ascii(rest: &[u8], p: Ascii) -> Option<usize> {
    rest.iter().copied().position(|x| x == p.g())
}

#[inline]
fn next_is_ascii(rest: &[u8], c: Ascii) -> bool {
    rest.first().copied().map_or(false, |x| x == c.g())
}

#[inline]
fn start_with_ascii(rest: &[u8], s: &[Ascii]) -> bool {
    rest.len() >= s.len()
        && rest
            .iter()
            .copied()
            .zip(s.iter().map(|x| x.g()))
            .all(|(a, b)| a == b)
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Ascii(u8);
macro_rules! ascii_builder {
    ($($n:literal)+) => {
        /// New ascii
        /// ```rust
        /// # use yarte_lexer::{Ascii, ascii};
        /// // valid syntax `b'[char]'`
        /// const BAR: Ascii = ascii!(b'f');
        /// ```
        ///
        /// ```compile_fail
        /// # use yarte_lexer::{Ascii, ascii};
        /// const BAR: Ascii = ascii!(b' ');
        /// ```
        ///
        /// ```compile_fail
        /// # use yarte_lexer::{Ascii, ascii};
        /// const BAR: Ascii = ascii!(1);
        /// ```
        ///
        /// ```compile_fail
        /// # use yarte_lexer::{Ascii, ascii};
        /// const BAR: Ascii = ascii!(0x01);
        /// ```
        #[macro_export]
        macro_rules! ascii {
            $(($n) => { unsafe { Ascii::new($n) } };)+
            ($t:tt) => {
                compile_error!(
                    concat!("No valid ascii token select another or report: ", stringify!($t))
                );
            };
        }
    };
}

#[rustfmt::skip]
ascii_builder!(
    b'!' b'"' b'#' b'$' b'%' b'&' b'\'' b'(' b')' b'*' b'+' b',' b'-' b'.' b'/' b'\\'
    b'0' b'1' b'2' b'3' b'4' b'5' b'6' b'7' b'8' b'9' b':' b';' b'<' b'=' b'>' b'?'
    b'@' b'A' b'B' b'C' b'D' b'E' b'F' b'G' b'H' b'I' b'J' b'K' b'L' b'M' b'N' b'O'
    b'P' b'Q' b'R' b'S' b'T' b'U' b'V' b'W' b'X' b'Y' b'Z' b'[' b']' b'^' b'_' b'`'
    b'a' b'b' b'c' b'd' b'e' b'f' b'g' b'h' b'i' b'j' b'k' b'l' b'm' b'n' b'o' b'p'
    b'q' b'r' b's' b't' b'u' b'v' b'w' b'x' b'y' b'z' b'{' b'|' b'}' b'~'
);

impl Ascii {
    /// Unchecked new ascii
    ///
    /// # Safety
    /// Use `ascii!(b'[char]')` macro instead
    pub const unsafe fn new(n: u8) -> Self {
        Self(n)
    }

    pub const fn g(self) -> u8 {
        self.0
    }
}

#[inline]
fn ascii_to_str(s: &[Ascii]) -> &str {
    // SAFETY: the caller must guarantee that the bytes `s`
    // are valid UTF-8, thus the cast to `*mut str` is safe.
    // Also, the pointer dereference is safe because that pointer
    // comes from a reference which is guaranteed to be valid for writes.
    // And Ascii have transparent representation
    unsafe { &mut *(s as *const [Ascii] as *mut [u8] as *mut str) }
}

#[inline]
fn ascii_to_char(s: Ascii) -> char {
    // SAFETY: the caller must guarantee that the byte `s`
    // is valid UTF-8, thus the cast to `char` is safe.
    s.g() as char
}

impl Into<char> for Ascii {
    fn into(self) -> char {
        ascii_to_char(self)
    }
}

pub fn get_chars(text: &str, left: usize, right: usize) -> &str {
    debug_assert!(right.checked_sub(left).is_some());

    let (left, right) = text
        .char_indices()
        .chain(once((text.len(), '\0')))
        .skip(left)
        .take(right - left + 1)
        .fold((None, 0), |(left, _), (i, _)| {
            if left.is_some() {
                (left, i)
            } else {
                (Some(i), i)
            }
        });

    left.map_or("", |left| &text[left..right])
}

#[macro_export]
macro_rules! do_parse {
    ($i:expr, ( $($rest:expr),* )) => {
        Ok(($i, ( $($rest),* )))
    };

    ($i:expr, $e:ident >> $($rest:tt)*) => {
        do_parse!($i, call!($e) >> $($rest)*)
    };

    ($i:expr, $submac:ident!( $($args:tt)* ) >> $($rest:tt)*) => {
        match $submac!($i, $($args)*) {
            Err(e) => Err(e),
            Ok((i, _)) => do_parse!(i, $($rest)*),
        }
    };

    ($i:expr, $field:ident : $e:ident >> $($rest:tt)*) => {
        do_parse!($i, $field: call!($e) >> $($rest)*)
    };

    ($i:expr, $field:ident : $submac:ident!( $($args:tt)* ) >> $($rest:tt)*) => {
        match $submac!($i, $($args)*) {
            Err(e) => Err(e),
            Ok((i, o)) => {
                let $field = o;
                do_parse!(i, $($rest)*)
            },
        }
    };
}

#[macro_export]
macro_rules! call {
    ($i:expr, $fun:expr $(, $args:expr)*) => {
        $fun($i $(, $args)*)
    };
}

#[macro_export]
macro_rules! opt {
    ($i:expr, $submac:ident!($($args:tt)*)) => {
        match $submac!($i, $($args)*) {
            Ok((i, o)) => Ok((i, Some(o))),
            Err(_) => Ok(($i, None)),
        }
    };
    ($i:expr, $f:expr) => {
        match $f($i) {
            Ok((i, o)) => Ok((i, Some(o))),
            Err(_) => Ok(($i, None)),
        }
    };
}

#[macro_export]
macro_rules! take_while {
    ($i:expr, $f:expr) => {{
        if $i.len() == 0 {
            Ok(($i, ""))
        } else {
            match $i.chars().position(|c| !$f(c)) {
                Some(c) => Ok(($i.adv(c), $crate::strnom::get_chars(&$i.rest, 0, c))),
                None => Ok((
                    $i.adv($i.len()),
                    $crate::strnom::get_chars(&$i.rest, 0, $i.len()),
                )),
            }
        }
    }};
}

#[inline]
pub fn tag<'a, E: KiError>(i: Cursor<'a>, tag: &'static [Ascii]) -> PResult<'a, &'static str, E> {
    debug_assert!(!tag.is_empty());

    if i.starts_with(tag) {
        Ok((i.adv_ascii(tag), ascii_to_str(tag)))
    } else {
        Err(LexError::Next(
            E::tag(ascii_to_str(tag)),
            Span::from_len(i, tag.len()),
        ))
    }
}

#[inline]
pub fn tac<E: KiError>(i: Cursor, tag: Ascii) -> PResult<char, E> {
    if i.next_is(tag) {
        Ok((i.adv(1), tag.into()))
    } else {
        Err(LexError::Next(E::tac(tag.into()), Span::from(i)))
    }
}

#[macro_export]
macro_rules! map_fail {
    ($($t:tt)*) => {
        ($($t)*).map_err(|e| match e {
            LexError::Next(m, s) => LexError::Fail(m, s),
            e => e,
        });
    };
}

pub fn ws<E: KiError>(input: Cursor) -> PResult<(), E> {
    if input.is_empty() {
        return Err(LexError::Next(E::WHITESPACE, Span::from(input)));
    }

    take_while!(input, is_ws).map(|(c, _)| (c, ()))
}

pub fn skip_ws<E: KiError>(input: Cursor) -> Cursor {
    match ws::<E>(input) {
        Ok((rest, _)) => rest,
        Err(_) => input,
    }
}

#[inline]
pub fn is_ws(ch: char) -> bool {
    ch.is_whitespace()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[ignore = "not yet implemented"]
    fn cursor() {
        unimplemented!()
    }

    #[test]
    fn test_get_chars() {
        let rest = "foó bañ tuú";
        let len = rest.chars().count();
        assert_eq!("", get_chars(rest, len, len));
        assert_eq!("", get_chars(rest, 0, 0));
        assert_eq!(rest, get_chars(rest, 0, std::usize::MAX - 1));
        assert_eq!(rest, get_chars(rest, 0, len));
        assert_eq!(&rest[1..], get_chars(rest, 1, len));
        assert_eq!(&rest[4..], get_chars(rest, 3, len));
        assert_eq!(&rest[4..rest.len() - 3], get_chars(rest, 3, len - 2));
        assert_eq!(&rest[4..rest.len() - 2], get_chars(rest, 3, len - 1));
        assert_eq!(&rest[7..rest.len() - 5], get_chars(rest, 6, len - 4));
    }
}
