use std::fmt::Debug;
use std::marker::PhantomData;

use syn::parse_str;

use gencode::unsafe_asciis;

use crate::arm::Arm;
use crate::error::{ErrorMessage, KiError, LexError, Result as PResult};
use crate::expr_list::ExprList;
use crate::source_map::{Span, S};
use crate::strnom::pipes::{is_some, opt};
use crate::strnom::{_while, get_chars, is_ws, tac, tag, ws, Cursor};
use crate::{Ascii, Kinder, SArm, SExpr, SLocal, SStr, SVExpr, StmtLocal, Ws};

pub trait Ki<'a>: Kinder<'a> + Debug + PartialEq + Clone {}

impl<'a, T: Kinder<'a> + Debug + PartialEq + Clone> Ki<'a> for T {}

pub struct Lexer<'a, K: Ki<'a>, S: Sink<'a, K>> {
    sink: S,
    _p: PhantomData<&'a K>,
}

pub type LexResult<E, O = ()> = Result<O, E>;

///
pub trait Sink<'a, K: Ki<'a>>: 'a {
    fn arm(&mut self, ws: Ws, arm: SArm, span: Span) -> LexResult<K::Error>;
    fn arm_kind(&mut self, ws: Ws, kind: K, arm: SArm, span: Span) -> LexResult<K::Error>;

    fn block(&mut self, ws: Ws, expr: SVExpr, span: Span) -> LexResult<K::Error>;
    fn block_kind(&mut self, ws: Ws, kind: K, expr: SVExpr, span: Span) -> LexResult<K::Error>;

    fn comment(&mut self, src: &'a str, span: Span) -> LexResult<K::Error>;

    fn error(&mut self, expr: SVExpr, span: Span) -> LexResult<K::Error>;

    fn expr(&mut self, ws: Ws, expr: SVExpr, span: Span) -> LexResult<K::Error>;
    fn expr_kind(&mut self, ws: Ws, kind: K, expr: SVExpr, span: Span) -> LexResult<K::Error>;

    fn lit(&mut self, left: &'a str, src: SStr<'a>, right: &'a str, span: Span);

    fn local(&mut self, ws: Ws, local: SLocal, span: Span) -> LexResult<K::Error>;

    fn safe(&mut self, ws: Ws, expr: SExpr, span: Span) -> LexResult<K::Error>;

    fn end(&mut self) -> LexResult<K::Error>;
}

macro_rules! comment {
    ($_self:ident, $K:ty, $cur:expr, $i:ident, $at:ident, $j:ident) => {
        match <$K>::comment($cur) {
            Ok((c, s)) => {
                $_self.eat_lit($i, $at + $j);
                let span = Span::from_cursor($i.adv($at + $j), c);
                $_self
                    .sink
                    .comment(s, span)
                    .map_err(|e| LexError::Fail(e, span))?;
                $i = c;
                $at = 0;
                continue;
            }
            Err(LexError::Next(..)) => (),
            Err(e) => break Err(e.into()),
        }
    };
}

macro_rules! safe {
    ($_self:ident, $K:ty, $cur:expr, $i:ident, $at:ident, $j:ident) => {
        match safe::<$K>($cur) {
            Ok((c, (ws, expr))) => {
                $_self.eat_lit($i, $at + $j);
                let span = Span::from_cursor($i.adv($at + $j), c);
                $_self
                    .sink
                    .safe(ws, expr, span)
                    .map_err(|e| LexError::Fail(e, span))?;
                $i = c;
                $at = 0;
                continue;
            }
            Err(LexError::Next(..)) => (),
            Err(e) => break Err(e.into()),
        }
    };
}

impl<'a, K: Ki<'a>, Si: Sink<'a, K>> Lexer<'a, K, Si> {
    pub fn new(sink: Si) -> Lexer<'a, K, Si> {
        Lexer {
            sink,
            _p: PhantomData,
        }
    }

    pub fn feed(mut self, mut i: Cursor<'a>) -> Result<Si, ErrorMessage<K::Error>> {
        let mut at = 0;
        loop {
            if let Some(j) = i.adv_find(at, K::OPEN) {
                let n = i.rest[at + j + 1..].as_bytes();
                if 3 < n.len() {
                    let next = n[0];

                    if K::OPEN_BLOCK == K::OPEN_EXPR && next == K::OPEN_EXPR.g() {
                        let next = i.adv(at + j + 2);
                        comment!(self, K, next, i, at, j);
                        safe!(self, K, next, i, at, j);
                        if let Ok((c, inner)) = end::<K>(next, true) {
                            self.eat_lit(i, at + j);
                            let span = Span::new(next.off - 2, c.off);
                            self.eat_expr(inner, span).or_else(|pe| {
                                self.eat_block(inner, span).map_err(|e| match e {
                                    LexError::Next(..) => pe,
                                    e => e,
                                })
                            })?;
                            at = 0;
                            i = c;
                        } else {
                            at += j + 1;
                        }
                    } else if next == K::OPEN_EXPR.g() {
                        let next = i.adv(at + j + 2);
                        comment!(self, K, next, i, at, j);
                        safe!(self, K, next, i, at, j);
                        if let Ok((c, inner)) = end::<K>(next, true) {
                            self.eat_lit(i, at + j);
                            let span = Span::new(next.off - 2, c.off);
                            self.eat_expr(inner, span)?;
                            at = 0;
                            i = c;
                        } else {
                            at += j + 1;
                        }
                    } else if next == K::OPEN_BLOCK.g() {
                        let next = i.adv(at + j + 2);
                        comment!(self, K, next, i, at, j);
                        if let Ok((c, inner)) = end::<K>(next, false) {
                            self.eat_lit(i, at + j);
                            let span = Span::new(next.off - 2, c.off);
                            self.eat_block(inner, span)?;
                            at = 0;
                            i = c;
                        } else {
                            at += j + 1;
                        }
                    } else {
                        at += j + 1;
                    }
                } else {
                    at += j + 1;
                };
            } else {
                self.eat_lit(i, i.len());
                self.sink
                    .end()
                    .map_err(|e| LexError::Fail(e, Span::from(i)))?;
                break Ok(self.sink);
            }
        }
    }

    /// Push literal at cursor with length
    fn eat_lit(&mut self, i: Cursor<'a>, len: usize) {
        let lit = &i.rest[..len];
        if !lit.is_empty() {
            let (l, lit, r) = trim(lit);
            let ins = Span {
                lo: i.off + l.len() as u32,
                hi: i.off + lit.len() as u32,
            };
            let out = Span {
                lo: i.off,
                hi: i.off + len as u32,
            };
            self.sink.lit(l, S(lit, ins), r, out)
        }
    }

    fn eat_expr(&mut self, i: Cursor<'a>, span: Span) -> Result<(), LexError<K::Error>> {
        const LET: &[Ascii] = unsafe { unsafe_asciis!("let ") };

        let (i, gws) = Self::eat_ws(i)?;
        if do_parse!(i, ws => tag::<K::Error>[LET] => ()).is_ok() {
            let (l, s, _) = trim(i.rest);
            let init = i.off + l.len() as u32;
            eat_local(s)
                .map_err(|e| {
                    LexError::Fail(
                        K::Error::string(e.message),
                        Span::new(init + e.span.0, init + e.span.1),
                    )
                })
                .and_then(|e| {
                    self.sink
                        .local(gws, S(e, Span::new(init, init + s.len() as u32)), span)
                        .map_err(|e| LexError::Fail(e, span))
                })
        } else {
            let (i, kind) = match K::parse(i) {
                Ok((c, kind)) => (c, Some(kind)),
                Err(LexError::Next(..)) => (i, None),
                Err(e @ LexError::Fail(..)) => return Err(e),
            };
            let (l, s, _) = trim(i.rest);
            let init = i.off + l.len() as u32;
            if let Ok(arm) = eat_arm(s) {
                let arm = S(arm, Span::new(init, init + s.len() as u32));
                return if let Some(kind) = kind {
                    self.sink
                        .arm_kind(gws, kind, arm, span)
                        .map_err(|e| LexError::Fail(e, span))
                } else {
                    self.sink
                        .arm(gws, arm, span)
                        .map_err(|e| LexError::Fail(e, span))
                };
            }
            let expr = eat_expr_list(s)
                .map(|e| S(e, Span::new(init, init + s.len() as u32)))
                .map_err(|e| {
                    LexError::Fail(
                        K::Error::string(e.message),
                        Span::new(init + e.span.0, init + e.span.1),
                    )
                })?;

            if let Some(kind) = kind {
                self.sink
                    .expr_kind(gws, kind, expr, span)
                    .map_err(|e| LexError::Fail(e, span))
            } else {
                self.sink
                    .expr(gws, expr, span)
                    .map_err(|e| LexError::Fail(e, span))
            }
        }
    }

    fn eat_ws(i: Cursor) -> PResult<(bool, bool), K::Error> {
        let (i, lws) = match tac::<K::Error>(i, K::WS) {
            Ok((c, _)) => (c, true),
            _ => (i, false),
        };
        if i.is_empty() {
            return Err(LexError::Next(K::Error::WHITESPACE, Span::from(i)));
        }
        let (rest, rws) = match tac::<K::Error>(i.adv(i.len() - 1), K::WS) {
            Ok(_) => (&i.rest[..i.len() - 1], true),
            _ => (i.rest, false),
        };

        Ok((Cursor { rest, off: i.off }, (lws, rws)))
    }

    fn eat_block(&mut self, i: Cursor<'a>, span: Span) -> Result<(), LexError<K::Error>> {
        let (i, gws) = Self::eat_ws(i)?;
        let (i, kind) = match K::parse(i) {
            Ok((c, kind)) => (c, Some(kind)),
            Err(LexError::Next(..)) => (i, None),
            Err(e @ LexError::Fail(..)) => return Err(e),
        };
        let (l, s, _) = trim(i.rest);
        let init = i.off + l.len() as u32;
        let expr = eat_expr_list(s)
            .map(|e| S(e, Span::new(init, init + s.len() as u32)))
            .map_err(|e| {
                LexError::Fail(
                    K::Error::string(e.message),
                    Span::new(init + e.span.0, init + e.span.1),
                )
            })?;

        if let Some(kind) = kind {
            self.sink
                .block_kind(gws, kind, expr, span)
                .map_err(|e| LexError::Fail(e, span))
        } else {
            self.sink
                .block(gws, expr, span)
                .map_err(|e| LexError::Fail(e, span))
        }
    }
}

