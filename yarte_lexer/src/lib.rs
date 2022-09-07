use std::str;

#[cfg(feature = "test")]
use syn::parse::{Parse, ParseBuffer};

#[macro_use]
mod strnom;
#[macro_use]
pub mod error;
mod arm;
mod expr_list;
mod parse;
mod source_map;
mod stmt_local;

use self::arm::Arm;
use self::error::{KiError, Result as PResult};

pub use self::{
    error::LexError,
    parse::{path, Ki, LexResult, Lexer, Sink},
    source_map::{clean, get_cursor, spanned, LineColumn, Span, S},
    stmt_local::StmtLocal,
    strnom::*,
};

pub type Ws = (bool, bool);

#[cfg(feature = "test")]
#[derive(std::fmt::Debug, PartialEq, Clone, serde::Deserialize)]
#[serde(transparent)]
pub struct Local(#[serde(deserialize_with = "de_local")] syn::Local);

#[cfg(not(feature = "test"))]
pub use syn::Local;

#[cfg(feature = "test")]
fn de_local<'de, D>(deserializer: D) -> Result<syn::Local, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    <&str>::deserialize(deserializer).and_then(|x| {
        syn::parse_str::<StmtLocal>(x)
            .map(Into::into)
            .map_err(|_| serde::de::Error::custom("Parse error"))
    })
}

#[cfg(feature = "test")]
impl Parse for Local {
    fn parse(input: &ParseBuffer<'_>) -> syn::Result<Self> {
        Ok(Local(input.parse::<StmtLocal>()?.into()))
    }
}

#[cfg(feature = "test")]
#[derive(std::fmt::Debug, PartialEq, Clone, serde::Deserialize)]
#[serde(transparent)]
pub struct Expr(#[serde(deserialize_with = "de_expr")] syn::Expr);

#[cfg(not(feature = "test"))]
pub type Expr = syn::Expr;

#[cfg(feature = "test")]
fn de_expr<'de, D>(deserializer: D) -> Result<syn::Expr, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    <&str>::deserialize(deserializer)
        .and_then(|x| syn::parse_str(x).map_err(|_| serde::de::Error::custom("Parse error")))
}

#[cfg(feature = "test")]
impl Parse for Expr {
    fn parse(input: &ParseBuffer<'_>) -> syn::Result<Self> {
        Ok(Expr(input.parse()?))
    }
}

pub type SArm = S<Box<Arm>>;
pub type SExpr = S<Box<Expr>>;
pub type SLocal = S<Box<Local>>;
pub type SStr<'a> = S<&'a str>;
pub type SVExpr = S<Vec<Expr>>;

macro_rules! ki {
    ($ty:ident: $($cname:ident: $cty:ty)+; $($method:ident -> $ret:ty)+) => {
        pub trait $ty<'a>: Sized + 'a {
            type Error: KiError;
            $(
            const $cname: $cty;
            )+
            $(
            #[inline]
            fn $method(_: Cursor<'a>) -> PResult<'a, $ret, Self::Error> {
                Err(next!(Self::Error))
            }
            )+
        }
    };
}

ki!(
    Kinder:
        OPEN: u8
        CLOSE: u8
        OPEN_EXPR: u8
        CLOSE_EXPR: u8
        OPEN_BLOCK: u8
        CLOSE_BLOCK: u8
        WS: u8
        WS_AFTER: bool
    ;
        parse -> Self
        comment -> &'a str
);
