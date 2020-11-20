//! Adapted from [`proc-macro2`](https://github.com/alexcrichton/proc-macro2).

use std::iter::once;
use std::str::{Bytes, Chars};

use crate::error::{KiError, LexError, PResult};
use crate::source_map::Span;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Cursor<'a> {
    pub rest: &'a str,
    pub(crate) off: u32,
}

impl<'a> Cursor<'a> {
    /// Create new unregistered cursor
    ///
    /// # Safety
    /// Use get_cursor function instead for registered cursor
    #[inline]
    pub unsafe fn new(rest: &str, off: u32) -> Cursor {
        Cursor { rest, off }
    }

    #[inline]
    pub(crate) fn _new(rest: &str, off: u32) -> Cursor {
        unsafe { Self::new(rest, off) }
    }

    #[inline]
    pub fn off(&self) -> usize {
        self.off as usize
    }

    #[inline]
    pub fn adv(&self, amt: usize) -> Cursor<'a> {
        Cursor {
            rest: &self.rest[amt..],
            off: self.off + amt as u32,
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
    pub fn adv_starts_with(&self, amt: usize, s: &'static [Ascii]) -> bool {
        start_with_ascii(&self.as_bytes()[amt..], s)
    }

    #[inline]
    pub fn starts_with(&self, s: &'static [Ascii]) -> bool {
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
    pub fn bytes(&self) -> Bytes<'_> {
        self.rest.bytes()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.rest.len()
    }

    #[inline]
    pub fn chars(&self) -> Chars<'_> {
        self.rest.chars()
    }
}

