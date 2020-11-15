#![allow(dead_code)]

use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::str;

use serde::{Deserialize, Deserializer};
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Pat, PatOr, Token};

#[macro_use]
mod strnom;
#[macro_use]
mod error;
mod expr_list;
mod parse;
mod source_map;
mod stmt_local;

use crate::source_map::S;

pub use gencode::asciis;

pub use self::{
    error::{emitter, ErrorMessage, KiError, LexError, PResult},
    parse::*,
    source_map::Span,
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

#[derive(Debug, PartialEq, Clone)]
pub struct Arm {
    pat: syn::Pat,
    guard: Option<(syn::token::If, Box<syn::Expr>)>,
    fat_arrow_token: syn::token::FatArrow,
}

impl<'de> Deserialize<'de> for Arm {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        <&str>::deserialize(deserializer)
            .and_then(|x| syn::parse_str(x).map_err(|_| serde::de::Error::custom("Parse error")))
    }
}

pub fn multi_pat_with_leading_vert(input: ParseStream) -> syn::Result<Pat> {
    let leading_vert: Option<Token![|]> = input.parse()?;
    multi_pat_impl(input, leading_vert)
}

// TODO: Pipes with |> or @ or another no rust token
// TODO: Pipes like tensors for avoid multiple reallocation
fn multi_pat_impl(input: ParseStream, leading_vert: Option<Token![|]>) -> syn::Result<Pat> {
    let mut pat: Pat = input.parse()?;
    if leading_vert.is_some()
        || input.peek(Token![|]) && !input.peek(Token![||]) && !input.peek(Token![|=])
    {
        let mut cases = Punctuated::new();
        cases.push_value(pat);
        while input.peek(Token![|]) && !input.peek(Token![||]) && !input.peek(Token![|=]) {
            let punct = input.parse()?;
            cases.push_punct(punct);
            let pat: Pat = input.parse()?;
            cases.push_value(pat);
        }
        pat = Pat::Or(PatOr {
            attrs: Vec::new(),
            leading_vert,
            cases,
        });
    }
    Ok(pat)
}

impl Parse for Arm {
    fn parse(input: ParseStream) -> syn::Result<Arm> {
        Ok(Arm {
            pat: multi_pat_with_leading_vert(input)?,
            guard: {
                if input.peek(Token![if]) {
                    let if_token: Token![if] = input.parse()?;
                    let guard: syn::Expr = input.parse()?;
                    Some((if_token, Box::new(guard)))
                } else {
                    None
                }
            },
            fat_arrow_token: input.parse()?,
        })
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
    Expr(Ws, SExpr),
    ExprList(Ws, SVExpr),
    ExprListKind(Ws, Kind, SVExpr),
    Kind(Ws, Kind),
    Lit(
        #[serde(borrow)] &'a str,
        #[serde(borrow)] SStr<'a>,
        #[serde(borrow)] &'a str,
    ),
    Block(Ws, Kind),
    BlockExpr(Ws, Kind, SVExpr),
    Error(SVExpr),
}
