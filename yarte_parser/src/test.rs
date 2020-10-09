use syn::{parse_str, Expr, Stmt};

use super::{parse as _parse, Helper, Node::*, Partial, PartialBlock, *};

const WS: Ws = (false, false);

macro_rules! bytes {
    ($a:tt..$b:tt) => {
        Span {
            lo: $a as u32,
            hi: $b as u32,
        }
    };
}

fn parse(rest: &str) -> Vec<SNode> {
    _parse(Cursor { rest, off: 0 }).unwrap()
}

#[test]
fn test_1() {
    let src = r#"{{# unless flag}}{{{/ unless}}"#;
    let span = Span { lo: 17, hi: 18 };
    let expr: syn::Expr = parse_str("flag").unwrap();
    assert_eq!(
        parse(src),
        vec![S(
            Node::Helper(Box::new(Helper::Unless(
                ((false, false), (false, false)),
                S(Box::new(expr), Span { lo: 11, hi: 15 }),
                vec![S(Lit("", S("{", span), ""), span)]
            ))),
            Span { lo: 0, hi: 30 }
        )]
    );
}

#[test]
fn test_eat_safe() {
    let src = r#"{{{ var }}}"#;
    let span = Span {
        lo: 0,
        hi: src.len() as u32,
    };
    assert_eq!(
        parse(src),
        vec![S(
            Safe(
                WS,
                S(
                    Box::new(parse_str::<Expr>("var").unwrap()),
                    Span { lo: 4, hi: 7 },
                ),
            ),
            span,
        )]
    );

    let src = r#"{{{ fun() }}}"#;
    let span = Span {
        lo: 0,
        hi: src.len() as u32,
    };
    assert_eq!(
        parse(src),
        vec![S(
            Safe(
                WS,
                S(
                    Box::new(parse_str::<Expr>("fun()").unwrap()),
                    Span { lo: 4, hi: 9 },
                ),
            ),
            span,
        )]
    );

    let src = r#"{{{ fun(|a| a) }}}"#;
    let span = Span {
        lo: 0,
        hi: src.len() as u32,
    };
    assert_eq!(
        parse(src),
        vec![S(
            Safe(
                WS,
                S(
                    Box::new(parse_str::<Expr>("fun(|a| a)").unwrap()),
                    Span { lo: 4, hi: 14 },
                ),
            ),
            span,
        )]
    );

    let src = r#"{{{
            fun(|a| {
                {{ a }}
            })
        }}}"#;
    let span = Span {
        lo: 0,
        hi: src.len() as u32,
    };
    assert_eq!(
        parse(src),
        vec![S(
            Safe(
                WS,
                S(
                    Box::new(parse_str::<Expr>("fun(|a| {{{a}}})").unwrap()),
                    Span { lo: 16, hi: 64 },
                ),
            ),
            span,
        )]
    );
}

