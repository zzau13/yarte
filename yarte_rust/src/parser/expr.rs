#![allow(dead_code, unused_variables)]
use serde::Deserialize;
use std::marker::PhantomData;
use yarte_strnom::source_map::S;
use yarte_strnom::{Cursor, LexError, Span};

use crate::error;
use crate::error::Error;
use crate::lexer::token_stream;
use crate::sink::{SResult, Sink, State};
use crate::tokens::*;

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
    /// A function call expression: `invoke(a, b)`.
    Call(ExprCall),
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
    pub elems: Vec<Expr>,
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

pub struct ExprCall {
    pub func: Box<Expr>,
    pub args: Vec<Expr>,
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
    pub elems: Vec<Expr>,
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

enum Ends {
    None,
    First,
}

struct ExprSink<'a, const END0: u8, const END1: u8> {
    ends: Ends,
    expr: Option<Expr>,
    groups: Vec<Delimiter>,
    _w: PhantomData<&'a ()>,
}

impl<'a, const END0: u8, const END1: u8> Default for ExprSink<'a, END0, END1> {
    #[inline]
    fn default() -> Self {
        Self {
            ends: Ends::None,
            expr: None,
            groups: Vec::with_capacity(32),
            _w: PhantomData,
        }
    }
}

// TODO: add terminate expression
impl<'a, const END0: u8, const END1: u8> Sink<'a> for ExprSink<'a, END0, END1> {
    #[inline]
    fn open_group(&mut self, del: S<Delimiter>) -> SResult {
        self.groups.push(del.0);
        Ok(State::Continue)
    }

    #[inline]
    fn close_group(&mut self, del: S<Delimiter>) -> SResult {
        if self
            .groups
            .last()
            .copied()
            .map(|x| x == del.0)
            .unwrap_or(false)
        {
            self.groups.pop();
            todo!();
        } else {
            // TODO: check end
            // TODO: correct error
            Err(LexError::Fail(Error::Empty, del.span()))
        }
    }

    #[inline]
    fn ident(&mut self, ident: S<Ident<'a>>) -> SResult {
        todo!()
    }

    #[inline]
    fn punct(&mut self, punct: S<Punct>) -> SResult {
        // TODO: check end
        // TODO: punct is ASCII, add enum repr u8
        todo!()
    }

    #[inline]
    fn literal(&mut self, literal: S<Literal<'a>>) -> SResult {
        // TODO: select between literal in
        todo!()
    }

    #[inline]
    fn end(&mut self) -> SResult {
        todo!()
    }
}

// TODO: end: use adt experimental or use tuple or
pub fn expr<const END0: u8, const END1: u8>(c: Cursor) -> error::Result<Expr> {
    let mut sink = ExprSink::<END0, END1>::default();
    let c = token_stream(c, &mut sink)?;

    Ok((
        c,
        sink.expr
            .ok_or(LexError::Next(error::Error::Uncompleted, Span::from(c)))?,
    ))
}
