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
    pub fn adv(&self, amt: usize) -> Cursor<'a> {
        if amt == 0 {
            return *self;
        }

        let len = self.len();
        if amt >= len {
            return Cursor {
                rest: "",
                off: self.off + (len as u32),
            };
        }
        let next = self
            .rest
            .char_indices()
            .nth(amt)
            .map_or(self.rest.len(), |(i, _)| i);
        Cursor {
            rest: &self.rest[next..],
            off: self.off + (amt as u32),
        }
    }

    pub fn find(&self, p: char) -> Option<usize> {
        self.chars().position(|x| x == p)
    }

    #[inline]
    pub fn adv_find(&self, amt: usize, p: char) -> Option<usize> {
        self.chars().skip(amt).position(|x| x == p)
    }

    pub fn adv_starts_with(&self, amt: usize, s: &str) -> bool {
        let len = amt + s.chars().count();
        len <= self.len() && self.chars().skip(amt).zip(s.chars()).all(|(a, b)| a == b)
    }

    pub fn starts_with(&self, s: &str) -> bool {
        self.rest.starts_with(s)
    }

    pub fn next_is(&self, c: char) -> bool {
        self.chars().next().map_or(false, |x| c == x)
    }

    pub fn adv_next_is(&self, amt: usize, c: char) -> bool {
        self.adv(amt).chars().next().map_or(false, |x| c == x)
    }

    pub fn is_empty(&self) -> bool {
        self.rest.is_empty()
    }

    pub fn len(&self) -> usize {
        self.chars().count()
    }

    pub fn chars(&self) -> Chars<'a> {
        self.rest.chars()
    }

    #[inline]
    pub fn get_chars(&self, left: usize, right: usize) -> &str {
        get_chars(self.rest, left, right)
    }
}

#[derive(Debug, Copy, Clone)]
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
            ($t:tt) => { compile_error!(concat!("No valid ascii token select another or report: ", stringify!($t))); }
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
pub fn tag<'a, E: KiError>(i: Cursor<'a>, tag: &'static str) -> PResult<'a, &'static str, E> {
    debug_assert!(!tag.is_empty());

    if i.starts_with(tag) {
        Ok((i.adv(tag.chars().count()), tag))
    } else {
        Err(LexError::Next(
            E::tag(tag),
            Span::from_len(i, tag.chars().count()),
        ))
    }
}

#[inline]
pub fn tac<E: KiError>(i: Cursor, tag: char) -> PResult<char, E> {
    if i.next_is(tag) {
        Ok((i.adv(1), tag))
    } else {
        Err(LexError::Next(E::tac(tag), Span::from(i)))
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
    fn cursor() {
        let rest = "foó bañ tuú";
        let len = rest.chars().count();

        let c = Cursor { rest, off: 0 };
        assert!(c.adv(0).starts_with("f"));
        assert!(c.adv(1).starts_with("o"));
        assert!(c.adv(2).starts_with("ó"));
        assert!(c.adv(3).starts_with(" "));
        assert!(c.adv(6).starts_with("ñ"));
        assert!(c.adv(len).starts_with(""));
        assert!(!c.adv(len).starts_with("ú"));

        assert!(c.adv_starts_with(6, "ñ"));
        assert!(c.adv_starts_with(len, ""));

        assert_eq!(c.find('f'), Some(0));
        assert_eq!(c.find('ñ'), Some(6));
        assert_eq!(c.find('h'), None);

        assert_eq!(c.adv_find(3, 'ñ'), Some(3));
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
