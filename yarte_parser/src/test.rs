use syn::parse_str;

use crate::{
    eat_expr_list, eat_if, hel, if_else,
    source_map::{Span, S},
    trim, Cursor, Helper,
    Node::*,
    Ws,
};

const WS: Ws = (false, false);

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
            vec![S(
                Lit("", S("foo", Span { lo: 0, hi: 3 }), ""),
                Span { lo: 0, hi: 3 },
            )]
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
            vec![S(
                Expr(
                    WS,
                    S(
                        Box::new(parse_str::<crate::Expr>("foo").unwrap()),
                        Span { lo: 2, hi: 5 },
                    ),
                ),
                Span { lo: 0, hi: 7 },
            )]
        )
    );
    let rest = r#"{{ let a = foo }}{{else if cond}}{{else}}"#;
    let local = parse_str::<crate::Local>("let a = foo").unwrap();
    let result = "else if cond}}{{else}}";
    assert_eq!(
        eat_if(Cursor { rest, off: 0 }).unwrap(),
        (
            Cursor {
                rest: result,
                off: (rest.len() - result.len()) as u32,
            },
            vec![S(
                Local(S(Box::new(local), Span { lo: 3, hi: 14 })),
                Span { lo: 0, hi: 17 },
            )]
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
            Helper(Box::new(Helper::Each(
                (WS, WS),
                S(
                    Box::new(parse_str::<crate::Expr>("name").unwrap()),
                    Span { lo: 5, hi: 9 },
                ),
                vec![
                    S(
                        Expr(
                            WS,
                            S(
                                Box::new(parse_str::<crate::Expr>("first").unwrap()),
                                Span { lo: 14, hi: 19 },
                            ),
                        ),
                        Span { lo: 12, hi: 21 },
                    ),
                    S(
                        Lit(" ", S("", Span { lo: 22, hi: 22 }), ""),
                        Span { lo: 21, hi: 22 },
                    ),
                    S(
                        Expr(
                            WS,
                            S(
                                Box::new(parse_str::<crate::Expr>("last").unwrap()),
                                Span { lo: 24, hi: 28 },
                            ),
                        ),
                        Span { lo: 22, hi: 30 },
                    ),
                ],
            )))
        )
    );
}

#[test]
fn test_if_else() {
    let rest = "foo{{/if}}";
    let args = S(
        Box::new(parse_str::<crate::Expr>("bar").unwrap()),
        Span { lo: 0, hi: 0 },
    );

    assert_eq!(
        if_else(WS, Cursor { rest, off: 0 }, args.clone()).unwrap(),
        (
            Cursor {
                rest: "",
                off: rest.len() as u32,
            },
            Helper(Box::new(Helper::If(
                (
                    (WS, WS),
                    args,
                    vec![S(
                        Lit("", S("foo", Span { lo: 0, hi: 3 }), ""),
                        Span { lo: 0, hi: 3 },
                    )]
                ),
                vec![],
                None,
            )))
        )
    );

    let rest = "foo{{else}}bar{{/if}}";
    let args = S(
        Box::new(parse_str::<crate::Expr>("bar").unwrap()),
        Span { lo: 0, hi: 0 },
    );

    assert_eq!(
        if_else(WS, Cursor { rest, off: 0 }, args.clone()).unwrap(),
        (
            Cursor {
                rest: "",
                off: rest.len() as u32,
            },
            Helper(Box::new(Helper::If(
                (
                    (WS, WS),
                    args,
                    vec![S(
                        Lit("", S("foo", Span { lo: 0, hi: 3 }), ""),
                        Span { lo: 0, hi: 3 },
                    )]
                ),
                vec![],
                Some((
                    WS,
                    vec![S(
                        Lit("", S("bar", Span { lo: 11, hi: 14 }), ""),
                        Span { lo: 11, hi: 14 },
                    )]
                )),
            )))
        )
    );
}

#[test]
fn test_else_if() {
    let rest = "foo{{else if cond }}bar{{else}}foO{{/if}}";
    let args = S(
        Box::new(parse_str::<crate::Expr>("bar").unwrap()),
        Span { lo: 0, hi: 0 },
    );

    assert_eq!(
        if_else(WS, Cursor { rest, off: 0 }, args.clone()).unwrap(),
        (
            Cursor {
                rest: "",
                off: rest.len() as u32,
            },
            Helper(Box::new(Helper::If(
                (
                    (WS, WS),
                    args,
                    vec![S(
                        Lit("", S("foo", Span { lo: 0, hi: 3 }), ""),
                        Span { lo: 0, hi: 3 },
                    )]
                ),
                vec![(
                    WS,
                    S(
                        Box::new(parse_str::<crate::Expr>("cond").unwrap()),
                        Span { lo: 13, hi: 17 },
                    ),
                    vec![S(
                        Lit("", S("bar", Span { lo: 20, hi: 23 }), ""),
                        Span { lo: 20, hi: 23 },
                    )]
                )],
                Some((
                    WS,
                    vec![S(
                        Lit("", S("foO", Span { lo: 31, hi: 34 }), ""),
                        Span { lo: 31, hi: 34 },
                    )]
                )),
            )))
        )
    );
}

#[test]
fn test_expr_list() {
    let src = "bar, foo = \"bar\"\n, fuu = 1  , goo = true,    ";
    assert_eq!(
        eat_expr_list(src).unwrap(),
        vec![
            parse_str::<crate::Expr>("bar").unwrap(),
            parse_str::<crate::Expr>("foo=\"bar\"").unwrap(),
            parse_str::<crate::Expr>("fuu=1").unwrap(),
            parse_str::<crate::Expr>("goo=true").unwrap(),
        ]
    );
}