// TODO: check rust token groups and LitStr, LitChar, LitBytes
#[inline]
fn end_safe_after<'a, K: Ki<'a>>(i: Cursor<'a>) -> PResult<(Cursor, bool), K::Error> {
    let ws_end = &[K::WS, K::CLOSE_EXPR, K::CLOSE];
    let end = &[K::CLOSE_EXPR, K::CLOSE];

    let mut at = 0;

    loop {
        if let Some(j) = i.adv_find(at, K::CLOSE_EXPR) {
            if 0 < at + j && i.adv_starts_with(at + j - 1, ws_end) {
                let next = i.adv(at + j - 1 + ws_end.len());
                let cur = Cursor::_new(&i.rest[..at + j - 2], i.off);
                break Ok((next, (cur, true)));
            } else if i.adv_starts_with(at + j, end) {
                let next = i.adv(at + j + 1 + end.len());
                let cur = Cursor::_new(&i.rest[..at + j], i.off);
                break Ok((next, (cur, false)));
            }

            at += j + 1;
        } else {
            break Err(LexError::Next(
                K::Error::UNCOMPLETED,
                Span::from_cursor(i, i.adv(at)),
            ));
        }
    }
}

// TODO: check rust token groups and LitStr, LitChar, LitBytes
#[inline]
fn end_safe<'a, K: Ki<'a>>(i: Cursor<'a>) -> PResult<(Cursor, bool), K::Error> {
    let ws_end = &[K::WS, K::CLOSE_EXPR, K::CLOSE_EXPR, K::CLOSE];
    let end = &[K::CLOSE_EXPR, K::CLOSE_EXPR, K::CLOSE];

    let mut at = 0;

    loop {
        if let Some(j) = i.adv_find(at, K::CLOSE_EXPR) {
            if 0 < at + j && i.adv_starts_with(at + j - 1, ws_end) {
                let next = i.adv(at + j - 1 + ws_end.len());
                let cur = Cursor::_new(&i.rest[..at + j - 1], i.off);
                break Ok((next, (cur, true)));
            } else if i.adv_starts_with(at + j, end) {
                let next = i.adv(at + j + end.len());
                let cur = Cursor::_new(&i.rest[..at + j], i.off);
                break Ok((next, (cur, false)));
            }

            at += j + 1;
        } else {
            break Err(LexError::Next(
                K::Error::UNCOMPLETED,
                Span::from_cursor(i, i.adv(at)),
            ));
        }
    }
}

