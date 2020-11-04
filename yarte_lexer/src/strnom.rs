//! Adapted from [`proc-macro2`](https://github.com/alexcrichton/proc-macro2).

use std::str::Chars;

use crate::{error::PError, source_map::Span};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Cursor<'a> {
    pub rest: &'a str,
    pub off: u32,
}

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

    pub fn adv_find(&self, amt: usize, p: char) -> Option<usize> {
        self.chars().skip(amt).position(|x| x == p)
    }

    pub fn adv_starts_with(&self, amt: usize, s: &str) -> bool {
        let len = amt + s.chars().count();
        len <= self.len() && self.chars().skip(amt).zip(s.chars()).all(|(a, b)| a == b)
    }

    pub fn starts_with(&self, s: &str) -> bool {
        s.chars().count() <= self.len() && self.chars().zip(s.chars()).all(|(a, b)| a == b)
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
}

#[derive(Debug, Clone)]
pub enum LexError {
    Fail(PError, Span),
    Next(PError, Span),
}

pub type PResult<'a, O> = Result<(Cursor<'a>, O), LexError>;

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
                Some(i) => Ok(($i.adv(i), &$i.rest[..i])),
                None => Ok(($i.adv($i.len()), &$i.rest[..$i.len()])),
            }
        }
    }};
}

#[macro_export]
macro_rules! tag {
    ($i:expr, $tag:expr) => {
        if $i.starts_with($tag) {
            Ok(($i.adv($tag.len()), &$i.rest[..$tag.len()]))
        } else {
            Err(LexError::Next(PError::Tag, Span::from($i)))
        }
    };
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

pub fn ws(input: Cursor) -> PResult<()> {
    if input.is_empty() {
        return Err(LexError::Next(PError::Whitespace, Span::from(input)));
    }

    take_while!(input, is_ws).map(|(c, _)| (c, ()))
}

pub fn skip_ws(input: Cursor) -> Cursor {
    match ws(input) {
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
}
