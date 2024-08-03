#![allow(clippy::many_single_char_names, clippy::cognitive_complexity)]
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::rc::Rc;
use std::str;

use serde::{Deserialize, Deserializer};
use syn::parse::{Parse, ParseBuffer};

#[macro_use]
mod strnom;

#[cfg(test)]
mod test;

mod error;
mod expr_list;
mod parse;
pub mod source_map;
mod stmt_local;

use crate::source_map::S;

pub use self::{
    error::{emitter, ErrorMessage, MResult, PError},
    parse::*,
    stmt_local::StmtLocal,
    strnom::Cursor,
};

pub type OwnParsed = HashMap<Rc<Path>, (String, Vec<SNode<'static>>)>;
pub type Parsed<'a> = &'a OwnParsed;

pub type Ws = (bool, bool);

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
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

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
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

pub type SExpr = S<Box<Expr>>;
pub type SLocal = S<Box<Local>>;
pub type SNode<'a> = S<Node<'a>>;
pub type SStr<'a> = S<&'a str>;
pub type SVExpr = S<Vec<Expr>>;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
pub struct Partial<'a>(pub Ws, #[serde(borrow)] pub SStr<'a>, pub SVExpr);

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct PartialBlock<'a>(
    pub (Ws, Ws),
    #[serde(borrow)] pub SStr<'a>,
    pub SVExpr,
    #[serde(borrow)] pub Vec<SNode<'a>>,
);

// TODO: reduce size
#[derive(Debug, PartialEq, Clone, Deserialize)]
pub enum Node<'a> {
    Comment(#[serde(borrow)] &'a str),
    Expr(Ws, SExpr),
    AtHelper(Ws, AtHelperKind, SVExpr),
    RExpr(Ws, SExpr),
    Helper(#[serde(borrow)] Box<Helper<'a>>),
    Lit(
        #[serde(borrow)] &'a str,
        #[serde(borrow)] SStr<'a>,
        #[serde(borrow)] &'a str,
    ),
    Local(SLocal),
    Partial(#[serde(borrow)] Partial<'a>),
    PartialBlock(#[serde(borrow)] PartialBlock<'a>),
    Block(Ws),
    Raw(
        (Ws, Ws),
        #[serde(borrow)] &'a str,
        #[serde(borrow)] SStr<'a>,
        #[serde(borrow)] &'a str,
    ),
    Safe(Ws, SExpr),
    Error(SVExpr),
}

pub(crate) const JSON: &str = "json";
pub(crate) const JSON_PRETTY: &str = "json_pretty";
#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
pub enum AtHelperKind {
    Json,
    JsonPretty,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub enum Helper<'a> {
    Each((Ws, Ws), SExpr, #[serde(borrow)] Vec<SNode<'a>>),
    If(
        ((Ws, Ws), SExpr, Vec<SNode<'a>>),
        Vec<(Ws, SExpr, Vec<SNode<'a>>)>,
        Option<(Ws, Vec<SNode<'a>>)>,
    ),
    With((Ws, Ws), SExpr, #[serde(borrow)] Vec<SNode<'a>>),
    Unless((Ws, Ws), SExpr, #[serde(borrow)] Vec<SNode<'a>>),
    // TODO:
    Defined(
        (Ws, Ws),
        #[serde(borrow)] &'a str,
        SExpr,
        #[serde(borrow)] Vec<SNode<'a>>,
    ),
}