#[inline]
fn find_ascii(rest: &[u8], p: Ascii) -> Option<usize> {
    memchr::memchr(p.g(), rest)
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

/// Bound byte to few ascii character
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Ascii(u8);
macro_rules! ascii_builder {
    ($($n:literal)+) => {
        /// New ascii
        /// ```rust
        /// # use yarte_lexer::{Ascii, ascii};
        /// // valid syntax `b'[char]'`
        /// const BAR: Ascii = ascii!('f');
        /// ```
        ///
        /// ```compile_fail
        /// # use yarte_lexer::{Ascii, ascii};
        /// const BAR: Ascii = ascii!(' ');
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
            $(($n) => { unsafe { $crate::Ascii::new($n) } };)+
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
    '!' '"' '#' '$' '%' '&' '\'' '(' ')' '*' '+' ',' '-' '.' '/' '\\'
    '0' '1' '2' '3' '4' '5' '6' '7' '8' '9' ':' ';' '<' '=' '>' '?'
    '@' 'A' 'B' 'C' 'D' 'E' 'F' 'G' 'H' 'I' 'J' 'K' 'L' 'M' 'N' 'O'
    'P' 'Q' 'R' 'S' 'T' 'U' 'V' 'W' 'X' 'Y' 'Z' '[' ']' '^' '_' '`'
    'a' 'b' 'c' 'd' 'e' 'f' 'g' 'h' 'i' 'j' 'k' 'l' 'm' 'n' 'o' 'p'
    'q' 'r' 's' 't' 'u' 'v' 'w' 'x' 'y' 'z' '{' '|' '}' '~'
);

impl Ascii {
    /// Unchecked new ascii
    ///
    /// # Safety
    /// Use `ascii!(b'[char]')` macro instead
    pub const unsafe fn new(n: char) -> Self {
        Self(n as u8)
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

pub trait AsStr {
    fn as_str(&self) -> &str;
}

impl AsStr for [Ascii] {
    fn as_str(&self) -> &str {
        ascii_to_str(self)
    }
}

/// Get char range in text
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

/// Cast byte range to chars range in text
pub fn get_bytes_to_chars(text: &str, left: usize, right: usize) -> (usize, usize) {
    let mut taken = false;
    let (left, right) = text
        .char_indices()
        .chain(once((text.len(), '\0')))
        .enumerate()
        .skip_while(|(_, (i, _))| *i < left)
        .take_while(|(_, (i, _))| {
            if taken {
                return false;
            }
            if *i >= right {
                taken = true;
            }
            true
        })
        .fold((None, 0), |(left, _), (i, _)| {
            if left.is_some() {
                (left, i)
            } else {
                (Some(i), i)
            }
        });

    left.map_or((0, 0), |left| (left, right))
}

// TODO: remove unnecessary on pipe [] from do_parse
#[macro_export]
macro_rules! do_parse {
    ($i:expr, ( $($rest:tt)* )) => {
        Ok(($i, ( $($rest)* )))
    };

    ($i:expr, $fun:path $(:$pipe:path)* => $($rest:tt)+) => {
        do_parse!($i, $fun[] $(:$pipe)* => $($rest)+)
    };

    ($i:expr, $field:ident = $fun:path $(:$pipe:path)* => $($rest:tt)+) => {
        do_parse!($i, $field = $fun[] $(:$pipe)* => $($rest)+)
    };
    ($i:expr, $field:ident = $fun:path $(:$pipe:path[$($argsp:tt)*])* => $($rest:tt)+) => {
        do_parse!($i, $field = $fun[] $(:$pipe[$($argsp)*])* => $($rest)+)
    };
    ($i:expr, $fun:path [ $($args:tt)* ]$(:$pipe:path)*  => $($rest:tt)+) => {
        match $crate::pipes!($i, $fun[$($args)*]$(:$pipe)*) {
            Err(e) => Err(e),
            Ok((i, _)) => do_parse!(i, $($rest)+),
        }
    };
    ($i:expr, $fun:path [ $($args:tt)* ]$(:$pipe:path[$($argsp:tt)*])*  => $($rest:tt)+) => {
        match $crate::pipes!($i, $fun[$($args)*]$(:$pipe[$($argsp)*])*) {
            Err(e) => Err(e),
            Ok((i, _)) => do_parse!(i, $($rest)+),
        }
    };
    ($i:expr, $field:ident = $fun:path [ $($args:tt)* ]$(:$pipe:path)* => $($rest:tt)+) => {{
        match $crate::pipes!($i, $fun[$($args)*]$(:$pipe)*) {
            Err(e) => Err(e),
            Ok((i, o)) => {
                let $field = o;
                do_parse!(i, $($rest)+)
            },
        }
    }};
    ($i:expr, $field:ident = $fun:path [ $($args:tt)* ]$(:$pipe:path[$($argsp:tt)*])* => $($rest:tt)+) => {{
        match $crate::pipes!($i, $fun[$($args)*]$(:$pipe[$($argsp)*])*) {
            Err(e) => Err(e),
            Ok((i, o)) => {
                let $field = o;
                do_parse!(i, $($rest)+)
            },
        }
    }};
}

#[macro_export]
macro_rules! alt {
    ($i:expr, $fun:path $(:$pipe:path)*) => {
        alt!($i, $fun[] $(:$pipe)*)
    };

    ($i:expr, $fun:path [ $($args:tt)* ]$(:$pipe:path)*) => {
        $crate::pipes!($i, $fun[$($args)*]$(:$pipe)*)
    };
    ($i:expr, $fun:path $(:$pipe:path)* | $($rest:tt)+) => {
        alt!($i, $fun[] $(:$pipe)* | $($rest)+)
    };

    ($i:expr, $fun:path [ $($args:tt)* ]$(:$pipe:path)* | $($rest:tt)+) => {
        match $crate::pipes!($i, $fun[$($args)*]$(:$pipe)*) {
            Ok(o) => Ok(o),
            Err($crate::LexError::Next(..)) => alt!($i, $($rest)*),
            Err(e) => Err(e),
        }
    };
}

/// Make a in tail function call
///
/// # Syntax:
/// ```rust
/// use yarte_lexer::{pipes, ws, is_empty, map, do_parse, tac, ascii, Ascii, PResult, error::Empty, Cursor, get_cursor};
/// # use std::path::PathBuf;
/// # let path = PathBuf::from("FooFile");
/// const B: Ascii = ascii!('b');
///
/// let stmt = |i| pipes!(i, ws:is_empty:map[|x| !x]);
/// let parser = |i| do_parse!(i, ws= stmt => tac[B] => (ws));
/// let result: PResult<bool, Empty> = parser(get_cursor(&path, " b"));
/// let (c, result) = result.unwrap();
///
/// assert!(result);
/// assert!(c.is_empty());
/// ```
#[macro_export]
macro_rules! pipes {
    ($i:expr, $fun:path) => {
        $crate::call!($i, $fun)
    };
    ($i:expr, $fun:path [ $($args:tt)* ]) => {
        $crate::call!($i, $fun, $($args)*)
    };
    ($i:expr, $fun:path [ $($args:tt)* ] : $($rest:tt)+) => {
        $crate::pipes!(impl $i, $crate::pipes!($i, $fun[$($args)*]), : $($rest)+)
    };
    ($i:expr, $fun:path : $($rest:tt)+) => {
        $crate::pipes!(impl $i, $crate::pipes!($i, $fun[]), : $($rest)+)
    };

    (impl $i:expr, $r:expr, :$pipe:path) => {
        $crate::call!($i, $pipe, $r)
    };
    (impl $i:expr, $r:expr, :$pipe:path[$($args:tt)*]) => {
        $crate::call!($i, $pipe, $r, $($args)*)
    };
    (impl $i:expr, $r:expr, :$pipe:path : $($rest:tt)+) => {
        $crate::pipes!(impl $i, $pipe($i, $r), : $($rest)+)
    };
    (impl $i:expr, $r:expr, :$pipe:path[$($args:tt)*] : $($rest:tt)+) => {
        $crate::pipes!(impl $i, $pipe($i, $r, $($args)*), : $($rest)+)
    };
}

#[macro_export]
macro_rules! call {
    ($i:expr, $fun:expr, $($args:tt)*) => {
        $fun($i, $($args)*)
    };
}

/// Result Pipe optional is some
#[inline]
pub fn is_some<'a, E, O>(_: Cursor<'a>, next: PResult<'a, Option<O>, E>) -> PResult<'a, bool, E> {
    match next {
        Ok((i, o)) => Ok((i, o.is_some())),
        Err(e) => Err(e),
    }
}

/// Result Pipe optional is none
#[inline]
pub fn is_none<'a, E: KiError, O>(
    _: Cursor<'a>,
    next: PResult<'a, Option<O>, E>,
) -> PResult<'a, bool, E> {
    match next {
        Ok((i, o)) => Ok((i, o.is_none())),
        Err(e) => Err(e),
    }
}

/// Result Pipe optional
#[inline]
pub fn opt<'a, E: KiError, O>(i: Cursor<'a>, next: PResult<'a, O, E>) -> PResult<'a, Option<O>, E> {
    match next {
        Ok((i, o)) => Ok((i, Some(o))),
        Err(_) => Ok((i, None)),
    }
}

/// Result Pipe then
#[inline]
pub fn then<'a, E, O, N>(
    _: Cursor<'a>,
    next: PResult<'a, O, E>,
    c: fn(PResult<'a, O, E>) -> PResult<'a, N, E>,
) -> PResult<'a, N, E> {
    c(next)
}

/// Result Pipe map
#[inline]
pub fn map<'a, E, O, N>(
    _: Cursor<'a>,
    next: PResult<'a, O, E>,
    c: fn(O) -> N,
) -> PResult<'a, N, E> {
    next.map(|(i, x)| (i, c(x)))
}

