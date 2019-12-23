#![allow(
clippy::many_single_char_names,
clippy::cognitive_complexity
)]

use std::str;

use syn::{parse_str, Expr, Local};

mod expr_list;
mod pre_partials;
mod stmt_local;
#[macro_use]
mod strnom;

pub(crate) use self::pre_partials::parse_partials;
use self::{
    expr_list::ExprList,
    stmt_local::StmtLocal,
    strnom::{is_ws, skip_ws, Cursor, LexError, PResult},
};

use crate::parser::strnom::ws;
use unicode_xid::UnicodeXID;

pub(crate) type Ws = (bool, bool);

#[derive(Debug, PartialEq)]
pub(crate) struct Partial<'a>(pub Ws, pub &'a str, pub Vec<Expr>);

#[derive(Debug, PartialEq)]
pub(crate) enum Node<'a> {
    Comment(&'a str),
    Expr(Ws, Box<Expr>),
    Helper(Box<Helper<'a>>),
    Lit(&'a str, &'a str, &'a str),
    Local(Box<Local>),
    Partial(Partial<'a>),
    Raw((Ws, Ws), &'a str, &'a str, &'a str),
    Safe(Ws, Box<Expr>),
}

#[derive(Debug, PartialEq)]
pub(crate) enum Helper<'a> {
    Each((Ws, Ws), Expr, Vec<Node<'a>>),
    If(
        ((Ws, Ws), Expr, Vec<Node<'a>>),
        Vec<(Ws, Expr, Vec<Node<'a>>)>,
        Option<(Ws, Vec<Node<'a>>)>,
    ),
    With((Ws, Ws), Expr, Vec<Node<'a>>),
    Unless((Ws, Ws), Expr, Vec<Node<'a>>),
    // TODO:
    Defined((Ws, Ws), &'a str, Expr, Vec<Node<'a>>),
}

pub(crate) fn parse(rest: &str, off: u32) -> Vec<Node> {
    match eat(Cursor { rest, off }) {
        Ok((l, res)) => {
            if l.is_empty() {
                return res;
            }
            panic!("problems parsing template source: {:?}", l.rest);
        }
        Err(LexError::Next) | Err(LexError::Fail) => panic!("problems parsing template source"),
    }
}

macro_rules! try_eat {
    ($nodes:ident, $i:ident, $at:ident, $j:ident, $($t:tt)+) => {
        match $($t)+ {
            Ok((c, n)) => {
                eat_lit!($nodes, &$i.rest[..$at + $j]);
                $nodes.push(n);
                $i = c;
                0
            },
            Err(LexError::Fail) => break Err(LexError::Fail),
            Err(LexError::Next) => $at + $j + 3,
        }
    };
}

macro_rules! eat_lit {
    ($nodes:ident, $i:expr) => {
        let i = &$i;
        if !i.is_empty() {
            let (l, lit, r) = trim(i);
            $nodes.push(Node::Lit(l, lit, r));
        }
    };
}

macro_rules! kill {
    ($nodes:ident, $c:expr, $i:expr) => {{
        eat_lit!($nodes, $i);
        break Ok(($c, $nodes));
    }};
}

/// $callback: special expressions like {{ else if }}
macro_rules! make_eater {
    ($name:ident, $callback:ident) => {
        fn $name(mut i: Cursor) -> PResult<Vec<Node>> {
            let mut buf = vec![];
            let mut at = 0;

            loop {
                if let Some(j) = i.adv_find(at, '{') {
                    macro_rules! _switch {
                        ($n:expr, $t:expr, $ws:expr) => {
                            match $n {
                                b'{' => try_eat!(buf, i, at, j, safe(i.adv(at + j + 3 + $t), $ws)),
                                b'!' => try_eat!(buf, i, at, j, comment(i.adv(at + j + 3 + $t))),
                                b'#' => try_eat!(buf, i, at, j, hel(i.adv(at + j + 3 + $t), $ws)),
                                b'>' => try_eat!(buf, i, at, j, par(i.adv(at + j + 3 + $t), $ws)),
                                b'R' => try_eat!(buf, i, at, j, raw(i.adv(at + j + 3 + $t), $ws)),
                                b'/' => kill!(buf, i.adv(at + j + 2), &i.rest[..at + j]),
                                _ => {
                                    $callback!(buf, i, at, j, $t);
                                    try_eat!(buf, i, at, j, expr(i.adv(at + j + 2 + $t), $ws))
                                }
                            }
                        };
                    }

                    let n = &i.rest[j + 1..].as_bytes();
                    if 2 < n.len() {
                        at = if n[0] == b'{' {
                            if n[1] == b'~' {
                                _switch!(n[2], 1, true)
                            } else {
                                _switch!(n[1], 0, false)
                            }
                        } else {
                            // next
                            at + j + 2
                        }
                    } else {
                        kill!(buf, i.adv(i.len()), &i.rest);
                    };
                } else {
                    kill!(buf, i.adv(i.len()), &i.rest);
                }
            }
        }
    };
}

macro_rules! non {
    ($($t:tt)*) => {};
}

make_eater!(eat, non);

const IF: &str = "if";
const ELSE: &str = "else";

macro_rules! is_else {
    ($n:ident, $i:ident, $at:ident, $j:ident, $t:expr) => {
        if skip_ws($i.adv($at + $j + 2 + $t)).starts_with(ELSE) {
            kill!($n, $i.adv($at + $j + 2), &$i.rest[..$at + $j]);
        }
    };
}

make_eater!(eat_if, is_else);

fn comment(c: Cursor) -> PResult<Node> {
    let (c, expected) = if c.starts_with("--") {
        (c.adv(2), "--!}}")
    } else {
        (c, "!}}")
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
            break Err(LexError::Next);
        }
    }
}

const LET: &str = "let ";
macro_rules! try_eat_local {
    ($c:ident, $s:ident) => {
        if $s.starts_with(LET) {
            if let Ok(e) = eat_local($s) {
                return Ok(($c, Node::Local(Box::new(e))));
            }
        }
    };
}

#[inline]
fn par(i: Cursor, lws: bool) -> PResult<Node> {
    partial(i, lws).map(|(i, p)| (i, Node::Partial(p)))
}

fn partial(i: Cursor, lws: bool) -> PResult<Partial> {
    let (i, ident) = do_parse!(i, ws >> p: path >> ws >> (p))?;

    let (i, scope) = if let Ok((i, scope)) = args_list(i) {
        (i, scope)
    } else {
        (i, vec![])
    };
    let (i, rws) = end_expr(i)?;

    Ok((i, Partial((lws, rws), ident, scope)))
}

fn hel(i: Cursor, a_lws: bool) -> PResult<Node> {
    let (i, (above_ws, ident, args)) = do_parse!(
        i,
        ws >> ident: identifier
            >> args: arguments
            >> rws: end_expr
            >> (((a_lws, rws), ident, args))
    )?;

    if ident.eq("if") {
        return if_else(above_ws, i, args);
    }

    let (c, (below_ws, block, c_ident)) = do_parse!(
        i,
        block: eat
            >> lws: opt!(tag!("~"))
            >> tag!("/")
            >> ws
            >> c_ident: identifier
            >> rws: end_expr
            >> (((lws.is_some(), rws), block, c_ident))
    )?;

    if ident.eq(c_ident) {
        Ok((
            c,
            Node::Helper(Box::new({
                match ident {
                    "each" => Helper::Each((above_ws, below_ws), args, block),
                    "with" => Helper::With((above_ws, below_ws), args, block),
                    "unless" => Helper::Unless((above_ws, below_ws), args, block),
                    defined => Helper::Defined((above_ws, below_ws), defined, args, block),
                }
            })),
        ))
    } else {
        Err(LexError::Fail)
    }
}

#[inline]
fn if_else(abode_ws: Ws, i: Cursor, args: Expr) -> PResult<Node> {
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
            break Err(LexError::Fail);
        }
    }
}

#[inline]
fn end_expr(i: Cursor) -> PResult<bool> {
    let i = skip_ws(i);
    if i.starts_with("~}}") {
        Ok((i.adv(3), true))
    } else if i.starts_with("}}") {
        Ok((i.adv(2), false))
    } else {
        Err(LexError::Fail)
    }
}

fn raw(i: Cursor, a_lws: bool) -> PResult<Node> {
    let (i, a_rws) = end_expr(i)?;
    let mut at = 0;

    let (c, (i, b_ws)) = loop {
        if let Some(j) = i.adv_find(at, '{') {
            let n = i.adv(at + j + 1);
            if n.chars().next().map(|x| '{' == x).unwrap_or(false) {
                if let Ok((c, ws)) = do_parse!(
                    n.adv(1),
                    lws: opt!(tag!("~"))
                        >> tag!("/R")
                        >> ws
                        >> rws: opt!(tag!("~"))
                        >> tag!("}}")
                        >> ((lws.is_some(), rws.is_some()))
                ) {
                    break (c, (&i.rest[..at + j], ws));
                } else {
                    at += j + 4;
                }
            } else {
                at += j + 1;
            }
        } else {
            return Err(LexError::Fail);
        }
    };

    let (l, v, r) = trim(i);
    Ok((c, Node::Raw(((a_lws, a_rws), b_ws), l, v, r)))
}

macro_rules! make_argument {
    ($name:ident, $fun:ident, $ret:ty) => {
        fn $name(i: Cursor) -> $ret {
            let mut at = 0;
            loop {
                if let Some(j) = i.adv_find(at, '}') {
                    if 0 < j && i.adv_starts_with(at + j - 1, "~}}") {
                        break $fun(&i.rest[..j - 1]).map(|e| (i.adv(at + j - 1), e));
                    } else if i.adv_starts_with(j + 1, "}") {
                        break $fun(&i.rest[..j]).map(|e| (i.adv(at + j), e));
                    }

                    at += j + 1;
                } else {
                    break Err(LexError::Next);
                }
            }
        }
    };
}

make_argument!(arguments, eat_expr, PResult<Expr>);

make_argument!(args_list, eat_expr_list, PResult<Vec<Expr>>);

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
            return Err(LexError::Next);
        }
    };

    let (_, s, _) = trim(s);
    eat_expr(s).map(|e| (c, Node::Safe((lws, rws), Box::new(e))))
}

fn expr(i: Cursor, lws: bool) -> PResult<Node> {
    let mut at = 0;
    let (i, rws, s) = loop {
        if let Some(j) = i.adv_find(at, '}') {
            if 0 < at + j && i.adv_starts_with(at + j - 1, "~}}") {
                break (i.adv(at + j + 2), true, &i.rest[..at + j - 1]);
            } else if i.adv_starts_with(at + j + 1, "}") {
                break (i.adv(at + j + 2), false, &i.rest[..at + j]);
            }

            at += j + 1;
        } else {
            return Err(LexError::Next);
        }
    };

    let (_, s, _) = trim(s);
    try_eat_local!(i, s);
    eat_expr(s).map(|e| (i, Node::Expr((lws, rws), Box::new(e))))
}

fn eat_expr(i: &str) -> Result<Expr, LexError> {
    map_fail!(parse_str::<Expr>(i))
}

fn eat_local(i: &str) -> Result<Local, LexError> {
    map_fail!(parse_str::<StmtLocal>(i).map(Into::into))
}

fn eat_expr_list(i: &str) -> Result<Vec<Expr>, LexError> {
    map_fail!(parse_str::<ExprList>(i).map(Into::into))
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
        Err(LexError::Next)
    }
}

/// Next white space close path
fn path(c: Cursor) -> PResult<&str> {
    c.chars()
        .position(is_ws)
        .map(|j| (c.adv(j), &c.rest[..j]))
        .ok_or(LexError::Next)
}

fn trim(i: &str) -> (&str, &str, &str) {
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

/// Use when previous checks
fn safe_utf8(s: &[u8]) -> &str {
    unsafe { ::std::str::from_utf8_unchecked(s) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_str, Stmt};

    const WS: Ws = (false, false);

    #[test]
    fn test_empty() {
        let src = r#""#;
        assert_eq!(parse(src, 0), vec![]);
    }

    #[test]
    fn test_fallback() {
        let src = r#"{{"#;
        assert_eq!(parse(src, 0), vec![Node::Lit("", "{{", "")]);
        let src = r#"{{{"#;
        assert_eq!(parse(src, 0), vec![Node::Lit("", "{{{", "")]);
        let src = r#"{{#"#;
        assert_eq!(parse(src, 0), vec![Node::Lit("", "{{#", "")]);
        let src = r#"{{>"#;
        assert_eq!(parse(src, 0), vec![Node::Lit("", "{{>", "")]);
        let src = r#"{"#;
        assert_eq!(parse(src, 0), vec![Node::Lit("", "{", "")]);
    }

    #[test]
    fn test_eat_comment() {
        let src = r#"{{! Commentary !}}"#;
        assert_eq!(parse(src, 0), vec![Node::Comment(" Commentary ")]);
        let src = r#"{{!-- Commentary --!}}"#;
        assert_eq!(parse(src, 0), vec![Node::Comment(" Commentary ")]);
        let src = r#"foo {{!-- Commentary --!}}"#;
        assert_eq!(
            parse(src, 0),
            vec![Node::Lit("", "foo", " "), Node::Comment(" Commentary ")]
        );
    }

    #[test]
    fn test_eat_expr() {
        let src = r#"{{ var }}"#;
        assert_eq!(
            parse(src, 0),
            vec![Node::Expr(WS, Box::new(parse_str::<Expr>("var").unwrap()))]
        );

        let src = r#"{{ fun() }}"#;
        assert_eq!(
            parse(src, 0),
            vec![Node::Expr(
                WS,
                Box::new(parse_str::<Expr>("fun()").unwrap()),
            )]
        );

        let src = r#"{{ fun(|a| a) }}"#;
        assert_eq!(
            parse(src, 0),
            vec![Node::Expr(
                WS,
                Box::new(parse_str::<Expr>("fun(|a| a)").unwrap()),
            )]
        );

        let src = r#"{{
            fun(|a| {
                { a }
            })
        }}"#;
        assert_eq!(
            parse(src, 0),
            vec![Node::Expr(
                WS,
                Box::new(parse_str::<Expr>("fun(|a| {{a}})").unwrap()),
            )]
        );
    }

    #[should_panic]
    #[test]
    fn test_eat_expr_panic_a() {
        let src = r#"{{ fn(|a| {{a}}) }}"#;
        parse(src, 0);
    }

    #[should_panic]
    #[test]
    fn test_eat_expr_panic_b() {
        let src = r#"{{ let a = mut a  }}"#;
        parse(src, 0);
    }

    #[test]
    fn test_eat_safe() {
        let src = r#"{{{ var }}}"#;
        assert_eq!(
            parse(src, 0),
            vec![Node::Safe(WS, Box::new(parse_str::<Expr>("var").unwrap()))]
        );

        let src = r#"{{{ fun() }}}"#;
        assert_eq!(
            parse(src, 0),
            vec![Node::Safe(
                WS,
                Box::new(parse_str::<Expr>("fun()").unwrap()),
            )]
        );

        let src = r#"{{{ fun(|a| a) }}}"#;
        assert_eq!(
            parse(src, 0),
            vec![Node::Safe(
                WS,
                Box::new(parse_str::<Expr>("fun(|a| a)").unwrap()),
            )]
        );

        let src = r#"{{{
            fun(|a| {
                {{ a }}
            })
        }}}"#;
        assert_eq!(
            parse(src, 0),
            vec![Node::Safe(
                WS,
                Box::new(parse_str::<Expr>("fun(|a| {{{a}}})").unwrap()),
            )]
        );
    }

    #[should_panic]
    #[test]
    fn test_eat_safe_panic() {
        let src = r#"{{ fn(|a| {{{a}}}) }}"#;
        assert_eq!(
            parse(src, 0),
            vec![Node::Safe(
                WS,
                Box::new(parse_str::<Expr>("fn(|a| {{{a}}})").unwrap()),
            )]
        );
    }

    #[test]
    fn test_trim() {
        assert_eq!(trim(" a "), (" ", "a", " "));
        assert_eq!(trim(" a"), (" ", "a", ""));
        assert_eq!(trim("a"), ("", "a", ""));
        assert_eq!(trim(""), ("", "", ""));
        assert_eq!(trim("a "), ("", "a", " "));
        assert_eq!(trim("a a"), ("", "a a", ""));
        assert_eq!(trim("a a "), ("", "a a", " "));
        assert_eq!(trim(" \n\t\ra a "), (" \n\t\r", "a a", " "));
        assert_eq!(trim(" \n\t\r "), (" \n\t\r ", "", ""));
    }

    #[test]
    fn test_eat_if() {
        let rest = r#"foo{{ else }}"#;
        let result = " else }}";
        assert_eq!(
            eat_if(Cursor { rest, off: 0 }).unwrap(),
            (
                Cursor {
                    rest: result,
                    off: (rest.len() - result.len()) as u32,
                },
                vec![Node::Lit("", "foo", "")]
            )
        );
        let rest = r#"{{foo}}{{else}}"#;
        let result = "else}}";
        assert_eq!(
            eat_if(Cursor { rest, off: 0 }).unwrap(),
            (
                Cursor {
                    rest: result,
                    off: (rest.len() - result.len()) as u32,
                },
                vec![Node::Expr(WS, Box::new(parse_str::<Expr>("foo").unwrap()))]
            )
        );
        let rest = r#"{{ let a = foo }}{{else if cond}}{{else}}"#;
        let local = if let Stmt::Local(local) = parse_str::<Stmt>("let a = foo;").unwrap() {
            local
        } else {
            unreachable!();
        };
        let result = "else if cond}}{{else}}";
        assert_eq!(
            eat_if(Cursor { rest, off: 0 }).unwrap(),
            (
                Cursor {
                    rest: result,
                    off: (rest.len() - result.len()) as u32,
                },
                vec![Node::Local(Box::new(local))]
            )
        );
    }

    #[test]
    fn test_helpers() {
        let rest = "each name }}{{first}} {{last}}{{/each}}";
        assert_eq!(
            hel(Cursor { rest, off: 0 }, false).unwrap(),
            (
                Cursor {
                    rest: "",
                    off: rest.len() as u32,
                },
                Node::Helper(Box::new(Helper::Each(
                    (WS, WS),
                    parse_str::<Expr>("name").unwrap(),
                    vec![
                        Node::Expr(WS, Box::new(parse_str::<Expr>("first").unwrap())),
                        Node::Lit(" ", "", ""),
                        Node::Expr(WS, Box::new(parse_str::<Expr>("last").unwrap())),
                    ],
                )))
            )
        );
    }

    #[test]
    fn test_if_else() {
        let rest = "foo{{/if}}";
        let arg = parse_str::<Expr>("bar").unwrap();

        assert_eq!(
            if_else(WS, Cursor { rest, off: 0 }, arg).unwrap(),
            (
                Cursor {
                    rest: "",
                    off: rest.len() as u32,
                },
                Node::Helper(Box::new(Helper::If(
                    (
                        (WS, WS),
                        parse_str::<Expr>("bar").unwrap(),
                        vec![Node::Lit("", "foo", "")]
                    ),
                    vec![],
                    None,
                )))
            )
        );

        let rest = "foo{{else}}bar{{/if}}";
        let arg = parse_str::<Expr>("bar").unwrap();

        assert_eq!(
            if_else(WS, Cursor { rest, off: 0 }, arg).unwrap(),
            (
                Cursor {
                    rest: "",
                    off: rest.len() as u32,
                },
                Node::Helper(Box::new(Helper::If(
                    (
                        (WS, WS),
                        parse_str::<Expr>("bar").unwrap(),
                        vec![Node::Lit("", "foo", "")]
                    ),
                    vec![],
                    Some((WS, vec![Node::Lit("", "bar", "")])),
                )))
            )
        );
    }

    #[test]
    fn test_else_if() {
        let rest = "foo{{else if cond }}bar{{else}}foO{{/if}}";
        let arg = parse_str::<Expr>("bar").unwrap();

        assert_eq!(
            if_else(WS, Cursor { rest, off: 0 }, arg).unwrap(),
            (
                Cursor {
                    rest: "",
                    off: rest.len() as u32,
                },
                Node::Helper(Box::new(Helper::If(
                    (
                        (WS, WS),
                        parse_str::<Expr>("bar").unwrap(),
                        vec![Node::Lit("", "foo", "")]
                    ),
                    vec![(
                        WS,
                        parse_str::<Expr>("cond").unwrap(),
                        vec![Node::Lit("", "bar", "")],
                    )],
                    Some((WS, vec![Node::Lit("", "foO", "")])),
                )))
            )
        );
    }

    #[test]
    fn test_defined() {
        let src = "{{#foo bar}}hello{{/foo}}";

        assert_eq!(
            parse(src, 0),
            vec![Node::Helper(Box::new(Helper::Defined(
                (WS, WS),
                "foo",
                parse_str::<Expr>("bar").unwrap(),
                vec![Node::Lit("", "hello", "")],
            )))]
        );
    }

    #[test]
    fn test_ws_expr() {
        let src = "{{~foo~}}";
        assert_eq!(
            parse(src, 0),
            vec![Node::Expr(
                (true, true),
                Box::new(parse_str::<Expr>("foo").unwrap()),
            )]
        );
        let src = "{{~ foo~}}";
        assert_eq!(
            parse(src, 0),
            vec![Node::Expr(
                (true, true),
                Box::new(parse_str::<Expr>("foo").unwrap()),
            )]
        );
        let src = "{{~ foo}}";
        assert_eq!(
            parse(src, 0),
            vec![Node::Expr(
                (true, false),
                Box::new(parse_str::<Expr>("foo").unwrap()),
            )]
        );
        let src = "{{foo    ~}}";
        assert_eq!(
            parse(src, 0),
            vec![Node::Expr(
                (false, true),
                Box::new(parse_str::<Expr>("foo").unwrap()),
            )]
        );
        let src = "{{~{foo }~}}";
        assert_eq!(
            parse(src, 0),
            vec![Node::Safe(
                (true, true),
                Box::new(parse_str::<Expr>("foo").unwrap()),
            )]
        );
        let src = "{{{foo }~}}";
        assert_eq!(
            parse(src, 0),
            vec![Node::Safe(
                (false, true),
                Box::new(parse_str::<Expr>("foo").unwrap()),
            )]
        );
    }

    #[test]
    fn test_ws_each() {
        let src = "{{~#each bar~}}{{~/each~}}";
        assert_eq!(
            parse(src, 0),
            vec![Node::Helper(Box::new(Helper::Each(
                ((true, true), (true, true)),
                parse_str::<Expr>("bar").unwrap(),
                vec![],
            )))]
        );
    }

    #[test]
    fn test_ws_if() {
        let src = "{{~#if bar~}}{{~/if~}}";
        assert_eq!(
            parse(src, 0),
            vec![Node::Helper(Box::new(Helper::If(
                (
                    ((true, true), (true, true)),
                    parse_str::<Expr>("bar").unwrap(),
                    vec![],
                ),
                vec![],
                None,
            )))]
        );
    }

    #[test]
    fn test_ws_if_else() {
        let src = "{{~#if bar~}}{{~else~}}{{~/if~}}";
        assert_eq!(
            parse(src, 0),
            vec![Node::Helper(Box::new(Helper::If(
                (
                    ((true, true), (true, true)),
                    parse_str::<Expr>("bar").unwrap(),
                    vec![],
                ),
                vec![],
                Some(((true, true), vec![])),
            )))]
        );
    }

    #[test]
    fn test_ws_if_else_if() {
        let src = "{{~#if bar~}}{{~else if bar~}}{{~else~}}{{~/if~}}";
        assert_eq!(
            parse(src, 0),
            vec![Node::Helper(Box::new(Helper::If(
                (
                    ((true, true), (true, true)),
                    parse_str::<Expr>("bar").unwrap(),
                    vec![],
                ),
                vec![((true, true), parse_str::<Expr>("bar").unwrap(), vec![],)],
                Some(((true, true), vec![])),
            )))]
        );
    }

    #[test]
    fn test_ws_raw() {
        let src = "{{~R~}}{{#some }}{{/some}}{{~/R ~}}";
        assert_eq!(
            parse(src, 0),
            vec![Node::Raw(
                ((true, true), (true, true)),
                "",
                "{{#some }}{{/some}}",
                "",
            )]
        );
        let src = "{{R  ~}}{{#some }}{{/some}}{{/R ~}}";
        assert_eq!(
            parse(src, 0),
            vec![Node::Raw(
                ((false, true), (false, true)),
                "",
                "{{#some }}{{/some}}",
                "",
            )]
        );
    }

    #[test]
    fn test_partial_ws() {
        let src = "{{~> partial ~}}";
        assert_eq!(
            parse(src, 0),
            vec![Node::Partial(Partial((true, true), "partial", vec![]))]
        );
        let src = "{{> partial scope ~}}";
        assert_eq!(
            parse(src, 0),
            vec![Node::Partial(Partial(
                (false, true),
                "partial",
                vec![parse_str::<Expr>("scope").unwrap()],
            ))]
        );
    }

    #[test]
    fn test_partial() {
        let src = "{{> partial }}";
        assert_eq!(
            parse(src, 0),
            vec![Node::Partial(Partial(WS, "partial", vec![]))]
        );
        let src = "{{> partial scope }}";
        assert_eq!(
            parse(src, 0),
            vec![Node::Partial(Partial(
                WS,
                "partial",
                vec![parse_str::<Expr>("scope").unwrap()],
            ))]
        );
    }

    #[test]
    fn test_raw() {
        let src = "{{R}}{{#some }}{{/some}}{{/R}}";
        assert_eq!(
            parse(src, 0),
            vec![Node::Raw((WS, WS), "", "{{#some }}{{/some}}", "")]
        );
    }

    #[test]
    fn test_expr_list() {
        let src = "bar, foo = \"bar\"\n, fuu = 1  , goo = true,    ";
        assert_eq!(
            eat_expr_list(src).unwrap(),
            vec![
                parse_str::<Expr>("bar").unwrap(),
                parse_str::<Expr>("foo=\"bar\"").unwrap(),
                parse_str::<Expr>("fuu=1").unwrap(),
                parse_str::<Expr>("goo=true").unwrap(),
            ]
        );
    }
}
