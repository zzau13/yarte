use std::fmt::Debug;

use syn::parse_str;

use crate::error::{ErrorMessage, KiError, LexError, PResult};
use crate::expr_list::ExprList;
use crate::source_map::{Span, S};
use crate::strnom::{is_ws, Cursor};
use crate::{take_while, Kinder, SToken, StmtLocal, Token};

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

fn eat<'a, K: Ki<'a>>(mut i: Cursor<'a>) -> PResult<Vec<SToken<'a, K>>, K::Error> {
    let mut nodes = vec![];
    let mut at = 0;
    loop {
        if let Some(j) = i.adv_find(at, K::OPEN) {
            let n = i.rest[at + j + 1..].as_bytes();
            if 3 < n.len() {
                let next = n[0];
                if K::OPEN_BLOCK == K::OPEN_EXPR && next == K::OPEN_EXPR.g() {
                    let next = i.adv(at + j + 2);
                    comment!(K, next, i, at, j, nodes);
                    if let Ok((c, inner)) = end::<K>(next, true) {
                        eat_lit(i, at + j, &mut nodes);
                        nodes.push(
                            eat_expr::<K>(inner, c.off - 2)
                                .or_else(|_| eat_block::<K>(inner))
                                .map(|x| S(x, Span::new(next.off - 2, c.off)))?,
                        );
                        at = 0;
                        i = c;
                        continue;
                    } else {
                        eat_lit(i, i.len(), &mut nodes);
                        break Ok((i.adv(i.len()), nodes));
                    }
                } else if next == K::OPEN_EXPR.g() {
                    let next = i.adv(at + j + 2);
                    if let Ok((c, inner)) = end::<K>(next, true) {
                        eat_lit(i, at + j, &mut nodes);
                        nodes.push(
                            eat_expr::<K>(inner, c.off - 2)
                                .map(|x| S(x, Span::new(next.off - 2, c.off)))?,
                        );
                        at = 0;
                        i = c;
                        continue;
                    } else {
                        eat_lit(i, i.len(), &mut nodes);
                        break Ok((i.adv(i.len()), nodes));
                    }
                } else if next == K::OPEN_BLOCK.g() {
                    let next = i.adv(at + j + 2);
                    comment!(K, i.adv(at + j + 2), i, at, j, nodes);
                    if let Ok((c, inner)) = end::<K>(next, false) {
                        eat_lit(i, at + j, &mut nodes);
                        nodes.push(
                            eat_block::<K>(inner).map(|x| S(x, Span::new(next.off - 2, c.off)))?,
                        );
                        at = 0;
                        i = c;
                        continue;
                    } else {
                        eat_lit(i, i.len(), &mut nodes);
                        break Ok((i.adv(i.len()), nodes));
                    }
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
        let lit_len = lit.chars().count();
        let ins = Span {
            lo: i.off + l.len() as u32,
            hi: i.off + lit_len as u32,
        };
        let out = Span {
            lo: i.off,
            hi: i.off + (l.len() + lit_len + r.len()) as u32,
        };
        nodes.push(S(Token::Lit(l, S(lit, ins), r), out));
    }
}

// TODO: whitespace
// TODO: Kind
// TODO: local
// TODO: Arm
// TODO: Safe
// TODO: more todo
fn eat_expr<'a, K: Ki<'a>>(i: Cursor<'a>, end: u32) -> Result<Token<'a, K>, LexError<K::Error>> {
    let (l, s, r) = trim(i.rest);
    let init = i.off + l.len() as u32;
    eat_expr_list(s)
        .map(|e| Token::<K>::Expr((false, false), S(e, Span::new(init, end - r.len() as u32))))
        .map_err(|e| LexError::Fail(K::Error::EXPR, Span::new(init + e.span.0, init + e.span.1)))
}

fn eat_block<'a, K: Ki<'a>>(_i: Cursor<'a>) -> Result<Token<'a, K>, LexError<K::Error>> {
    unimplemented!()
}

/// Intermediate error representation
#[derive(Debug)]
pub(crate) struct MiddleError {
    pub message: String,
    pub span: (u32, u32),
}

fn get_line_offset(src: &str, line_num: usize) -> usize {
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

    prev
}

impl MiddleError {
    fn new(src: &str, e: syn::Error) -> Self {
        let start = e.span().start();
        let end = e.span().end();
        let lo = if start.line == 1 {
            start.column
        } else {
            get_line_offset(src, start.line) + start.column
        };
        let hi = if end.line == 1 {
            end.column
        } else {
            get_line_offset(src, end.line) + end.column
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
fn path<E: KiError>(i: Cursor) -> PResult<&str, E> {
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

    let b = i.as_bytes();

    if let Some(ln) = b.iter().position(|x| !is_ws((*x).into())) {
        let rn = b.iter().rposition(|x| !is_ws((*x).into())).unwrap();
        (
            safe_utf8(&b[..ln]),
            safe_utf8(&b[ln..=rn]),
            safe_utf8(&b[rn + 1..]),
        )
    } else {
        (i, "", "")
    }
}

/// Convert from bytes to str
/// Use when previous check bytes it's valid utf8
fn safe_utf8(s: &[u8]) -> &str {
    unsafe { ::std::str::from_utf8_unchecked(s) }
}
