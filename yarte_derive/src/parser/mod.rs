#![allow(
    clippy::many_single_char_names,
    clippy::cognitive_complexity
)]

use memchr::memchr;
use syn::{parse_str, Expr, Local};

use std::str::{self, from_utf8};

mod expr_list;
mod pre_partials;
mod stmt_local;

pub(crate) use self::pre_partials::parse_partials;
use self::{expr_list::ExprList, stmt_local::StmtLocal};

pub(crate) type Ws = (bool, bool);

#[derive(Debug, PartialEq)]
pub(crate) enum Node<'a> {
    Comment(&'a str),
    Expr(Ws, Box<Expr>),
    Helper(Box<Helper<'a>>),
    Lit(&'a str, &'a str, &'a str),
    Local(Box<Local>),
    Partial(Ws, &'a str, Vec<Expr>),
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

const ERR_ARGS: nom::ErrorKind = nom::ErrorKind::Custom(0);
const ERR_EXPR: nom::ErrorKind = nom::ErrorKind::Custom(1);
const ERR_EXPR_LIST: nom::ErrorKind = nom::ErrorKind::Custom(2);
const ERR_HELPER: nom::ErrorKind = nom::ErrorKind::Custom(3);
const ERR_IDENT: nom::ErrorKind = nom::ErrorKind::Custom(4);
const ERR_IF: nom::ErrorKind = nom::ErrorKind::Custom(5);
const ERR_LOCAL: nom::ErrorKind = nom::ErrorKind::Custom(6);
const ERR_PARTIAL: nom::ErrorKind = nom::ErrorKind::Custom(7);
const ERR_RAW: nom::ErrorKind = nom::ErrorKind::Custom(8);

pub(crate) fn parse(src: &str) -> Vec<Node> {
    match eat(Input(src.as_bytes())) {
        Ok((l, res)) => {
            if l.0.is_empty() {
                return res;
            }
            panic!(
                "problems parsing template source: {:?}",
                from_utf8(l.0).unwrap()
            );
        }
        Err(nom::Err::Error(err)) | Err(nom::Err::Failure(err)) => {
            match err.clone().into_error_kind() {
                ERR_EXPR => panic!(
                    "problems parsing wrapped or unwrapped expression: {:?}",
                    err
                ),
                ERR_EXPR_LIST => panic!("problems parsing partial arguments: {:?}", err),
                ERR_ARGS => panic!("problems parsing arguments: {:?}", err),
                ERR_HELPER => panic!("problems parsing helper: {:?}", err),
                ERR_IDENT => panic!("problems parsing identification variable: {:?}", err),
                ERR_IF => panic!("problems parsing helper IF: {:?}", err),
                ERR_LOCAL => panic!("problems parsing LET block: {:?}", err),
                ERR_PARTIAL => panic!("problems parsing partial: {:?}", err),
                _ => panic!("problems parsing template source: {:?}", err),
            }
        }
        Err(nom::Err::Incomplete(_)) => panic!("parsing incomplete"),
    }
}

type Input<'a> = nom::types::CompleteByteSlice<'a>;

#[allow(non_snake_case)]
fn Input(input: &[u8]) -> Input {
    nom::types::CompleteByteSlice(input)
}

macro_rules! try_eat {
    ($nodes:ident, $i:ident, $at:ident, $j:ident, $($t:tt)+) => {
        match $($t)+ {
            Ok((c, n)) => {
                eat_lit!($nodes, &$i[..$at + $j]);
                $nodes.push(n);
                $i = c;
                0
            },
            Err(nom::Err::Failure(err)) => break Err(nom::Err::Failure(err)),
            Err(_) => $at + $j + 3,
        }
    };
}

macro_rules! eat_lit {
    ($nodes:ident, $i:expr) => {
        let i = &$i;
        if !i.is_empty() {
            let (l, lit, r) = trim(Input(i));
            $nodes.push(Node::Lit(
                from_utf8(l.0).unwrap(),
                from_utf8(lit.0).unwrap(),
                from_utf8(r.0).unwrap(),
            ));
        }
    };
}

macro_rules! kill {
    ($nodes:ident, $c:expr, $i:expr) => {{
        eat_lit!($nodes, $i);
        break Ok((Input($c), $nodes));
    }};
}

/// $callback: special expressions like {{ else if }}
macro_rules! make_eater {
    ($name:ident, $callback:ident) => {
        fn $name(mut i: Input) -> Result<(Input, Vec<Node>), nom::Err<Input>> {
            let mut nodes = vec![];
            let mut at = 0;

            loop {
                if let Some(j) = memchr(b'{', &i[at..]) {
                    macro_rules! _switch {
                        ($n:expr, $t:expr, $ws:expr) => {
                            match $n {
                                b'{' => try_eat!(
                                    nodes,
                                    i,
                                    at,
                                    j,
                                    safe(Input(&i[at + j + 3 + $t..]), $ws)
                                ),
                                b'!' => {
                                    try_eat!(nodes, i, at, j, comment(Input(&i[at + j + 3 + $t..])))
                                }
                                b'#' => try_eat!(
                                    nodes,
                                    i,
                                    at,
                                    j,
                                    helper(Input(&i[at + j + 3 + $t..]), $ws)
                                ),
                                b'>' => try_eat!(
                                    nodes,
                                    i,
                                    at,
                                    j,
                                    partial(Input(&i[at + j + 3 + $t..]), $ws)
                                ),
                                b'R' => try_eat!(
                                    nodes,
                                    i,
                                    at,
                                    j,
                                    raw(Input(&i[at + j + 3 + $t..]), $ws)
                                ),
                                b'/' => kill!(nodes, &i[at + j + 2..], i[..at + j]),
                                _ => {
                                    $callback!(nodes, i, at, j, $t);
                                    try_eat!(
                                        nodes,
                                        i,
                                        at,
                                        j,
                                        expr(Input(&i[at + j + 2 + $t..]), $ws)
                                    )
                                }
                            }
                        };
                    }

                    let n = &i[at + j + 1..];
                    at = if 2 < n.len() {
                        if n[0] == b'{' {
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
                        kill!(nodes, &[], i.0);
                    };
                } else {
                    kill!(nodes, &[], i.0);
                }
            }
        }
    };
}

macro_rules! non {
    ($($t:tt)*) => {};
}

make_eater!(eat, non);

static IF: &[u8] = b"if";
static ELSE: &[u8] = b"else";

macro_rules! is_else {
    ($n:ident, $i:ident, $at:ident, $j:ident, $t:expr) => {
        if let Ok((c, _)) = take_while!(Input(&$i[$at + $j + 2 + $t..]), ws) {
            if c.0.starts_with(ELSE) {
                kill!($n, &$i[$at + $j + 2..], $i[..$at + $j]);
            }
        }
    };
}

make_eater!(eat_if, is_else);

// TODO: terminated with memchr
named!(comment<Input, Node>, map!(
    alt!(
        delimited!(tag!("--"), take_until!("--!}}"), tag!("--!}}")) |
        terminated!(take_until!("!}}"), tag!("!}}"))
    ),
    |i| Node::Comment(from_utf8(i.0).unwrap())
));

static LET: &[u8] = b"let ";
macro_rules! try_eat_local {
    ($c:ident, $s:ident) => {
        if $s.0.starts_with(LET) {
            if let Ok(e) = eat_local($s) {
                return Ok(($c, Node::Local(Box::new(e))));
            }
        }
    };
}

macro_rules! map_failure {
    ($i:expr, $e:ident, $($t:tt)+) => {
        ($($t)+).map_err(|_| nom::Err::Failure(error_position!($i, $e)))
    };
}

fn partial(i: Input, lws: bool) -> Result<(Input, Node), nom::Err<Input>> {
    let (i, ident) = do_parse!(
        i,
        take_while!(ws) >> ident: path >> take_while!(ws) >> (ident)
    )?;

    let (i, scope) = if let Ok((i, scope)) = args_list(i) {
        (i, scope)
    } else {
        (i, vec![])
    };
    let (_, rws) = map!(i, take!(1), |x| x.0.starts_with(b"~"))?;
    let (i, _) = map_failure!(
        i,
        ERR_PARTIAL,
        alt!(i, tag!("}}") | terminated!(take!(1), tag!("}}")))
    )?;

    Ok((i, Node::Partial((lws, rws), ident, scope)))
}

fn helper(i: Input, a_lws: bool) -> Result<(Input, Node), nom::Err<Input>> {
    let (i, (above_ws, ident, args)) = do_parse!(
        i,
        take_while!(ws)
            >> ident: identifier
            >> args: arguments
            >> take_while!(ws)
            >> rws: opt!(tag!("~"))
            >> tag!("}}")
            >> (((a_lws, rws.is_some()), ident, args))
    )?;

    if ident.eq("if") {
        return if_else(above_ws, i, args);
    }

    let (c, (below_ws, block, c_ident)) = map_failure!(
        i,
        ERR_HELPER,
        do_parse!(
            i,
            block: eat
                >> lws: opt!(tag!("~"))
                >> tag!("/")
                >> take_while!(ws)
                >> c_ident: identifier
                >> take_while!(ws)
                >> rws: opt!(tag!("~"))
                >> tag!("}}")
                >> (((lws.is_some(), rws.is_some()), block, c_ident))
        )
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
        Err(nom::Err::Failure(error_position!(i, ERR_HELPER)))
    }
}

#[inline]
fn if_else(abode_ws: Ws, i: Input, args: Expr) -> Result<(Input, Node), nom::Err<Input>> {
    let mut nodes = vec![];
    let mut tail = None;

    let (mut i, first) = eat_if(i)?;

    loop {
        if let Ok((c, lws)) = do_parse!(
            i,
            lws: opt!(tag!("~")) >> take_while!(ws) >> tag!(ELSE) >> (lws.is_some())
        ) {
            if let Ok((c, _)) = terminated!(c, take_while!(ws), tag!(IF)) {
                let (c, b) = map_failure!(
                    c,
                    ERR_IF,
                    do_parse!(
                        c,
                        take_while!(ws)
                            >> args: arguments
                            >> rws: opt!(tag!("~"))
                            >> tag!("}}")
                            >> block: eat_if
                            >> (((lws, rws.is_some()), args, block))
                    )
                )?;
                nodes.push(b);
                i = c;
            } else {
                let (c, b) = map_failure!(
                    c,
                    ERR_IF,
                    do_parse!(
                        c,
                        take_while!(ws)
                            >> rws: opt!(tag!("~"))
                            >> tag!("}}")
                            >> block: eat
                            >> (((lws, rws.is_some()), block))
                    )
                )?;
                tail = Some(b);
                i = c;
            }
        } else if let Ok((c, lws)) = do_parse!(
            i,
            lws: opt!(tag!("~")) >> tag!("/") >> take_while!(ws) >> tag!(IF) >> (lws.is_some())
        ) {
            let (c, below_ws) = map_failure!(
                c,
                ERR_IF,
                do_parse!(
                    c,
                    take_while!(ws) >> rws: opt!(tag!("~")) >> tag!("}}") >> ((lws, rws.is_some()))
                )
            )?;
            break Ok((
                c,
                Node::Helper(Box::new(Helper::If(
                    ((abode_ws, below_ws), args, first),
                    nodes,
                    tail,
                ))),
            ));
        } else {
            break Err(nom::Err::Failure(error_position!(i, ERR_IF)));
        }
    }
}

fn raw(i: Input, a_lws: bool) -> Result<(Input, Node), nom::Err<Input>> {
    let (i, a_rws) = do_parse!(
        i,
        take_while!(ws) >> rws: opt!(tag!("~")) >> tag!("}}") >> (rws.is_some())
    )?;

    let mut at = 0;
    let (c, (i, b_ws)) = loop {
        if let Some(j) = memchr(b'{', &i[at..]) {
            let n = &i[at + j + 1..];
            at = if !n.is_empty() && n[0] == b'{' {
                if let Ok((c, ws)) = do_parse!(
                    Input(&n[1..]),
                    lws: opt!(tag!("~"))
                        >> tag!("/R")
                        >> take_while!(ws)
                        >> rws: opt!(tag!("~"))
                        >> tag!("}}")
                        >> ((lws.is_some(), rws.is_some()))
                ) {
                    break (c, (Input(&i[..at + j]), ws));
                } else {
                    at + j + 4
                }
            } else {
                at + j + 1
            }
        } else {
            return Err(nom::Err::Failure(error_position!(i, ERR_RAW)));
        }
    };

    let (l, v, r) = trim(i);
    Ok((
        c,
        Node::Raw(
            ((a_lws, a_rws), b_ws),
            from_utf8(&l.0).unwrap(),
            from_utf8(&v.0).unwrap(),
            from_utf8(&r.0).unwrap(),
        ),
    ))
}

macro_rules! make_argument {
    ($name:ident, $fun:ident, $ret:ty) => {
        fn $name(i: Input) -> $ret {
            let mut at = 0;
            loop {
                if let Some(j) = memchr(b'}', &i[at..]) {
                    let n = &i[at + j + 1..];
                    if n.is_empty() {
                        break Err(nom::Err::Error(error_position!(i, ERR_ARGS)));
                    } else {
                        if n[0] == b'}' {
                            break if 0 < at + j {
                                if i[at + j - 1] == b'~' {
                                    $fun(Input(&i[..at + j - 1]))
                                        .map(|e| (Input(&i[at + j - 1..]), e))
                                } else {
                                    $fun(Input(&i[..at + j])).map(|e| (Input(&i[at + j..]), e))
                                }
                            } else {
                                Err(nom::Err::Failure(error_position!(i, ERR_ARGS)))
                            };
                        } else {
                            // next
                            at += j + 2;
                        }
                    }
                } else {
                    break Err(nom::Err::Error(error_position!(i, ERR_ARGS)));
                }
            }
        }
    };
}

make_argument!(arguments, eat_expr, Result<(Input, Expr), nom::Err<Input>>);

fn eat_expr_list(i: Input) -> Result<Vec<Expr>, nom::Err<Input>> {
    map_failure!(
        i,
        ERR_EXPR_LIST,
        parse_str::<ExprList>(safe_utf8(&i.0)).map(Into::into)
    )
}

make_argument!(
    args_list,
    eat_expr_list,
    Result<(Input, Vec<Expr>), nom::Err<Input>>
);

fn safe(i: Input, lws: bool) -> Result<(Input, Node), nom::Err<Input>> {
    let mut at = 0;

    let (c, rws, s) = loop {
        if let Some(j) = memchr(b'}', &i[at..]) {
            if let Ok((c, rws)) = do_parse!(
                Input(&i[at + j + 1..]),
                rws: opt!(tag!("~")) >> tag!("}}") >> (rws.is_some())
            ) {
                break (c, rws, Input(&i[..at + j]));
            }

            at += j + 1;
        } else {
            return Err(nom::Err::Error(error_position!(i, ERR_ARGS)));
        }
    };

    let (_, s, _) = trim(s);
    try_eat_local!(c, s);
    eat_expr(s).map(|e| (c, Node::Safe((lws, rws), Box::new(e))))
}

fn expr(i: Input, lws: bool) -> Result<(Input, Node), nom::Err<Input>> {
    let mut at = 0;

    let (c, rws, s) = loop {
        if let Some(j) = memchr(b'}', &i[at..]) {
            let n = &i[at + j + 1..];
            if n.starts_with(b"}") && 0 < at + j {
                break if i[at + j - 1] == b'~' {
                    (Input(&i[at + j + 2..]), true, Input(&i[..at + j - 1]))
                } else {
                    (Input(&i[at + j + 2..]), false, Input(&i[..at + j]))
                };
            }

            at += j + 1;
        } else {
            return Err(nom::Err::Error(error_position!(i, ERR_ARGS)));
        }
    };

    let (_, s, _) = trim(s);
    try_eat_local!(c, s);
    eat_expr(s).map(|e| (c, Node::Expr((lws, rws), Box::new(e))))
}

#[inline]
fn eat_expr(i: Input) -> Result<Expr, nom::Err<Input>> {
    map_failure!(i, ERR_EXPR, parse_str::<Expr>(from_utf8(i.0).unwrap()))
}

#[inline]
fn eat_local(i: Input) -> Result<Local, nom::Err<Input>> {
    map_failure!(
        i,
        ERR_LOCAL,
        parse_str::<StmtLocal>(safe_utf8(i.0)).map(Into::into)
    )
}

fn identifier(i: Input) -> Result<(Input, &str), nom::Err<Input>> {
    if i.0.is_empty() || !nom::is_alphabetic(i[0]) && i[0] != b'_' {
        return Err(nom::Err::Error(error_position!(i, ERR_IDENT)));
    }

    for (j, c) in i[1..].iter().enumerate() {
        if !nom::is_alphanumeric(*c) && *c != b'_' {
            return Ok((Input(&i[j + 1..]), safe_utf8(&i[..=j])));
        }
    }

    Ok((Input(&i[1..]), safe_utf8(&i[..1])))
}

named!(path<Input, &str>, map!(take_while1!(is_path), |x| safe_utf8(&x)));

#[inline]
fn is_path(n: u8) -> bool {
    n.is_ascii_graphic()
}

#[inline]
pub(crate) fn ws(n: u8) -> bool {
    n.is_ascii_whitespace()
}

fn trim(i: Input) -> (Input, Input, Input) {
    if i.0.is_empty() {
        return (Input(&[]), Input(&[]), Input(&[]));
    }

    if let Some(ln) = i.iter().position(|x| !ws(*x)) {
        let rn = i.iter().rposition(|x| !ws(*x)).unwrap();
        (Input(&i[..ln]), Input(&i[ln..=rn]), Input(&i[rn + 1..]))
    } else {
        (i, Input(&[]), Input(&[]))
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
        assert_eq!(parse(src), vec![]);
    }

    #[test]
    fn test_fallback() {
        let src = r#"{{"#;
        assert_eq!(parse(src), vec![Node::Lit("", "{{", "")]);
        let src = r#"{{{"#;
        assert_eq!(parse(src), vec![Node::Lit("", "{{{", "")]);
        let src = r#"{{#"#;
        assert_eq!(parse(src), vec![Node::Lit("", "{{#", "")]);
        let src = r#"{{>"#;
        assert_eq!(parse(src), vec![Node::Lit("", "{{>", "")]);
        let src = r#"{"#;
        assert_eq!(parse(src), vec![Node::Lit("", "{", "")]);
    }

    #[test]
    fn test_eat_comment() {
        let src = r#"{{! Commentary !}}"#;
        assert_eq!(parse(src), vec![Node::Comment(" Commentary ")]);
        let src = r#"{{!-- Commentary --!}}"#;
        assert_eq!(parse(src), vec![Node::Comment(" Commentary ")]);
        let src = r#"foo {{!-- Commentary --!}}"#;
        assert_eq!(
            parse(src),
            vec![Node::Lit("", "foo", " "), Node::Comment(" Commentary ")]
        );
    }

    #[test]
    fn test_eat_expr() {
        let src = r#"{{ var }}"#;
        assert_eq!(
            parse(src),
            vec![Node::Expr(WS, Box::new(parse_str::<Expr>("var").unwrap()))]
        );

        let src = r#"{{ fun() }}"#;
        assert_eq!(
            parse(src),
            vec![Node::Expr(
                WS,
                Box::new(parse_str::<Expr>("fun()").unwrap())
            )]
        );

        let src = r#"{{ fun(|a| a) }}"#;
        assert_eq!(
            parse(src),
            vec![Node::Expr(
                WS,
                Box::new(parse_str::<Expr>("fun(|a| a)").unwrap())
            )]
        );

        let src = r#"{{
            fun(|a| {
                { a }
            })
        }}"#;
        assert_eq!(
            parse(src),
            vec![Node::Expr(
                WS,
                Box::new(parse_str::<Expr>("fun(|a| {{a}})").unwrap())
            )]
        );
    }

    #[should_panic]
    #[test]
    fn test_eat_expr_panic_a() {
        let src = r#"{{ fn(|a| {{a}}) }}"#;
        parse(src);
    }

    #[should_panic]
    #[test]
    fn test_eat_expr_panic_b() {
        let src = r#"{{ let a = mut a  }}"#;
        parse(src);
    }

    #[test]
    fn test_eat_safe() {
        let src = r#"{{{ var }}}"#;
        assert_eq!(
            parse(src),
            vec![Node::Safe(WS, Box::new(parse_str::<Expr>("var").unwrap()))]
        );

        let src = r#"{{{ fun() }}}"#;
        assert_eq!(
            parse(src),
            vec![Node::Safe(
                WS,
                Box::new(parse_str::<Expr>("fun()").unwrap())
            )]
        );

        let src = r#"{{{ fun(|a| a) }}}"#;
        assert_eq!(
            parse(src),
            vec![Node::Safe(
                WS,
                Box::new(parse_str::<Expr>("fun(|a| a)").unwrap())
            )]
        );

        let src = r#"{{{
            fun(|a| {
                {{ a }}
            })
        }}}"#;
        assert_eq!(
            parse(src),
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
            parse(src),
            vec![Node::Safe(
                WS,
                Box::new(parse_str::<Expr>("fn(|a| {{{a}}})").unwrap()),
            )]
        );
    }

    #[test]
    fn test_trim() {
        assert_eq!(trim(Input(b" a ")), (Input(b" "), Input(b"a"), Input(b" ")));
        assert_eq!(trim(Input(b" a")), (Input(b" "), Input(b"a"), Input(b"")));
        assert_eq!(trim(Input(b"a")), (Input(b""), Input(b"a"), Input(b"")));
        assert_eq!(trim(Input(b"")), (Input(b""), Input(b""), Input(b"")));
        assert_eq!(trim(Input(b"a ")), (Input(b""), Input(b"a"), Input(b" ")));
        assert_eq!(trim(Input(b"a a")), (Input(b""), Input(b"a a"), Input(b"")));
        assert_eq!(
            trim(Input(b"a a ")),
            (Input(b""), Input(b"a a"), Input(b" "))
        );
        assert_eq!(
            trim(Input(b" \n\t\ra a ")),
            (Input(b" \n\t\r"), Input(b"a a"), Input(b" "))
        );
        assert_eq!(
            trim(Input(b" \n\t\r ")),
            (Input(b" \n\t\r "), Input(b""), Input(b""))
        );
    }

    #[test]
    fn test_eat_if() {
        let src = Input(br#"foo{{ else }}"#);
        assert_eq!(
            eat_if(src).unwrap(),
            (Input(b" else }}"), vec![Node::Lit("", "foo", "")])
        );
        let src = Input(br#"{{foo}}{{else}}"#);
        assert_eq!(
            eat_if(src).unwrap(),
            (
                Input(b"else}}"),
                vec![Node::Expr(WS, Box::new(parse_str::<Expr>("foo").unwrap()))]
            )
        );
        let src = Input(br#"{{ let a = foo }}{{else if cond}}{{else}}"#);
        let local = if let Stmt::Local(local) = parse_str::<Stmt>("let a = foo;").unwrap() {
            local
        } else {
            unreachable!();
        };
        assert_eq!(
            eat_if(src).unwrap(),
            (
                Input(b"else if cond}}{{else}}"),
                vec![Node::Local(Box::new(local))]
            )
        );
    }

    #[test]
    fn test_helpers() {
        let src = Input(b"each name }}{{first}} {{last}}{{/each}}");
        assert_eq!(
            helper(src, false).unwrap(),
            (
                Input(&[]),
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
        let src = Input(b"foo{{/if}}");
        let arg = parse_str::<Expr>("bar").unwrap();

        assert_eq!(
            if_else(WS, src, arg).unwrap(),
            (
                Input(b""),
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

        let src = Input(b"foo{{else}}bar{{/if}}");
        let arg = parse_str::<Expr>("bar").unwrap();

        assert_eq!(
            if_else(WS, src, arg).unwrap(),
            (
                Input(b""),
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
        let src = Input(b"foo{{else if cond }}bar{{else}}foO{{/if}}");
        let arg = parse_str::<Expr>("bar").unwrap();

        assert_eq!(
            if_else(WS, src, arg).unwrap(),
            (
                Input(b""),
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
            parse(src),
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
            parse(src),
            vec![Node::Expr(
                (true, true),
                Box::new(parse_str::<Expr>("foo").unwrap())
            )]
        );
        let src = "{{~ foo~}}";
        assert_eq!(
            parse(src),
            vec![Node::Expr(
                (true, true),
                Box::new(parse_str::<Expr>("foo").unwrap())
            )]
        );
        let src = "{{~ foo}}";
        assert_eq!(
            parse(src),
            vec![Node::Expr(
                (true, false),
                Box::new(parse_str::<Expr>("foo").unwrap())
            )]
        );
        let src = "{{foo    ~}}";
        assert_eq!(
            parse(src),
            vec![Node::Expr(
                (false, true),
                Box::new(parse_str::<Expr>("foo").unwrap())
            )]
        );
        let src = "{{~{foo }~}}";
        assert_eq!(
            parse(src),
            vec![Node::Safe(
                (true, true),
                Box::new(parse_str::<Expr>("foo").unwrap())
            )]
        );
        let src = "{{{foo }~}}";
        assert_eq!(
            parse(src),
            vec![Node::Safe(
                (false, true),
                Box::new(parse_str::<Expr>("foo").unwrap())
            )]
        );
    }

    #[test]
    fn test_ws_each() {
        let src = "{{~#each bar~}}{{~/each~}}";
        assert_eq!(
            parse(src),
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
            parse(src),
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
            parse(src),
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
            parse(src),
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
            parse(src),
            vec![Node::Raw(
                ((true, true), (true, true)),
                "",
                "{{#some }}{{/some}}",
                ""
            )]
        );
        let src = "{{R  ~}}{{#some }}{{/some}}{{/R ~}}";
        assert_eq!(
            parse(src),
            vec![Node::Raw(
                ((false, true), (false, true)),
                "",
                "{{#some }}{{/some}}",
                ""
            )]
        );
    }

    #[test]
    fn test_partial_ws() {
        let src = "{{~> partial ~}}";
        assert_eq!(
            parse(src),
            vec![Node::Partial((true, true), "partial", vec![])]
        );
        let src = "{{> partial scope ~}}";
        assert_eq!(
            parse(src),
            vec![Node::Partial(
                (false, true),
                "partial",
                vec![parse_str::<Expr>("scope").unwrap()],
            )]
        );
    }

    #[test]
    fn test_partial() {
        let src = "{{> partial }}";
        assert_eq!(parse(src), vec![Node::Partial(WS, "partial", vec![])]);
        let src = "{{> partial scope }}";
        assert_eq!(
            parse(src),
            vec![Node::Partial(
                WS,
                "partial",
                vec![parse_str::<Expr>("scope").unwrap()],
            )]
        );
    }

    #[test]
    fn test_raw() {
        let src = "{{R}}{{#some }}{{/some}}{{/R}}";
        assert_eq!(
            parse(src),
            vec![Node::Raw((WS, WS), "", "{{#some }}{{/some}}", "")]
        );
    }

    #[test]
    fn test_expr_list() {
        let src = Input(b"bar, foo = \"bar\"\n, fuu = 1  , goo = true,    ");
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
