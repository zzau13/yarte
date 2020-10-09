#![allow(clippy::many_single_char_names, clippy::cognitive_complexity)]
use std::ops::{Deref, DerefMut};
use std::str;

use serde::{Deserialize, Deserializer};
use syn::parse::{Parse, ParseBuffer};
use syn::parse_str;
use unicode_xid::UnicodeXID;

#[cfg(test)]
mod test;

mod error;
mod expr_list;
mod pre_partials;
pub mod source_map;
mod stmt_local;
#[macro_use]
mod strnom;

use self::{
    error::{DOption, PError},
    expr_list::ExprList,
    source_map::{spanned, Span, S},
    strnom::{is_ws, skip_ws, ws, LexError, PResult},
};

pub use self::{
    error::{emitter, ErrorMessage},
    pre_partials::parse_partials,
    stmt_local::StmtLocal,
    strnom::Cursor,
};

pub type Ws = (bool, bool);

#[derive(Debug, PartialEq, Clone, Deserialize)]
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

#[derive(Debug, PartialEq, Clone, Deserialize)]
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

const JSON: &str = "json";
const JSON_PRETTY: &str = "json_pretty";
#[derive(Debug, PartialEq, Clone, Deserialize)]
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

pub fn parse(i: Cursor) -> Result<Vec<SNode>, ErrorMessage<PError>> {
    let (c, res) = eat(i)?;
    if c.is_empty() {
        Ok(res)
    } else {
        Err(ErrorMessage {
            message: PError::Uncompleted,
            span: Span::from_len(c, 1),
        })
    }
}

/// Step in eater
///     - Ok -> eat_lit -> push node -> restart in next cursor and continue
///     - Err(Next) -> advance
///     - Err(Fail) -> Insuperable error stop
macro_rules! try_eat {
    ($nodes:ident, $i:ident, $at:ident, $j:ident, $($t:tt)+) => {
        match $($t)+ {
            Ok((c, n)) => {
                eat_lit(&mut $nodes, $i, $at + $j);
                $nodes.push(S(n, Span::from_cursor($i.adv($at + $j), c)));
                $i = c;
                0
            },
            Err(e @ LexError::Fail(..)) => break Err(e),
            Err(LexError::Next(..)) => $at + $j + 1,
        }
    };
}

/// Exit of eater with ok in the current cursor
macro_rules! kill {
    ($nodes:ident, $c:expr, $i:expr, $len:expr) => {{
        eat_lit(&mut $nodes, $i, $len);
        break Ok(($c, $nodes));
    }};
}

/// Eater builder
///
/// $callback: macro for special expressions like {{ else if }}
macro_rules! make_eater {
    ($name:ident, $callback:ident) => {
        fn $name(mut i: Cursor) -> PResult<Vec<SNode>> {
            let mut buf = vec![];
            let mut at = 0;

            loop {
                if let Some(j) = i.adv_find(at, '{') {
                    macro_rules! _switch {
                        ($n:expr, $t:expr, $ws:expr) => {
                            match $n {
                                b'{' => try_eat!(buf, i, at, j, safe(i.adv(at + j + 3 + $t), $ws)),
                                b'!' => try_eat!(buf, i, at, j, comment(i.adv(at + j + 3 + $t))),
                                b'?' => try_eat!(buf, i, at, j, res(i.adv(at + j + 3 + $t), $ws)),
                                b'#' => try_eat!(buf, i, at, j, hel(i.adv(at + j + 3 + $t), $ws)),
                                b'>' => try_eat!(buf, i, at, j, par(i.adv(at + j + 3 + $t), $ws)),
                                b'R' => try_eat!(buf, i, at, j, raw(i.adv(at + j + 3 + $t), $ws)),
                                b'$' => try_eat!(buf, i, at, j, error(i.adv(at + j + 3 + $t))),
                                b'/' => kill!(buf, i.adv(at + j + 2), i, at + j),
                                _ => {
                                    $callback!(buf, i, at, j, $t, $ws);
                                    try_eat!(buf, i, at, j, expr(i.adv(at + j + 2 + $t), $ws))
                                }
                            }
                        };
                    }

                    let n = &i.rest[at + j + 1..].as_bytes();
                    if 2 < n.len() {
                        at = if n[0] == b'{' {
                            if n[1] == b'~' {
                                _switch!(n[2], 1, true)
                            } else {
                                _switch!(n[1], 0, false)
                            }
                        } else {
                            // next
                            at + j + 1
                        }
                    } else {
                        at += j + 1;
                    };
                } else {
                    kill!(buf, i.adv(i.len()), i, i.len());
                }
            }
        }
    };
}

