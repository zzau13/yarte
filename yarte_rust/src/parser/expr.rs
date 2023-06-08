#![allow(dead_code, unused_variables)]
use std::marker::PhantomData;

use yarte_strnom::source_map::S;
use yarte_strnom::{Cursor, LexError, Span};

use crate::error;
use crate::error::Error;
use crate::lexer::token_stream;
use crate::parser::ast::expr::Expr;
use crate::parser::sinks::punct::{expr_punct, ExprPunct};
use crate::sink::{SResult, Sink, State};
use crate::tokens::*;

enum Ends {
    None,
    First,
}

pub struct ExprSink<'a, const END0: u8, const END1: u8> {
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

impl<'a, 'b, const END0: u8, const END1: u8> From<&'b mut ExprSink<'a, END0, END1>>
    for ExprPunct<'b>
{
    fn from(value: &'b mut ExprSink<'a, END0, END1>) -> Self {
        ExprPunct {
            expr: &mut value.expr,
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
        if self.groups.pop().map_or(false, |x| x == del.0) {
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
        let _ = expr_punct(punct, self.into())?;
        todo!("end before check")
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
