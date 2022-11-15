use std::borrow::Cow;
use thiserror::Error;
use yarte_strnom::error::KiError;

pub type Result<'a, O> = yarte_strnom::error::Result<'a, O, Error>;
pub type CResult<'a> = yarte_strnom::error::CResult<'a, Error>;

impl KiError for Error {
    const EMPTY: Self = Self::Empty;
    const UNCOMPLETED: Self = Self::Uncompleted;
    const PATH: Self = Self::Empty;
    const WHITESPACE: Self = Self::Whitespace;

    fn str(s: &'static str) -> Self {
        Self::Str(Cow::Borrowed(s))
    }

    fn char(c: char) -> Self {
        Self::Char(c)
    }

    fn string(s: String) -> Self {
        Self::Str(Cow::Owned(s))
    }
}

#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum Error {
    #[error("Incorrect first char of token")]
    StartToken,
    #[error("Punct")]
    Punct, // 3 errors
    #[error("Literal")]
    Literal, // 11 errors (is not ident start, )
    #[error("Ident")]
    Ident, // 2 errors (literal and L)
    #[error("SinkEnd")]
    SinkEnd,
    #[error("UnmatchedToken")]
    UnmatchedToken,
    #[error("CursorParse")]
    CursorParse,
    #[error("Whitespace")]
    Whitespace,
    #[error("Uncompleted")]
    Uncompleted,
    #[error("empty")]
    Empty,
    #[error("char {}", _0)]
    Char(char),
    #[error("{}", _0)]
    Str(Cow<'static, str>),
}