#[inline]
fn safe<'a, K: Ki<'a>>(i: Cursor<'a>) -> PResult<(Ws, SExpr), K::Error> {
    let (c, (i, ws)) = if K::WS_AFTER {
        do_parse!(i,
            lws= tac[K::WS]:opt:is_some =>
            tac[K::OPEN_EXPR]           =>
            end= end_safe_after::<K>    =>
            ((end.0, (lws, end.1)))
        )?
    } else {
        do_parse!(i,
            tac[K::OPEN_EXPR]           =>
            lws= tac[K::WS]:opt:is_some =>
            end= end_safe::<K>          =>
            ((end.0, (lws, end.1)))
        )?
    };

    let (l, s, _) = trim(i.rest);
    let init = i.off + l.len() as u32;

    let expr = eat_expression(s)
        .map(|e| S(e, Span::new(init, init + s.len() as u32)))
        .map_err(|e| {
            LexError::Fail(
                K::Error::string(e.message),
                Span::new(init + e.span.0, init + e.span.1),
            )
        })?;

    Ok((c, (ws, expr)))
}

/// Intermediate error representation
#[derive(Debug)]
pub(crate) struct MiddleError {
    pub message: String,
    pub span: (u32, u32),
}

fn get_line_offset(src: &str, line_num: usize, column: usize) -> usize {
    assert!(1 < line_num);
    let mut line_num = line_num - 1;
    let mut prev = 0;
    while let Some(len) = src[prev..].find('\n') {
        prev += len + 1;
        line_num -= 1;
        if line_num == 0 {
            break;
        }
    }
    assert_eq!(line_num, 0);

    prev + get_chars(&src[prev..], 0, column).len()
}

