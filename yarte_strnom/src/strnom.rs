//! Adapted from [`proc-macro2`](https://github.com/alexcrichton/proc-macro2).

use std::iter::once;
use std::str::{Bytes, Chars};

use crate::error::{KiError, LexError, Result};
use crate::source_map::Span;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Cursor<'a> {
    pub rest: &'a str,
    pub off: u32,
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
    pub fn _new(rest: &str, off: u32) -> Cursor {
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
    pub fn adv_str(&self, s: &str) -> Cursor<'a> {
        Cursor {
            rest: &self.rest[s.len()..],
            off: self.off + s.len() as u32,
        }
    }

    #[inline]
    pub fn find(&self, p: u8) -> Option<usize> {
        find(self.as_bytes(), p)
    }

    #[inline]
    pub fn adv_find(&self, amt: usize, p: u8) -> Option<usize> {
        find(&self.as_bytes()[amt..], p)
    }

    #[inline]
    pub fn adv_starts_with(&self, amt: usize, s: &'static str) -> bool {
        start_with(&self.as_bytes()[amt..], s.as_bytes())
    }

    #[inline]
    pub fn adv_starts_with_bytes(&self, amt: usize, s: &'static [u8]) -> bool {
        start_with(&self.as_bytes()[amt..], s)
    }

    #[inline]
    pub fn starts_with(&self, s: &'static str) -> bool {
        start_with(self.as_bytes(), s.as_bytes())
    }

    #[inline]
    pub fn starts_with_bytes(&self, s: &'static [u8]) -> bool {
        start_with(self.as_bytes(), s)
    }

    #[inline]
    pub fn next_is(&self, c: u8) -> bool {
        next_is(self.as_bytes(), c)
    }

    #[inline]
    pub fn adv_next_is(&self, amt: usize, c: u8) -> bool {
        next_is(&self.as_bytes()[amt..], c)
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
fn find(rest: &[u8], p: u8) -> Option<usize> {
    memchr::memchr(p, rest)
}

#[inline]
fn next_is(rest: &[u8], c: u8) -> bool {
    rest.first().copied().map_or(false, |x| x == c)
}

#[inline]
fn start_with(rest: &[u8], s: &[u8]) -> bool {
    rest.len() >= s.len()
        && rest
            .iter()
            .copied()
            .zip(s.iter().copied())
            .all(|(a, b)| a == b)
}

// Char converters
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

// Macro helpers
/// Call function
#[macro_export]
macro_rules! call {
    ($i:expr, $fun:expr, $($args:tt)*) => {
        $fun($i, $($args)*)
    };
}

/// Make parser
///
/// # Syntax:
/// ```rust
/// # use yarte_strnom::{pipes, do_parse, ws, tac, Cursor, get_cursor};
/// # use yarte_strnom::pipes::*;
/// # use yarte_strnom::error::{Empty, Result, LexError};
/// # use std::path::PathBuf;
/// # let path = PathBuf::from("FooFile");
/// # let path2 = PathBuf::from("FooFile2");
///
/// let stmt = |i| pipes!(i, ws:is_empty:_false);
/// let parser = |i| do_parse!(i, ws= stmt:important => tac[b'b'] => (ws));
/// let result: Result<bool, Empty> = parser(get_cursor(&path, " b"));
/// let (c, result) = result.unwrap();
///
/// assert!(!result);
/// assert!(c.is_empty());
///
/// let result: Result<bool, Empty> = parser(get_cursor(&path2, "b"));
/// assert!(matches!(result.err().unwrap(), LexError::Fail(..)))
/// ```
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

// TODO: remove unnecessary on pipe [] from do_parse
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
/// # Syntax
/// ```rust
/// # use yarte_strnom::{pipes, do_parse, ws, tac, Cursor, get_cursor};
/// # use yarte_strnom::pipes::*;
/// # use yarte_strnom::error::{Empty, Result};
/// # use std::path::PathBuf;
/// # let path = PathBuf::from("FooFile");
/// # const B: u8 = b'b';
///
/// let stmt = |i| pipes!(i, ws:is_empty:map[|x| !x]);
/// let parser = |i| do_parse!(i, ws= stmt => tac[B] => (ws));
/// let result: Result<bool, Empty> = parser(get_cursor(&path, " b"));
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

// TODO: implement for Cursor
pub mod pipes {
    use super::*;
    use std::fmt::Debug;
    use std::result;

    /// Result Pipe optional is some
    #[inline]
    pub fn is_some<'a, O, E>(_: Cursor<'a>, next: Result<'a, Option<O>, E>) -> Result<'a, bool, E> {
        match next {
            Ok((i, o)) => Ok((i, o.is_some())),
            Err(e) => Err(e),
        }
    }

    /// Result Pipe optional is none
    #[inline]
    pub fn is_none<'a, O, E>(_: Cursor<'a>, next: Result<'a, Option<O>, E>) -> Result<'a, bool, E> {
        match next {
            Ok((i, o)) => Ok((i, o.is_none())),
            Err(e) => Err(e),
        }
    }

    /// Result Pipe optional
    #[inline]
    pub fn opt<'a, O, E>(i: Cursor<'a>, next: Result<'a, O, E>) -> Result<'a, Option<O>, E> {
        match next {
            Ok((i, o)) => Ok((i, Some(o))),
            Err(_) => Ok((i, None)),
        }
    }

    // TODO: is it really usable?
    /// Result Pipe then
    #[inline]
    pub fn then<'a, O, E, N, F, Callback>(
        i: Cursor<'a>,
        next: Result<'a, O, E>,
        mut callback: Callback,
    ) -> Result<'a, N, F>
    where
        Callback: FnMut(result::Result<O, E>) -> result::Result<N, F>,
    {
        match next {
            Ok((c, o)) => callback(Ok(o))
                .map(|n| (c, n))
                .map_err(|e| LexError::Next(e, Span::from_cursor(i, c))),
            Err(LexError::Fail(e, s)) => callback(Err(e))
                .map(|n| (i, n))
                .map_err(|e| LexError::Fail(e, s)),
            Err(LexError::Next(e, s)) => callback(Err(e))
                .map(|n| (i, n))
                .map_err(|e| LexError::Next(e, s)),
        }
    }

    /// Result Pipe then
    #[inline]
    pub fn and_then<'a, O, E, N, Callback>(
        i: Cursor<'a>,
        next: Result<'a, O, E>,
        mut callback: Callback,
    ) -> Result<'a, N, E>
    where
        Callback: FnMut(O) -> result::Result<N, E>,
    {
        match next {
            Ok((c, o)) => callback(o)
                .map(|n| (c, n))
                .map_err(|e| LexError::Next(e, Span::from_cursor(i, c))),
            Err(e) => Err(e),
        }
    }

    /// Result Pipe map
    #[inline]
    pub fn map<'a, O, E, N, Callback>(
        _: Cursor<'a>,
        next: Result<'a, O, E>,
        mut callback: Callback,
    ) -> Result<'a, N, E>
    where
        Callback: FnMut(O) -> N,
    {
        next.map(|(i, x)| (i, callback(x)))
    }

    /// Result Pipe map_err
    #[inline]
    pub fn map_err<'a, O, E, F, Callback>(
        _: Cursor<'a>,
        next: Result<'a, O, E>,
        mut c: Callback,
    ) -> Result<'a, O, F>
    where
        Callback: FnMut(E) -> F,
    {
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
    pub fn is_empty<'a, O: IsEmpty, E>(
        _: Cursor<'a>,
        next: Result<'a, O, E>,
    ) -> Result<'a, bool, E> {
        match next {
            Ok((i, o)) => Ok((i, o.is_empty_())),
            Err(e) => Err(e),
        }
    }

    // TODO:
    pub trait False {
        fn _false(&self) -> bool;
    }

    impl False for bool {
        #[inline]
        fn _false(&self) -> bool {
            !*self
        }
    }

    impl False for &str {
        #[inline]
        fn _false(&self) -> bool {
            self.is_empty()
        }
    }

    impl<T> False for Vec<T> {
        #[inline]
        fn _false(&self) -> bool {
            self.is_empty()
        }
    }

    /// Result Pipe true to error next
    #[inline]
    pub fn _false<'a, O: False + As<N>, E: KiError, N>(
        i: Cursor<'a>,
        next: Result<'a, O, E>,
    ) -> Result<'a, N, E> {
        match next {
            Ok((i, o)) if o._false() => Ok((i, o._as())),
            Ok((c, _)) => Err(LexError::Next(E::EMPTY, Span::from_cursor(i, c))),
            Err(e) => Err(e),
        }
    }

    // TODO:
    pub trait True {
        fn _true(&self) -> bool;
    }

    impl True for bool {
        #[inline]
        fn _true(&self) -> bool {
            *self
        }
    }

    impl True for &str {
        #[inline]
        fn _true(&self) -> bool {
            !self.is_empty()
        }
    }

    impl<T> True for Vec<T> {
        #[inline]
        fn _true(&self) -> bool {
            !self.is_empty()
        }
    }

    /// Result Pipe false to error next
    #[inline]
    pub fn _true<'a, O: True + As<N>, E: KiError, N>(
        i: Cursor<'a>,
        next: Result<'a, O, E>,
    ) -> Result<'a, N, E> {
        match next {
            Ok((i, o)) if o._true() => Ok((i, o._as())),
            Ok((c, _)) => Err(LexError::Next(E::EMPTY, Span::from_cursor(i, c))),
            Err(e) => Err(e),
        }
    }

    // TODO: feature specialized, when is stable?
    pub trait As<N> {
        fn _as(self) -> N;
    }

    macro_rules! impl_as {
        ($($ty:ty)+) => {
            $(
            impl As<$ty> for $ty {
                #[inline]
                fn _as(self) -> $ty { self }
            }
            )+
        };
    }

    impl_as!(char bool usize u64 u32 u16 u8 isize i64 i32 i16 i8);

    impl<T> As<Vec<T>> for Vec<T> {
        #[inline]
        fn _as(self) -> Vec<T> {
            self
        }
    }

    impl<'a> As<&'a str> for &'a str {
        #[inline]
        fn _as(self) -> &'a str {
            self
        }
    }

    impl<'a> As<Cursor<'a>> for Cursor<'a> {
        #[inline]
        fn _as(self) -> Cursor<'a> {
            self
        }
    }

    /// Result Pipe As
    #[inline]
    pub fn _as<'a, O: As<N>, E, N>(_: Cursor<'a>, next: Result<'a, O, E>) -> Result<'a, N, E> {
        match next {
            Ok((i, o)) => Ok((i, o._as())),
            Err(e) => Err(e),
        }
    }

    /// Result Pipe to Len comparator

    #[inline]
    pub fn debug<'a, O: Debug, E: Debug>(
        i: Cursor<'a>,
        next: Result<'a, O, E>,
        message: &'static str,
    ) -> Result<'a, O, E> {
        // TODO: use log!
        eprintln!("{message}:\n\tCursor: {i:?}\n\tnext: {next:?}\n");
        next
    }

    pub trait IsLen {
        fn is_len(&self, len: usize) -> bool;
    }

    impl IsLen for &str {
        fn is_len(&self, len: usize) -> bool {
            self.len() == len
        }
    }

    impl<T> IsLen for Vec<T> {
        fn is_len(&self, len: usize) -> bool {
            self.len() == len
        }
    }

    #[derive(Debug)]
    pub struct Not<T>(T);

    /// Result Pipe to Len comparator
    #[inline]
    pub fn not<'a, O, E: KiError>(_: Cursor<'a>, next: Result<'a, O, E>) -> Result<'a, Not<O>, E> {
        match next {
            Ok((i, o)) => Ok((i, Not(o))),
            Err(e) => Err(e),
        }
    }

    impl<T> As<T> for Not<T> {
        #[inline]
        fn _as(self) -> T {
            self.0
        }
    }

    impl<T: True> True for Not<T> {
        #[inline]
        fn _true(&self) -> bool {
            !self.0._true()
        }
    }

    impl<T: False> False for Not<T> {
        #[inline]
        fn _false(&self) -> bool {
            !self.0._false()
        }
    }

    impl<T: IsLen> IsLen for Not<T> {
        #[inline]
        fn is_len(&self, len: usize) -> bool {
            !self.0.is_len(len)
        }
    }

    /// Result Pipe to Len comparator
    #[inline]
    pub fn is_len<'a, O: IsLen + As<N>, E: KiError, N>(
        i: Cursor<'a>,
        next: Result<'a, O, E>,
        len: usize,
    ) -> Result<'a, N, E> {
        match next {
            Ok((i, o)) if o.is_len(len) => Ok((i, o._as())),
            Ok((c, _)) => Err(LexError::Next(E::EMPTY, Span::from_cursor(i, c))),
            Err(e) => Err(e),
        }
    }

    #[inline]
    pub fn from<'a, O, E, N: From<O>>(_: Cursor<'a>, next: Result<'a, O, E>) -> Result<'a, N, E> {
        next.map(|(c, x)| (c, N::from(x)))
    }

    #[inline]
    pub fn into<'a, O: Into<N>, E, N>(_: Cursor<'a>, next: Result<'a, O, E>) -> Result<'a, N, E> {
        next.map(|(c, x)| (c, x.into()))
    }

    /// Cast next error to Fail error
    #[inline]
    pub fn important<'a, O, E>(_: Cursor<'a>, next: Result<'a, O, E>) -> Result<'a, O, E> {
        match next {
            Ok(o) => Ok(o),
            Err(LexError::Next(m, s)) => Err(LexError::Fail(m, s)),
            Err(e) => Err(e),
        }
    }
}