/// Result Pipe map_err
#[inline]
pub fn map_err<'a, E, O>(
    _: Cursor<'a>,
    next: PResult<'a, O, E>,
    c: fn(E) -> E,
) -> PResult<'a, O, E> {
    next.map_err(|x| match x {
        LexError::Next(e, s) => LexError::Next(c(e), s),
        LexError::Fail(e, s) => LexError::Fail(c(e), s),
    })
}

pub trait IsEmpty {
    fn is_empty_(&self) -> bool;
}

impl IsEmpty for &str {
    #[inline]
    fn is_empty_(&self) -> bool {
        self.is_empty()
    }
}

impl<T> IsEmpty for Vec<T> {
    #[inline]
    fn is_empty_(&self) -> bool {
        self.is_empty()
    }
}

/// Result Pipe is_empty
#[inline]
pub fn is_empty<'a, E, O: IsEmpty>(_: Cursor<'a>, next: PResult<'a, O, E>) -> PResult<'a, bool, E> {
    match next {
        Ok((i, o)) => Ok((i, o.is_empty_())),
        Err(e) => Err(e),
    }
}

// TODO:
pub trait NotTrue {
    fn not_true(&self) -> bool;
}

impl NotTrue for bool {
    #[inline]
    fn not_true(&self) -> bool {
        !*self
    }
}

/// Result Pipe true to error next
#[inline]
pub fn not_true<'a, E: KiError, O: NotTrue>(
    i: Cursor<'a>,
    next: PResult<'a, O, E>,
) -> PResult<'a, O, E> {
    match next {
        Ok((i, o)) if o.not_true() => Ok((i, o)),
        Ok(_) => Err(LexError::Next(E::UNCOMPLETED, Span::from(i))),
        Err(e) => Err(e),
    }
}