impl MiddleError {
    fn new(src: &str, e: syn::Error) -> Self {
        let start = e.span().start();
        let end = e.span().end();
        let lo = if start.line == 1 {
            get_chars(src, 0, start.column).len()
        } else {
            get_line_offset(src, start.line, start.column)
        };
        let hi = if end.line == 1 {
            end.column
        } else {
            get_line_offset(src, end.line, end.column)
        };
        Self {
            message: e.to_string(),
            span: (lo as u32, hi as u32),
        }
    }
}

/// Parse Arm
fn eat_arm(i: &str) -> Result<Box<Arm>, MiddleError> {
    parse_str::<Arm>(i)
        .map(Box::new)
        .map_err(|e| MiddleError::new(i, e))
}

/// Parse syn local
fn eat_local(i: &str) -> Result<Box<crate::Local>, MiddleError> {
    parse_str::<StmtLocal>(i)
        .map(Into::into)
        .map(Box::new)
        .map_err(|e| MiddleError::new(i, e))
}

/// Parse syn expression comma separated list
pub(crate) fn eat_expr_list(i: &str) -> Result<Vec<crate::Expr>, MiddleError> {
    parse_str::<ExprList>(i)
        .map(Into::into)
        .map_err(|e| MiddleError::new(i, e))
}

/// Parse syn expression comma separated list
pub(crate) fn eat_expression(i: &str) -> Result<Box<crate::Expr>, MiddleError> {
    parse_str::<crate::Expr>(i)
        .map(Box::new)
        .map_err(|e| MiddleError::new(i, e))
}

// TODO: check rust token groups and LitStr, LitChar, LitBytes
#[inline]
fn end<'a, K: Ki<'a>>(i: Cursor<'a>, expr: bool) -> PResult<Cursor<'a>, K::Error> {
    let mut at = 0;
    loop {
        if let Some(j) = i.adv_find(at, if expr { K::CLOSE_EXPR } else { K::CLOSE_BLOCK }) {
            if i.adv_next_is(at + j + 1, K::CLOSE) {
                let inner = Cursor {
                    rest: &i.rest[..at + j],
                    off: i.off,
                };
                break Ok((i.adv(at + j + 2), inner));
            } else {
                at += j + 1;
            }
        } else {
            break Err(LexError::Next(K::Error::UNCOMPLETED, Span::from(i)));
        }
    }
}

/// TODO: Define chars in path
/// Eat path at partial
/// Next white space close path
pub fn path<E: KiError>(i: Cursor) -> PResult<&str, E> {
    _while(i, |i| !is_ws(i)).and_then(|(c, s)| {
        if s.is_empty() {
            Err(LexError::Fail(E::PATH, Span::from(c)))
        } else {
            Ok((c, s))
        }
    })
}

pub fn trim(i: &str) -> (&str, &str, &str) {
    if i.is_empty() {
        return ("", "", "");
    }

    if let Some(ln) = i.bytes().position(|x| !is_ws(x)) {
        let rn = i.bytes().rposition(|x| !is_ws(x)).unwrap();

        (&i[..ln], &i[ln..=rn], &i[rn + 1..])
    } else {
        (i, "", "")
    }
}
