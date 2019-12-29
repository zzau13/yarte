use super::{
    comment, partial, raw,
    strnom::{Cursor, LexError, PResult},
    Partial,
};

pub fn parse_partials(rest: &str) -> Vec<Partial> {
    match eat_partials(Cursor { rest, off: 0 }) {
        Ok((l, res)) => {
            if l.is_empty() {
                return res;
            }
            panic!(
                "problems pre parsing partials at template source: {:?}",
                l.rest
            );
        }
        Err(LexError::Fail) | Err(LexError::Next) => {
            panic!("problems pre parsing partials at template source")
        }
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
                            match partial(i, $ws) {
                                Ok((i, n)) => {
                                    nodes.push(n);
                                    i
                                }
                                Err(LexError::Fail) => break Err(LexError::Fail),
                                Err(LexError::Next) => i,
                            }
                        }
                        b'R' => {
                            let i = i.adv(j + 3 + $t);
                            match raw(i, $ws) {
                                Ok((i, _)) => i,
                                Err(LexError::Fail) => break Err(LexError::Fail),
                                Err(LexError::Next) => i,
                            }
                        }
                        b'!' => {
                            let i = i.adv(j + 3);
                            match comment(i) {
                                Ok((i, _)) => i,
                                Err(_) => i,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let src = r#""#;
        assert_eq!(parse_partials(src), vec![]);
        let src = r#"{{/"#;
        assert_eq!(parse_partials(src), vec![]);
        let src = r#"{{"#;
        assert_eq!(parse_partials(src), vec![]);
        let src = r#"{"#;
        assert_eq!(parse_partials(src), vec![]);
        let src = r#"{{>"#;
        assert_eq!(parse_partials(src), vec![]);
        let src = r#"{{>}}"#;
        assert_eq!(parse_partials(src), vec![]);
        let src = r#"{{! {{> foo }} !}}"#;
        assert_eq!(parse_partials(src), vec![]);
        let src = r#"{{R}} {{> foo }} {{/R}}"#;
        assert_eq!(parse_partials(src), vec![]);
    }
}