// TODO: Should be return a Cursor
/// Take while function is true or empty Ok if is empty
#[inline]
pub fn _while<E, Callback: FnMut(u8) -> bool>(i: Cursor, mut f: Callback) -> Result<&str, E> {
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
pub fn tag<'a, E: KiError>(i: Cursor<'a>, tag: &'static str) -> Result<'a, &'static str, E> {
    debug_assert!(!tag.is_empty());

    if i.starts_with(tag) {
        Ok((i.adv_str(tag), tag))
    } else {
        Err(LexError::Next(E::str(tag), Span::from_len(i, tag.len())))
    }
}

/// Take an ascii character or next error
#[inline]
pub fn tac<E: KiError>(i: Cursor, tag: u8) -> Result<char, E> {
    if i.next_is(tag) {
        Ok((i.adv(1), tag.into()))
    } else {
        Err(LexError::Next(E::char(tag.into()), Span::from(i)))
    }
}

/// Take an ascii whitespace or next error
#[inline]
pub fn next_ws<E: KiError>(i: Cursor) -> Result<(), E> {
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
pub fn ws<E: KiError>(i: Cursor) -> Result<&str, E> {
    if i.is_empty() {
        return Err(LexError::Next(E::WHITESPACE, Span::from(i)));
    }
    _while(i, is_ws)
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
        assert_eq!(rest, get_chars(rest, 0, usize::MAX - 1));
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