// Empty macro for use when build eater
macro_rules! non {
    ($($t:tt)*) => {};
}

// Main eater
make_eater!(eat, non);

const IF: &str = "if";
const ELSE: &str = "else";

// Test special expression `{{ else ..` and kill eater at next brackets
macro_rules! is_else {
    ($buf:ident, $i:ident, $at:ident, $j:ident, $t:expr, $ws:expr) => {
        if skip_ws($i.adv($at + $j + 2 + $t)).starts_with(ELSE) {
            kill!($buf, $i.adv($at + $j + 2), $i, $at + $j);
        }
    };
}

// If else branch eater
make_eater!(eat_if, is_else);

const PARTIAL_BLOCK: &str = "@partial-block";
fn expr_partial_block(c: Cursor, lws: bool) -> PResult<Node> {
    do_parse!(
        c,
        ws >> tag!(PARTIAL_BLOCK) >> rws: end_expr >> (Node::Block((lws, rws)))
    )
}

/// Push literal at cursor with length
fn eat_lit<'a>(nodes: &mut Vec<SNode<'a>>, i: Cursor<'a>, len: usize) {
    let lit = &i.rest[..len];
    if !lit.is_empty() {
        let (l, lit, r) = trim(lit);
        let ins = Span {
            lo: i.off + (l.len() as u32),
            hi: i.off + ((len - r.len()) as u32),
        };
        let out = Span {
            lo: i.off,
            hi: i.off + (len as u32),
        };
        nodes.push(S(Node::Lit(l, S(lit, ins), r), out));
    }
}

/// Eat comment
fn comment(i: Cursor) -> PResult<Node> {
    let (c, expected) = if i.starts_with("--") {
        (i.adv(2), "--!}}")
    } else {
        (i, "!}}")
    };

    let ch = expected.chars().next().unwrap();
    let rest = &expected[1..];
    let mut at = 0;
    loop {
        if let Some(j) = c.adv_find(at, ch) {
            if c.adv_starts_with(at + j + 1, rest) {
                break Ok((
                    c.adv(at + j + expected.len()),
                    Node::Comment(&c.rest[..at + j]),
                ));
            } else {
                at += j + 1;
            }
        } else {
            break Err(LexError::Next(PError::Comment, Span::from_cursor(i, c)));
        }
    }
}

fn res(c: Cursor, lws: bool) -> PResult<Node> {
    do_parse!(
        c,
        ws >> expr: arguments >> rws: end_expr >> (Node::RExpr((lws, rws), expr))
    )
}

/// Wrap Partial into the Node
#[inline]
fn par(i: Cursor, lws: bool) -> PResult<Node> {
    match expr_partial_block(i, lws) {
        Ok(x) => Ok(x),
        Err(_) => partial(i, lws).map(|(c, p)| (c, Node::Partial(p))),
    }
}

/// Eat partial expression
#[inline]
fn partial(i: Cursor, lws: bool) -> PResult<Partial> {
    do_parse!(
        i,
        ws >> ident: call!(spanned, path)
            >> args: args_list
            >> rws: end_expr
            >> (Partial((lws, rws), ident, args))
    )
}

fn partial_block(i: Cursor, a_lws: bool) -> PResult<PartialBlock> {
    let (c, (ws, ident, args, block, c_ident)) = do_parse!(
        i,
        ws >> ident: call!(spanned, path)
            >> args: args_list
            >> a_rws: end_expr
            >> block: eat
            >> lws: opt!(tag!("~"))
            >> tag!("/")
            >> ws
            >> c_ident: call!(spanned, path)
            >> rws: end_expr
            >> (
                ((a_lws, a_rws), (lws.is_some(), rws)),
                ident,
                args,
                block,
                c_ident
            )
    )?;

    if ident.t() == c_ident.t() {
        Ok((c, PartialBlock(ws, ident, args, block)))
    } else {
        Err(LexError::Fail(
            PError::PartialBlock,
            Span::from_cursor(i, c),
        ))
    }
}

