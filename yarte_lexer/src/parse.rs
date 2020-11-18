use std::fmt::Debug;

use syn::parse_str;

use crate::error::{ErrorMessage, KiError, LexError, PResult};
use crate::expr_list::ExprList;
use crate::source_map::{Span, S};
use crate::strnom::{is_ws, Cursor};
use crate::{get_chars, tac, take_while, Kinder, SToken, StmtLocal, Token};

pub trait Ki<'a>: Kinder<'a> + Debug + PartialEq + Clone {}

impl<'a, T: Kinder<'a> + Debug + PartialEq + Clone> Ki<'a> for T {}

pub fn parse<'a, K: Ki<'a>>(i: Cursor<'a>) -> Result<Vec<SToken<'a, K>>, ErrorMessage<K::Error>> {
    let (c, res) = eat(i)?;
    if c.is_empty() {
        Ok(res)
    } else {
        Err(ErrorMessage {
            message: K::Error::UNCOMPLETED,
            span: Span::from_len(c, 1),
        })
    }
}

macro_rules! comment {
    ($K:ty, $cur:expr, $i:ident, $at:ident, $j:ident, $nodes:ident) => {
        match <$K>::comment($cur) {
            Ok((c, s)) => {
                eat_lit($i, $at + $j, &mut $nodes);
                $nodes.push(S(Token::Comment(s), Span::from_cursor($i.adv($at + $j), c)));
                $i = c;
                $at = 0;
                continue;
            }
            Err(LexError::Next(..)) => (),
            Err(e) => break Err(e),
        }
    };
}

macro_rules! safe {
    ($K:ty, $cur:expr, $i:ident, $at:ident, $j:ident, $nodes:ident) => {
        match safe::<$K>($cur) {
            Ok((c, token)) => {
                eat_lit($i, $at + $j, &mut $nodes);
                $nodes.push(S(token, Span::from_cursor($i.adv($at + $j), c)));
                $i = c;
                $at = 0;
                continue;
            }
            Err(LexError::Next(..)) => (),
            Err(e) => break Err(e),
        }
    };
}

fn eat<'a, K: Ki<'a>>(mut i: Cursor<'a>) -> PResult<Vec<SToken<'a, K>>, K::Error> {
    let mut nodes = vec![];
    let mut at = 0;

    loop {
        if let Some(j) = i.adv_find(at, K::OPEN) {
            let n = i.rest[at + j + 1..].as_bytes();
            if 3 < n.len() {
                macro_rules! inner {
                    ($f:expr, $next:ident, $is_expr:literal) => {
                        if let Ok((c, inner)) = end::<K>($next, $is_expr) {
                            eat_lit(i, at + j, &mut nodes);
                            nodes.push($f(inner).map(|x| S(x, Span::new($next.off - 2, c.off)))?);
                            at = 0;
                            i = c;
                        } else {
                            at += j + 1;
                        }
                    };
                }
                let next = n[0];
                if K::OPEN_BLOCK == K::OPEN_EXPR && next == K::OPEN_EXPR.g() {
                    let inner = |inner| {
                        eat_expr::<K>(inner).or_else(|pe| {
                            eat_block::<K>(inner).map_err(|e| match e {
                                LexError::Next(..) => pe,
                                e => e,
                            })
                        })
                    };
                    let next = i.adv(at + j + 2);
                    comment!(K, next, i, at, j, nodes);
                    safe!(K, next, i, at, j, nodes);
                    inner!(inner, next, true);
                } else if next == K::OPEN_EXPR.g() {
                    let next = i.adv(at + j + 2);
                    safe!(K, next, i, at, j, nodes);
                    inner!(eat_expr::<K>, next, true);
                } else if next == K::OPEN_BLOCK.g() {
                    let next = i.adv(at + j + 2);
                    comment!(K, next, i, at, j, nodes);
                    inner!(eat_block::<K>, next, false);
                } else {
                    at += j + 1;
                }
            } else {
                at += j + 1;
            };
        } else {
            eat_lit(i, i.len(), &mut nodes);
            break Ok((i.adv(i.len()), nodes));
        }
    }
}

/// Push literal at cursor with length
fn eat_lit<'a, K: Ki<'a>>(i: Cursor<'a>, len: usize, nodes: &mut Vec<SToken<'a, K>>) {
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
        nodes.push(S(Token::Lit(l, S(lit, ins), r), out));
    }
}

// TODO: local
// TODO: Arm
// TODO: Safe
// TODO: more todo
fn eat_expr<'a, K: Ki<'a>>(i: Cursor<'a>) -> Result<Token<'a, K>, LexError<K::Error>> {
    let (i, ws) = eat_ws::<K>(i)?;
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
                K::Error::expr(e.message),
                Span::new(init + e.span.0, init + e.span.1),
            )
        })?;

    if let Some(kind) = kind {
        Ok(Token::ExprKind(ws, kind, expr))
    } else {
        Ok(Token::Expr(ws, expr))
    }
}

fn eat_ws<'a, K: Ki<'a>>(i: Cursor) -> PResult<(bool, bool), K::Error> {
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

fn eat_block<'a, K: Ki<'a>>(_i: Cursor<'a>) -> Result<Token<'a, K>, LexError<K::Error>> {
    Err(next!(K::Error))
}

fn safe<'a, K: Ki<'a>>(_i: Cursor<'a>) -> PResult<Token<'a, K>, K::Error> {
    Err(next!(K::Error))
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
    take_while(i, |i| !is_ws(i)).and_then(|(c, s)| {
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