// TODO:
pub trait NotFalse {
    fn not_false(&self) -> bool;
}

impl NotFalse for bool {
    #[inline]
    fn not_false(&self) -> bool {
        *self
    }
}

/// Result Pipe false to error next
#[inline]
pub fn not_false<'a, E: KiError, O: NotFalse>(
    i: Cursor<'a>,
    next: PResult<'a, O, E>,
) -> PResult<'a, O, E> {
    match next {
        Ok((i, o)) if o.not_false() => Ok((i, o)),
        Ok(_) => Err(LexError::Next(E::UNCOMPLETED, Span::from(i))),
        Err(e) => Err(e),
    }
}

/// Cast next error to Fail error
#[inline]
pub fn important<'a, E, O>(_: Cursor<'a>, next: PResult<'a, O, E>) -> PResult<'a, O, E> {
    match next {
        Ok(o) => Ok(o),
        Err(LexError::Next(m, s)) => Err(LexError::Fail(m, s)),
        Err(e) => Err(e),
    }
}

/// Take while function is true or empty Ok if is empty
#[inline]
pub fn take_while<E>(i: Cursor, f: fn(u8) -> bool) -> PResult<&str, E> {
    if i.is_empty() {
        Ok((i, ""))
    } else {
        match i.bytes().position(|x| !f(x)) {
            None => Ok((i.adv(i.len()), i.rest)),
            Some(j) => Ok((i.adv(j), &i.rest[..j])),
        }
    }
}

/// Take ascii characters or next error
#[inline]
pub fn tag<'a, E: KiError>(i: Cursor<'a>, tag: &'static [Ascii]) -> PResult<'a, &'static str, E> {
    debug_assert!(!tag.is_empty());

    if i.starts_with(tag) {
        Ok((i.adv_ascii(tag), tag.as_str()))
    } else {
        Err(LexError::Next(
            E::str(tag.as_str()),
            Span::from_len(i, tag.len()),
        ))
    }
}

/// Take an ascii character or next error
#[inline]
pub fn tac<E: KiError>(i: Cursor, tag: Ascii) -> PResult<char, E> {
    if i.next_is(tag) {
        Ok((i.adv(1), tag.into()))
    } else {
        Err(LexError::Next(E::char(tag.into()), Span::from(i)))
    }
}

/// Take an ascii whitespace or next error
#[inline]
pub fn next_ws<E: KiError>(i: Cursor) -> PResult<(), E> {
    i.as_bytes()
        .first()
        .copied()
        .map_or(Err(LexError::Next(E::WHITESPACE, Span::from(i))), |b| {
            if is_ws(b) {
                Ok((i.adv(1), ()))
            } else {
                Err(LexError::Next(E::WHITESPACE, Span::from(i)))
            }
        })
}

/// Take ascii whitespaces, next error if is empty
#[inline]
pub fn ws<E: KiError>(i: Cursor) -> PResult<&str, E> {
    if i.is_empty() {
        return Err(LexError::Next(E::WHITESPACE, Span::from(i)));
    }
    take_while(i, is_ws)
}

#[inline]
pub fn skip_ws<E: KiError>(input: Cursor) -> Cursor {
    match ws::<E>(input) {
        Ok((rest, _)) => rest,
        Err(_) => input,
    }
}

#[inline]
pub fn is_ws(ch: u8) -> bool {
    ch.is_ascii_whitespace()
}

#[cfg(test)]
mod test {
    use super::*;

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

    #[test]
    fn test_get_bytes_chars() {
        let rest = "foó bañ tuú";
        assert_eq!(get_bytes_to_chars(rest, 0, 1), (0, 1));
        assert_eq!(get_bytes_to_chars(rest, 1, 2), (1, 2));
        assert_eq!(get_bytes_to_chars(rest, 2, 4), (2, 3));
        assert_eq!(get_bytes_to_chars(rest, 4, 5), (3, 4));
        assert_eq!(get_bytes_to_chars(rest, 5, 6), (4, 5));
        assert_eq!(get_bytes_to_chars(rest, 1, 6), (1, 5));
        assert_eq!(get_bytes_to_chars(rest, 1, 8), (1, 7));
        assert_eq!(get_bytes_to_chars(rest, 1, 9), (1, 7));
        assert_eq!(get_bytes_to_chars(rest, 1, 10), (1, 8));
        assert_eq!(get_bytes_to_chars(rest, 9, 10), (7, 8));
    }
}