/// Eat helper Node
fn hel(i: Cursor, a_lws: bool) -> PResult<Node> {
    if i.starts_with(">") {
        return partial_block(i.adv(1), a_lws).map(|(c, x)| (c, Node::PartialBlock(x)));
    }

    let (i, (above_ws, ident, args)) = do_parse!(
        i,
        ws >> ident: call!(spanned, identifier)
            >> args: arguments
            >> rws: end_expr
            >> (((a_lws, rws), ident, args))
    )?;

    if ident.0.eq("if") {
        return if_else(above_ws, i, args);
    }

    let (c, (below_ws, block, c_ident)) = do_parse!(
        i,
        block: eat
            >> lws: opt!(tag!("~"))
            >> tag!("/")
            >> ws
            >> c_ident: call!(spanned, identifier)
            >> rws: end_expr
            >> (((lws.is_some(), rws), block, c_ident))
    )?;

    if ident.0.eq(c_ident.0) {
        Ok((
            c,
            Node::Helper(Box::new({
                match ident.0 {
                    "each" => Helper::Each((above_ws, below_ws), args, block),
                    "with" => Helper::With((above_ws, below_ws), args, block),
                    "unless" => Helper::Unless((above_ws, below_ws), args, block),
                    defined => Helper::Defined((above_ws, below_ws), defined, args, block),
                }
            })),
        ))
    } else {
        Err(LexError::Fail(PError::Helpers, Span::from_cursor(i, c)))
    }
}

/// Eat if else Node
#[inline]
fn if_else(abode_ws: Ws, i: Cursor, args: SExpr) -> PResult<Node> {
    let mut nodes = vec![];
    let mut tail = None;

    let (mut i, first) = eat_if(i)?;

    loop {
        if let Ok((c, lws)) = do_parse!(
            i,
            lws: opt!(tag!("~")) >> ws >> tag!(ELSE) >> (lws.is_some())
        ) {
            if let Ok((c, _)) = tag!(skip_ws(c), IF) {
                let (c, b) = map_fail!(do_parse!(
                    c,
                    ws >> args: arguments
                        >> rws: end_expr
                        >> block: eat_if
                        >> (((lws, rws), args, block))
                ))?;
                nodes.push(b);
                i = c;
            } else {
                let (c, b) = map_fail!(do_parse!(
                    c,
                    rws: end_expr >> block: eat >> (((lws, rws), block))
                ))?;
                tail = Some(b);
                i = c;
            }
        } else if let Ok((c, lws)) = do_parse!(
            i,
            lws: opt!(tag!("~")) >> tag!("/") >> ws >> tag!(IF) >> (lws.is_some())
        ) {
            let (c, rws) = end_expr(c)?;

            break Ok((
                c,
                Node::Helper(Box::new(Helper::If(
                    ((abode_ws, (lws, rws)), args, first),
                    nodes,
                    tail,
                ))),
            ));
        } else {
            break Err(LexError::Fail(PError::IfElse, Span::from(i)));
        }
    }
}

/// Eat error Node
fn error(i: Cursor) -> PResult<Node> {
    do_parse!(i, ws >> args: args_list >> end_expr >> (Node::Error(args)))
}

/// Eat raw Node
fn raw(i: Cursor, a_lws: bool) -> PResult<Node> {
    let (i, a_rws) = end_expr(i)?;
    let mut at = 0;

    let (c, (j, b_ws)) = loop {
        if let Some(j) = i.adv_find(at, '{') {
            let n = i.adv(at + j + 1);
            if n.chars().next().map(|x| '{' == x).unwrap_or(false) {
                if let Ok((c, ws)) = do_parse!(
                    n.adv(1),
                    lws: opt!(tag!("~")) >> tag!("/R") >> rws: end_expr >> ((lws.is_some(), rws))
                ) {
                    break (c, (&i.rest[..at + j], ws));
                } else {
                    at += j + 4;
                }
            } else {
                at += j + 1;
            }
        } else {
            return Err(LexError::Fail(PError::Raw, Span::from(i)));
        }
    };

    let (l, v, r) = trim(j);
    let lo = i.off + (l.len() as u32);
    let hi = lo + (v.len() as u32);
    Ok((
        c,
        Node::Raw(((a_lws, a_rws), b_ws), l, S(v, Span { lo, hi }), r),
    ))
}

