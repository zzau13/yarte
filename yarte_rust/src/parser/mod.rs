#![allow(dead_code, unused_variables)]
use crate::error;
use serde::Deserialize;
use std::marker::PhantomData;
use yarte_strnom::{Cursor, LexError, Span};

use crate::lexer::token_stream;
use crate::sink::{SResult, Sink};
use crate::token_types::*;

pub enum Expr {
    Array(ExprArray),
    Binary(ExprBinary),
    Index(ExprIndex),
    Lit(ExprLit),
    Paren(ExprParen),
    Range(ExprRange),
    Reference(ExprReference),
    Repeat(ExprRepeat),
    Tuple(ExprTuple),
    // TODO: check before template logic for completeness ?
    //  i.e, no use ?
    Try(ExprTry),
    Unary(ExprUnary),
}

pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    BitXor,
    BitAnd,
    BitOr,
    Shl,
    Shr,
    Eq,
    Lt,
    Le,
    Ne,
    Ge,
    Gt,
    AddEq,
    SubEq,
    MulEq,
    DivEq,
    RemEq,
    BitXorEq,
    BitAndEq,
    BitOrEq,
    ShlEq,
    ShrEq,
    Pipe,
}

/// A slice literal expression: `[a, b, c, d]`.
pub struct ExprArray {
    pub elems: Vec<Box<Expr>>,
}

/// An assignment expression: `a = compute()`.
pub struct ExprAssign {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

/// A compound assignment expression: `counter += 1`.
///
/// *This type is available only if Syn is built with the `"full"` feature.*
pub struct ExprAssignOp {
    pub left: Box<Expr>,
    pub op: BinOp,
    pub right: Box<Expr>,
}

/// A binary operation: `a + b`, `a * b`.
pub struct ExprBinary {
    pub left: Box<Expr>,
    pub op: BinOp,
    pub right: Box<Expr>,
}

/// A blocked scope: `{ ... }`.
pub struct ExprBlock {
    pub block: Vec<Box<Expr>>,
}

/// A `break`, with an optional label to break and an optional
/// expression.
pub struct ExprBreak {
    // pub label: Option<Lifetime>,
    pub expr: Option<Box<Expr>>,
}

/// A square bracketed indexing expression: `vector[2]`.
pub struct ExprIndex {
    pub expr: Box<Expr>,
    pub index: Box<Expr>,
}

/// A literal in place of an expression: `1`, `"foo"`.
pub struct ExprLit {
    pub lit: String,
}

/// A parenthesized expression: `(a + b)`.
pub struct ExprParen {
    pub expr: Box<Expr>,
}

/// A range expression: `1..2`, `1..`, `..2`, `1..=2`, `..=2`.
pub struct ExprRange {
    pub from: Option<Box<Expr>>,
    pub limits: RangeLimits,
    pub to: Option<Box<Expr>>,
}

/// A referencing operation: `&a` or `&mut a`.
pub struct ExprReference {
    pub mutability: bool,
    pub expr: Box<Expr>,
}

/// An array literal constructed from one repeated element: `[0u8; N]`.
pub struct ExprRepeat {
    pub expr: Box<Expr>,
    pub len: Box<Expr>,
}

/// A try-expression: `expr?`.
pub struct ExprTry {
    pub expr: Box<Expr>,
}

/// A tuple expression: `(a, b, c, d)`.
pub struct ExprTuple {
    pub elems: Vec<Box<Expr>>,
}

pub enum UnOp {
    Deref,
    Not,
    Neg,
}

/// A unary operation: `!x`, `*x`.
pub struct ExprUnary {
    pub op: UnOp,
    pub expr: Box<Expr>,
}

/// Limit types of a range, inclusive or exclusive.
///
/// *This type is available only if Syn is built with the `"full"` feature.*
pub enum RangeLimits {
    /// Inclusive at the beginning, exclusive at the end.
    HalfOpen, //..
    /// Inclusive at the beginning and end.
    Closed, // ..=
}
#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Token<'a> {
    OpenGroup(Delimiter),
    CloseGroup(Delimiter),
    #[serde(borrow)]
    Ident(Ident<'a>),
    Punct(Punct),
    #[serde(borrow)]
    Literal(Literal<'a>),
}

struct ExprSink<'a> {
    expr: Option<Expr>,
    _w: PhantomData<&'a ()>,
}

impl<'a> Default for ExprSink<'a> {
    fn default() -> Self {
        Self {
            expr: None,
            _w: PhantomData,
        }
    }
}

impl<'a> Sink<'a> for ExprSink<'a> {
    fn open_group(&mut self, del: Delimiter) -> SResult {
        todo!()
    }

    fn close_group(&mut self, del: Delimiter) -> SResult {
        todo!()
    }

    fn ident(&mut self, ident: Ident<'a>) -> SResult {
        todo!()
    }

    fn punct(&mut self, punct: Punct) -> SResult {
        todo!()
    }

    fn literal(&mut self, literal: Literal<'a>) -> SResult {
        todo!()
    }

    fn end(&mut self) -> SResult {
        todo!()
    }
}

pub fn expr(c: Cursor) -> error::Result<Expr> {
    let mut sink = ExprSink::default();
    let c = token_stream(c, &mut sink)?;

    Ok((
        c,
        sink.expr
            .ok_or(LexError::Next(error::Error::Uncompleted, Span::from(c)))?,
    ))
}