#[should_panic]
#[test]
fn test_eat_safe_panic() {
    let src = r#"{{ fn(|a| {{{a}}}) }}"#;
    parse(src);
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
                        Box::new(parse_str::<Expr>("foo").unwrap()),
                        Span { lo: 2, hi: 5 },
                    ),
                ),
                Span { lo: 0, hi: 7 },
            )]
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
                    Box::new(parse_str::<Expr>("name").unwrap()),
                    Span { lo: 5, hi: 9 },
                ),
                vec![
                    S(
                        Expr(
                            WS,
                            S(
                                Box::new(parse_str::<Expr>("first").unwrap()),
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
                                Box::new(parse_str::<Expr>("last").unwrap()),
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
        Box::new(parse_str::<Expr>("bar").unwrap()),
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
        Box::new(parse_str::<Expr>("bar").unwrap()),
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
        Box::new(parse_str::<Expr>("bar").unwrap()),
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
                        Box::new(parse_str::<Expr>("cond").unwrap()),
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
fn test_defined() {
    let src = "{{#foo bar}}hello{{/foo}}";
    assert_eq!(&src[12..17], "hello");
    let span = Span {
        lo: 0,
        hi: src.len() as u32,
    };
    assert_eq!(
        parse(src),
        vec![S(
            Helper(Box::new(Helper::Defined(
                (WS, WS),
                "foo",
                S(
                    Box::new(parse_str::<Expr>("bar").unwrap()),
                    Span { lo: 7, hi: 10 },
                ),
                vec![S(
                    Lit("", S("hello", Span { lo: 12, hi: 17 }), ""),
                    Span { lo: 12, hi: 17 },
                )],
            ))),
            span,
        )]
    );
}

#[test]
fn test_ws_expr() {
    let src = "{{~foo~}}";
    let span = Span {
        lo: 0,
        hi: src.len() as u32,
    };
    assert_eq!(
        parse(src),
        vec![S(
            Expr(
                (true, true),
                S(
                    Box::new(parse_str::<Expr>("foo").unwrap()),
                    Span { lo: 3, hi: 6 },
                ),
            ),
            span,
        )]
    );
    let src = "{{~ foo~}}";
    let span = Span {
        lo: 0,
        hi: src.len() as u32,
    };
    assert_eq!(
        parse(src),
        vec![S(
            Expr(
                (true, true),
                S(
                    Box::new(parse_str::<Expr>("foo").unwrap()),
                    Span { lo: 4, hi: 7 },
                ),
            ),
            span,
        )]
    );
    let src = "{{~ foo}}";
    let span = Span {
        lo: 0,
        hi: src.len() as u32,
    };
    assert_eq!(
        parse(src),
        vec![S(
            Expr(
                (true, false),
                S(
                    Box::new(parse_str::<Expr>("foo").unwrap()),
                    Span { lo: 4, hi: 7 },
                ),
            ),
            span,
        )]
    );
    let src = "{{foo    ~}}";
    let span = Span {
        lo: 0,
        hi: src.len() as u32,
    };
    assert_eq!(
        parse(src),
        vec![S(
            Expr(
                (false, true),
                S(
                    Box::new(parse_str::<Expr>("foo").unwrap()),
                    Span { lo: 2, hi: 5 },
                ),
            ),
            span,
        )]
    );
    let src = "{{~{foo }~}}";
    let span = Span {
        lo: 0,
        hi: src.len() as u32,
    };
    assert_eq!(
        parse(src),
        vec![S(
            Safe(
                (true, true),
                S(
                    Box::new(parse_str::<Expr>("foo").unwrap()),
                    Span { lo: 4, hi: 7 },
                ),
            ),
            span,
        )]
    );
    let src = "{{{foo }~}}";
    let span = Span {
        lo: 0,
        hi: src.len() as u32,
    };
    assert_eq!(
        parse(src),
        vec![S(
            Safe(
                (false, true),
                S(
                    Box::new(parse_str::<Expr>("foo").unwrap()),
                    Span { lo: 3, hi: 6 },
                ),
            ),
            span,
        )]
    );
}

#[test]
fn test_ws_each() {
    let src = "{{~#each bar~}}{{~/each~}}";
    let span = Span {
        lo: 0,
        hi: src.len() as u32,
    };
    assert_eq!(
        parse(src),
        vec![S(
            Helper(Box::new(Helper::Each(
                ((true, true), (true, true)),
                S(
                    Box::new(parse_str::<Expr>("bar").unwrap()),
                    Span { lo: 9, hi: 12 },
                ),
                vec![],
            ))),
            span,
        )]
    );
}

#[test]
fn test_ws_if() {
    let src = "{{~#if bar~}}{{~/if~}}";
    let span = Span {
        lo: 0,
        hi: src.len() as u32,
    };
    assert_eq!(
        parse(src),
        vec![S(
            Helper(Box::new(Helper::If(
                (
                    ((true, true), (true, true)),
                    S(
                        Box::new(parse_str::<Expr>("bar").unwrap()),
                        Span { lo: 7, hi: 10 },
                    ),
                    vec![],
                ),
                vec![],
                None,
            ))),
            span,
        )]
    );
}

#[test]
fn test_ws_if_else() {
    let src = "{{~#if bar~}}{{~else~}}{{~/if~}}";
    let span = Span {
        lo: 0,
        hi: src.len() as u32,
    };
    assert_eq!(
        parse(src),
        vec![S(
            Helper(Box::new(Helper::If(
                (
                    ((true, true), (true, true)),
                    S(
                        Box::new(parse_str::<Expr>("bar").unwrap()),
                        Span { lo: 7, hi: 10 },
                    ),
                    vec![],
                ),
                vec![],
                Some(((true, true), vec![])),
            ))),
            span,
        )]
    );
}

#[test]
fn test_ws_if_else_if() {
    let src = "{{~#if bar~}}{{~else if bar~}}{{~else~}}{{~/if~}}";
    let span = Span {
        lo: 0,
        hi: src.len() as u32,
    };
    assert_eq!(
        parse(src),
        vec![S(
            Helper(Box::new(Helper::If(
                (
                    ((true, true), (true, true)),
                    S(
                        Box::new(parse_str::<Expr>("bar").unwrap()),
                        Span { lo: 7, hi: 10 },
                    ),
                    vec![],
                ),
                vec![(
                    (true, true),
                    S(
                        Box::new(parse_str::<Expr>("bar").unwrap()),
                        Span { lo: 24, hi: 27 },
                    ),
                    vec![],
                )],
                Some(((true, true), vec![])),
            ))),
            span,
        )]
    );
}

#[test]
fn test_ws_raw() {
    let src = "{{~R~}}{{#some }}{{/some}}{{~/R ~}}";
    let span = Span {
        lo: 0,
        hi: src.len() as u32,
    };
    assert_eq!(
        parse(src),
        vec![S(
            Raw(
                ((true, true), (true, true)),
                "",
                S("{{#some }}{{/some}}", Span { lo: 7, hi: 26 }),
                "",
            ),
            span,
        )]
    );
    let src = "{{R  ~}}{{#some }}{{/some}}{{/R ~}}";
    let span = Span {
        lo: 0,
        hi: src.len() as u32,
    };
    assert_eq!(
        parse(src),
        vec![S(
            Raw(
                ((false, true), (false, true)),
                "",
                S("{{#some }}{{/some}}", Span { lo: 8, hi: 27 }),
                "",
            ),
            span,
        )]
    );
}

#[test]
fn test_partial_ws() {
    let src = "{{~> partial ~}}";
    let span = Span {
        lo: 0,
        hi: src.len() as u32,
    };

    assert_eq!(
        parse(src),
        vec![S(
            Node::Partial(Partial(
                (true, true),
                S("partial", Span { lo: 5, hi: 12 }),
                S(vec![], Span { lo: 13, hi: 13 }),
            )),
            span,
        )]
    );
    let src = "{{> partial scope }}";
    let span = Span {
        lo: 0,
        hi: src.len() as u32,
    };
    assert_eq!(
        parse(src),
        vec![S(
            Node::Partial(Partial(
                WS,
                S("partial", Span { lo: 4, hi: 11 }),
                S(
                    vec![parse_str::<Expr>("scope").unwrap()],
                    Span { lo: 12, hi: 17 },
                ),
            )),
            span,
        )]
    );
    let src = "{{> partial scope ~}}";
    let span = Span {
        lo: 0,
        hi: src.len() as u32,
    };
    assert_eq!(
        parse(src),
        vec![S(
            Node::Partial(Partial(
                (false, true),
                S("partial", Span { lo: 4, hi: 11 }),
                S(
                    vec![parse_str::<Expr>("scope").unwrap()],
                    Span { lo: 12, hi: 17 },
                ),
            )),
            span,
        )]
    );
}

#[test]
fn test_partial() {
    let src = "{{> partial }}";
    let span = Span {
        lo: 0,
        hi: src.len() as u32,
    };
    assert_eq!(
        parse(src),
        vec![S(
            Node::Partial(Partial(
                WS,
                S("partial", Span { lo: 4, hi: 11 }),
                S(vec![], Span { lo: 12, hi: 12 }),
            )),
            span,
        )]
    );
    let src = "{{> partial scope }}";
    let span = Span {
        lo: 0,
        hi: src.len() as u32,
    };
    assert_eq!(
        parse(src),
        vec![S(
            Node::Partial(Partial(
                WS,
                S("partial", Span { lo: 4, hi: 11 }),
                S(
                    vec![parse_str::<Expr>("scope").unwrap()],
                    Span { lo: 12, hi: 17 },
                ),
            )),
            span,
        )]
    );
}

#[test]
fn test_raw() {
    let src = "{{R}}{{#some }}{{/some}}{{/R}}";
    let span = Span {
        lo: 0,
        hi: src.len() as u32,
    };
    assert_eq!(
        parse(src),
        vec![S(
            Raw(
                (WS, WS),
                "",
                S("{{#some }}{{/some}}", Span { lo: 5, hi: 24 }),
                "",
            ),
            span,
        )]
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

#[test]
fn test_partial_block() {
    let rest = "{{> @partial-block }}";
    assert_eq!(
        parse(rest),
        vec![S(
            Node::Block((false, false)),
            Span {
                lo: 0,
                hi: rest.len() as u32
            }
        )]
    );
}

#[test]
fn test_partial_block_1() {
    let rest = "{{#> some }}foo{{/some }}";
    assert_eq!(
        parse(rest),
        vec![S(
            Node::PartialBlock(PartialBlock(
                ((false, false), (false, false)),
                S("some", bytes!(5..9)),
                S(vec![], bytes!(10..10)),
                vec![S(
                    Node::Lit("", S("foo", bytes!(12..15)), ""),
                    bytes!(12..15)
                ),]
            )),
            bytes!(0..25)
        )]
    );
}

#[test]
fn test_partial_block_ws() {
    let rest = "{{~> @partial-block ~}}";
    assert_eq!(
        parse(rest),
        vec![S(
            Node::Block((true, true)),
            Span {
                lo: 0,
                hi: rest.len() as u32
            }
        )]
    );
}

#[test]
fn test_partial_block_ws_1() {
    let rest = r#"Foo
    {{~> @partial-block ~}}
    Bar"#;
    assert_eq!(
        parse(rest),
        vec![
            S(Lit("", S("Foo", bytes!(0..3)), "\n    "), bytes!(0..8)),
            S(Block((true, true)), bytes!(8..31)),
            S(Lit("\n    ", S("Bar", bytes!(36..39)), ""), bytes!(31..39))
        ]
    );
}

#[test]
fn test_compile_error() {
    let rest = "{{$ \"foo\" }}";
    assert_eq!(
        parse(rest),
        vec![S(
            Error(S(vec![parse_str("\"foo\"").unwrap()], bytes!(4..9))),
            bytes!(0..12)
        )]
    );
}

#[test]
fn test_at_helpers() {
    let rest = "{{ @json foo }}";
    assert_eq!(
        parse(rest),
        vec![S(
            Node::AtHelper(
                (false, false),
                AtHelperKind::Json,
                S(vec![parse_str("foo").unwrap()], bytes!(9..12))
            ),
            bytes!(0..15)
        )]
    );

    let rest = "{{ @json_pretty foo }}";
    assert_eq!(
        parse(rest),
        vec![S(
            Node::AtHelper(
                (false, false),
                AtHelperKind::JsonPretty,
                S(vec![parse_str("foo").unwrap()], bytes!(16..19))
            ),
            bytes!(0..22)
        )]
    )
}

fn test_error(rest: &str, _message: PError, _span: Span) {
    let cursor = Cursor { rest, off: 0 };
    match _parse(cursor) {
        Err(ErrorMessage { message, span }) => {
            if _message != message || _span != span {
                panic!(
                        "\n\nExpect:\n\tmessage: {:?}\n\tspan: {:?}\n\nResult:\n\tmessage: {:?}\n\tspan: {:?}",
                        message.to_string(), span, _message.to_string(), _span
                    )
            }
        }
        _ => panic!(
            "\n\nIt's Ok rest: {:?}\n\nExpect:\n\tmessage: {:?}\n\tspan: {:?}",
            rest,
            _message.to_string(),
            _span
        ),
    };
}

#[test]
fn test_error_expr() {
    test_error(
        "{{ @ }}",
        PError::Expr(DOption::Some(String::from("expected expression"))),
        bytes!(3..4),
    );
}

#[test]
fn test_error_safe() {
    test_error(
        "{{{ @ }}}",
        PError::Safe(DOption::Some(String::from("expected expression"))),
        bytes!(4..5),
    );
}

#[test]
fn test_error_local() {
    test_error(
        "{{ let @ }}",
        PError::Local(DOption::Some(String::from("expected one of: identifier, `::`, `<`, `_`, literal, `ref`, `mut`, `&`, parentheses, square brackets, `..`"))),
        bytes!(7..8)
    );
    // TODO: Why `span-locations` runs here and not when use in derive
    let rest = String::from("{{ let") + " @ }}";
    test_error(
        &rest,
        PError::Local(DOption::Some(String::from("expected one of: identifier, `::`, `<`, `_`, literal, `ref`, `mut`, `&`, parentheses, square brackets, `..`"))),
        bytes!(7..8)
    );
}

#[test]
fn test_error_if() {
    test_error(
        "{{# if let @ }}{{/if }}",
        PError::Argument(DOption::Some(String::from("expected one of: identifier, `::`, `<`, `_`, literal, `ref`, `mut`, `&`, parentheses, square brackets, `..`"))),
        bytes!(11..12));
}

#[test]
fn test_error_expr_multiline() {
    test_error(
        "{{ foo\n\n.map(|x| x)\n\n   .bar(@)\n.foo() }}",
        PError::Expr(DOption::Some(String::from("expected expression"))),
        bytes!(29..30),
    );
}

#[test]
fn test_error_at_helper_not_exist() {
    test_error("{{ @foo }}", PError::AtHelperNotExist, bytes!(4..7));
}

#[test]
fn test_error_at_helper_check_len() {
    test_error(
        "{{ @json one, two }}",
        PError::AtHelperArgsLen(1),
        bytes!(9..17),
    );
}