/// Arguments builder
macro_rules! make_argument {
    ($name:ident, $fun:ident, $ret:ty) => {
        fn $name(i: Cursor) -> $ret {
            let mut at = 0;
            loop {
                if let Some(j) = i.adv_find(at, '}') {
                    macro_rules! cl {
                        ($s:ident, $d:expr) => {
                            $fun($s)
                                .map(|x| {
                                    (
                                        i.adv(at + j - $d),
                                        S(x, Span::from_len(skip_ws(i), $s.len())),
                                    )
                                })
                                .map_err(|e| {
                                    LexError::Fail(
                                        PError::Argument(DOption::Some(e.message)),
                                        Span::from_range(skip_ws(i), e.span),
                                    )
                                });
                        };
                    }
                    if 0 < j && i.adv_starts_with(at + j - 1, "~}}") {
                        let (_, s, _) = trim(&i.rest[..j - 1]);
                        break cl!(s, 1);
                    } else if i.adv_starts_with(j + 1, "}") {
                        let (_, s, _) = trim(&i.rest[..j]);
                        break cl!(s, 0);
                    }

                    at += j + 1;
                } else {
                    break Err(LexError::Next(
                        PError::EndExpression,
                        Span::from_cursor(i, i),
                    ));
                }
            }
        }
    };
}

// Eat arguments at helpers
make_argument!(arguments, eat_expr, PResult<SExpr>);

// Eat arguments at partials
make_argument!(args_list, eat_expr_list, PResult<SVExpr>);

/// Eat safe Node
fn safe(i: Cursor, lws: bool) -> PResult<Node> {
    let mut at = 0;
    let (c, rws, s) = loop {
        if let Some(j) = i.adv_find(at, '}') {
            let n = &i.rest[at + j + 1..];
            if n.starts_with("~}}") {
                break (i.adv(at + j + 4), true, &i.rest[..at + j]);
            } else if n.starts_with("}}") {
                break (i.adv(at + j + 3), false, &i.rest[..at + j]);
            }

            at += j + 1;
        } else {
            return Err(LexError::Next(PError::Safe(DOption::None), Span::from(i)));
        }
    };

    let (_, s, _) = trim(s);
    eat_expr(s)
        .map(|e| {
            (
                c,
                Node::Safe((lws, rws), S(e, Span::from_len(skip_ws(i), s.len()))),
            )
        })
        .map_err(|e| {
            LexError::Fail(
                PError::Safe(DOption::Some(e.message)),
                Span::from_range(skip_ws(i), e.span),
            )
        })
}

#[inline]
fn at_helper(i: Cursor, lws: bool) -> PResult<Node> {
    let (c, (name, args, rws)) = do_parse!(
        i,
        ws >> tag!("@")
            >> name: call!(spanned, identifier)
            >> args: args_list
            >> rws: end_expr
            >> ((name, args, rws))
    )?;

    macro_rules! check_args_len {
        ($len:expr) => {
            if args.t().len() != $len {
                return Err(LexError::Fail(PError::AtHelperArgsLen($len), args.span()));
            }
        };
    }
    match *name.t() {
        JSON => {
            check_args_len!(1);
            Ok((c, Node::AtHelper((lws, rws), AtHelperKind::Json, args)))
        }
        JSON_PRETTY => {
            check_args_len!(1);
            Ok((
                c,
                Node::AtHelper((lws, rws), AtHelperKind::JsonPretty, args),
            ))
        }
        _ => Err(LexError::Fail(PError::AtHelperNotExist, name.span())),
    }
}

