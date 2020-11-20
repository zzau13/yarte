#![allow(dead_code)]

use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::str;

use serde::{Deserialize, Deserializer};
use syn::parse::{Parse, ParseBuffer};

#[macro_use]
mod strnom;
#[macro_use]
pub mod error;
mod arm;
mod expr_list;
mod expr_pipe;
mod parse;
mod source_map;
mod stmt_local;

use self::arm::Arm;
use self::source_map::S;

pub use gencode::asciis;

pub use self::{
    error::{emitter, ErrorMessage, KiError, LexError, PResult},
    parse::*,
    source_map::{get_cursor, Span},
    stmt_local::StmtLocal,
    strnom::*,
};

pub type Ws = (bool, bool);

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(transparent)]
pub struct Local(#[serde(deserialize_with = "de_local")] syn::Local);

impl Deref for Local {
    type Target = syn::Local;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Local {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<syn::Local> for Local {
    fn as_ref(&self) -> &syn::Local {
        &self.0
    }
}

impl AsMut<syn::Local> for Local {
    fn as_mut(&mut self) -> &mut syn::Local {
        &mut self.0
    }
}

fn de_local<'de, D>(deserializer: D) -> Result<syn::Local, D::Error>
where
    D: Deserializer<'de>,
{
    <&str>::deserialize(deserializer).and_then(|x| {
        syn::parse_str::<StmtLocal>(x)
            .map(Into::into)
            .map_err(|_| serde::de::Error::custom("Parse error"))
    })
}

impl Parse for Local {
    fn parse(input: &ParseBuffer<'_>) -> syn::Result<Self> {
        Ok(Local(input.parse::<StmtLocal>()?.into()))
    }
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(transparent)]
pub struct Expr(#[serde(deserialize_with = "de_expr")] syn::Expr);

impl Deref for Expr {
    type Target = syn::Expr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Expr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<syn::Expr> for Expr {
    fn as_ref(&self) -> &syn::Expr {
        &self.0
    }
}

impl AsMut<syn::Expr> for Expr {
    fn as_mut(&mut self) -> &mut syn::Expr {
        &mut self.0
    }
}

fn de_expr<'de, D>(deserializer: D) -> Result<syn::Expr, D::Error>
where
    D: Deserializer<'de>,
{
    <&str>::deserialize(deserializer)
        .and_then(|x| syn::parse_str(x).map_err(|_| serde::de::Error::custom("Parse error")))
}

impl Parse for Expr {
    fn parse(input: &ParseBuffer<'_>) -> syn::Result<Self> {
        Ok(Expr(input.parse()?))
    }
}

pub type SArm = S<Box<Arm>>;
pub type SExpr = S<Box<Expr>>;
pub type SLocal = S<Box<Local>>;
pub type SToken<'a, Kind> = S<Token<'a, Kind>>;
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
        OPEN: Ascii
        CLOSE: Ascii
        OPEN_EXPR: Ascii
        CLOSE_EXPR: Ascii
        OPEN_BLOCK: Ascii
        CLOSE_BLOCK: Ascii
        WS: Ascii
        WS_AFTER: bool
    ;
        parse -> Self
        comment -> &'a str
);

// TODO: Visit trait
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum Token<'a, Kind>
where
    Kind: Kinder<'a>,
{
    Arm(Ws, SArm),
    ArmKind(Ws, Kind, SArm),
    Comment(#[serde(borrow)] &'a str),
    Safe(Ws, SExpr),
    Local(Ws, SLocal),
    Expr(Ws, SVExpr),
    ExprKind(Ws, Kind, SVExpr),
    Lit(
        #[serde(borrow)] &'a str,
        #[serde(borrow)] SStr<'a>,
        #[serde(borrow)] &'a str,
    ),
    Block(Ws, SVExpr),
    BlockKind(Ws, Kind, SVExpr),
    Error(SVExpr),
}
