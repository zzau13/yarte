use memchr::memchr;

use std::str::from_utf8;

use super::{partial, raw, Input, Node};

pub(crate) fn parse_partials(src: &str) -> Vec<Node> {
    match eat_partials(Input(src.as_bytes())) {
        Ok((l, res)) => {
            if l.0.is_empty() {
                return res;
            }
            panic!(
                "problems pre parsing partials at template source: {:?}",
                from_utf8(l.0).unwrap()
            );
        }
        Err(nom::Err::Error(err)) | Err(nom::Err::Failure(err)) => panic!(
            "problems pre parsing partials at template source: {:?}",
            err
        ),
        Err(nom::Err::Incomplete(_)) => panic!("pre partials parsing incomplete"),
    }
}

fn eat_partials(mut i: Input) -> Result<(Input, Vec<Node>), nom::Err<Input>> {
    let mut nodes = vec![];

    loop {
        if let Some(j) = memchr(b'{', i.0) {
            let n = &i[j + 1..];
            macro_rules! _switch {
                ($n:expr, $t:expr, $ws:expr) => {
                    match $n {
                        b'>' => {
                            let i = Input(&i[j + 3 + $t..]);
                            match partial(i, $ws) {
                                Ok((i, n)) => {
                                    nodes.push(n);
                                    i
                                }
                                Err(nom::Err::Failure(err)) => break Err(nom::Err::Failure(err)),
                                Err(_) => i,
                            }
                        }
                        b'R' => {
                            let i = Input(&i[j + 3 + $t..]);
                            match raw(i, $ws) {
                                Ok((i, _)) => i,
                                Err(nom::Err::Failure(err)) => break Err(nom::Err::Failure(err)),
                                Err(_) => i,
                            }
                        }
                        _ => Input(&n[1 + $t..]),
                    }
                };
            }

            i = if 2 < n.len() && n[0] == b'{' {
                if n[1] == b'~' {
                    _switch!(n[2], 1, true)
                } else {
                    _switch!(n[1], 0, false)
                }
            } else {
                // next
                Input(n)
            }
        } else {
            break Ok((Input(&[]), nodes));
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
    }
}
