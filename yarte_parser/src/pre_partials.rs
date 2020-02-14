use crate::{
    comment,
    error::PError,
    expr_partial_block, partial, raw,
    source_map::Span,
    strnom::{Cursor, LexError, PResult},
    ErrorMessage, Partial,
};

pub fn parse_partials(rest: &str) -> Result<Vec<Partial>, ErrorMessage<PError>> {
    let (c, res) = eat_partials(Cursor { rest, off: 0 })?;
    if c.is_empty() {
        Ok(res)
    } else {
        let end = (rest.len() - c.len()) as u32;
        Err(ErrorMessage {
            message: PError::Uncompleted,
            span: Span { lo: end, hi: end },
        })
    }
}

fn eat_partials(mut i: Cursor) -> PResult<Vec<Partial>> {
    let mut nodes = vec![];

    loop {
        if let Some(j) = i.find('{') {
            macro_rules! _switch {
                ($n:expr, $t:expr, $ws:expr) => {
                    match $n {
                        b'>' => {
                            let i = i.adv(j + 3 + $t);
                            match partial_block(i, $ws) {
                                Ok((i, n)) => {
                                    nodes.push(n);
                                    i
                                }
                                Err(e @ LexError::Fail(..)) => break Err(e),
                                Err(LexError::Next(..)) => i,
                            }
                        }
                        b'R' => {
                            let i = i.adv(j + 3 + $t);
                            match raw(i, $ws) {
                                Ok((i, _)) => i,
                                Err(e @ LexError::Fail(..)) => break Err(e),
                                Err(LexError::Next(..)) => i,
                            }
                        }
                        b'!' => {
                            let i = i.adv(j + 3);
                            match comment(i) {
                                Ok((i, _)) => i,
                                Err(_) => i,
                            }
                        }
                        b'#' if i.adv(j + 3 + $t).starts_with(">") => {
                            match partial(i.adv(j + 4 + $t), $ws) {
                                Ok((i, n)) => {
                                    nodes.push(n);
                                    i
                                }
                                Err(e @ LexError::Fail(..)) => break Err(e),
                                Err(LexError::Next(..)) => i.adv(j + 3 + $t),
                            }
                        }
                        _ => i.adv(j + 2 + $t),
                    }
                };
            }
            let n = i.rest[j + 1..].as_bytes();
            i = if 2 < n.len() && n[0] == b'{' {
                if n[1] == b'~' {
                    _switch!(n[2], 1, true)
                } else {
                    _switch!(n[1], 0, false)
                }
            } else {
                // next
                i.adv(j + 1)
            };
        } else {
            break Ok((i.adv(i.len()), nodes));
        }
    }
}

#[inline]
fn partial_block(i: Cursor, lws: bool) -> PResult<Partial> {
    match expr_partial_block(i, lws) {
        Ok(_) => Err(LexError::Next(PError::PartialBlock, Span::from(i))),
        Err(_) => partial(i, lws),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_map::{Span, S};

    #[test]
    fn test_empty() {
        let src = r#""#;
        assert_eq!(parse_partials(src).unwrap(), vec![]);
        let src = r#"{{/"#;
        assert_eq!(parse_partials(src).unwrap(), vec![]);
        let src = r#"{{"#;
        assert_eq!(parse_partials(src).unwrap(), vec![]);
        let src = r#"{"#;
        assert_eq!(parse_partials(src).unwrap(), vec![]);
        let src = r#"{{>"#;
        assert_eq!(parse_partials(src).unwrap(), vec![]);
        let src = r#"{{>}}"#;
        assert_eq!(parse_partials(src).unwrap(), vec![]);
        let src = r#"{{! {{> foo }} !}}"#;
        assert_eq!(parse_partials(src).unwrap(), vec![]);
        let src = r#"{{R}} {{> foo }} {{/R}}"#;
        assert_eq!(parse_partials(src).unwrap(), vec![]);
    }

    #[test]
    fn test_partial_block() {
        let src = "{{#> foo }}bar{{/foo }}";
        assert_eq!(
            parse_partials(src).unwrap(),
            vec![Partial(
                (false, false),
                S("foo", Span { lo: 5, hi: 8 }),
                S(vec![], Span { lo: 9, hi: 9 })
            )]
        );
    }
}