/// Eat expression Node
fn expr(i: Cursor, lws: bool) -> PResult<Node> {
    match at_helper(i, lws) {
        Ok(x) => return Ok(x),
        Err(e @ LexError::Fail(..)) => return Err(e),
        _ => (),
    }

    let mut at = 0;
    let (c, rws, s) = loop {
        if let Some(j) = i.adv_find(at, '}') {
            if 0 < at + j && i.adv_starts_with(at + j - 1, "~}}") {
                break (i.adv(at + j + 2), true, &i.rest[..at + j - 1]);
            } else if i.adv_starts_with(at + j + 1, "}") {
                break (i.adv(at + j + 2), false, &i.rest[..at + j]);
            }

            at += j + 1;
        } else {
            return Err(LexError::Next(
                PError::Expr(DOption::None),
                Span::from_cursor(i, i.adv(at)),
            ));
        }
    };

    let (_, s, _) = trim(s);
    macro_rules! s {
        () => {
            Span::from_len(skip_ws(i), s.len())
        };
    }
    if s.starts_with("let ") {
        eat_local(s)
            .map(|e| (c, Node::Local(S(e, s!()))))
            .map_err(|e| {
                LexError::Fail(
                    PError::Local(DOption::Some(e.message)),
                    Span::from_range(skip_ws(i), e.span),
                )
            })
    } else {
        eat_expr(s)
            .map(|e| (c, Node::Expr((lws, rws), S(e, s!()))))
            .map_err(|e| {
                LexError::Fail(
                    PError::Expr(DOption::Some(e.message)),
                    Span::from_range(skip_ws(i), e.span),
                )
            })
    }
}

/// Intermediate error representation
#[derive(Debug)]
struct MiddleError {
    pub message: String,
    pub span: (usize, usize),
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
            span: (lo, hi),
        }
    }
}

/// Parse syn expression
fn eat_expr(i: &str) -> Result<Box<crate::Expr>, MiddleError> {
    parse_str::<Expr>(i)
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
fn eat_expr_list(i: &str) -> Result<Vec<crate::Expr>, MiddleError> {
    parse_str::<ExprList>(i)
        .map(Into::into)
        .map_err(|e| MiddleError::new(i, e))
}

/// Eat whitespace flag in end of expressions `.. }}` or `.. ~}}`
fn end_expr(i: Cursor) -> PResult<bool> {
    let c = skip_ws(i);
    if c.starts_with("~}}") {
        Ok((c.adv(3), true))
    } else if c.starts_with("}}") {
        Ok((c.adv(2), false))
    } else {
        Err(LexError::Fail(
            PError::EndExpression,
            Span::from_cursor(i, c),
        ))
    }
}

fn is_ident_start(c: char) -> bool {
    ('a' <= c && c <= 'z')
        || ('A' <= c && c <= 'Z')
        || c == '_'
        || (c > '\x7f' && UnicodeXID::is_xid_start(c))
}

fn is_ident_continue(c: char) -> bool {
    ('a' <= c && c <= 'z')
        || ('A' <= c && c <= 'Z')
        || c == '_'
        || ('0' <= c && c <= '9')
        || (c > '\x7f' && UnicodeXID::is_xid_continue(c))
}

/// Eat identifier
fn identifier(i: Cursor) -> PResult<&str> {
    let mut chars = i.chars();
    if chars.next().map(is_ident_start).unwrap_or(false) {
        if chars.next().map(is_ident_continue).unwrap_or(false) {
            if let Some(j) = chars.position(|c| !is_ident_continue(c)) {
                Ok((i.adv(j + 2), &i.rest[..j + 2]))
            } else {
                Ok((i.adv(i.len()), &i.rest))
            }
        } else {
            Ok((i.adv(1), &i.rest[..1]))
        }
    } else {
        Err(LexError::Next(PError::Ident, Span::from(i)))
    }
}

/// TODO: Define chars in path
/// Eat path at partial
/// Next white space close path
fn path(i: Cursor) -> PResult<&str> {
    take_while!(i, |i| !is_ws(i)).and_then(|(c, s)| {
        if s.is_empty() {
            Err(LexError::Fail(PError::PartialPath, Span::from(c)))
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
