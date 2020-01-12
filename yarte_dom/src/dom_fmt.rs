use yarte_hir::{Each as HEach, IfElse as HIfElse, HIR};

use crate::{
    serialize::serialize,
    sink::{parse_document, parse_fragment, ParseResult, Sink, HEAD, TAIL},
};

pub struct DOMFmt(pub Vec<HIR>);

// TODO: to try from
impl From<Vec<HIR>> for DOMFmt {
    fn from(ir: Vec<HIR>) -> Self {
        DOMFmt(to_domfmt(ir).expect("correct html"))
    }
}

const HASH: &str = "0x00000000";

fn to_domfmt(ir: Vec<HIR>) -> ParseResult<Vec<HIR>> {
    let mut html = String::new();
    for x in &ir {
        match x {
            HIR::Lit(x) => html.push_str(x),
            _ => {
                html.push_str(HEAD);
                html.push_str(HASH);
                html.push_str(TAIL);
            }
        }
    }

    let sink = match parse_document(&html) {
        Ok(a) => a,
        Err(_) => parse_fragment(&html)?,
    };

    serialize_domfmt(sink, ir)
}

fn serialize_domfmt(sink: Sink, mut ir: Vec<HIR>) -> ParseResult<Vec<HIR>> {
    let mut writer = Vec::new();
    serialize(&mut writer, &sink.into()).expect("some serialize node");

    let html = String::from_utf8(writer).expect("");
    let mut chunks = html.split(HEAD).peekable();

    if let Some(first) = chunks.peek() {
        if first.is_empty() {
            chunks.next();
        }
    }

    let mut buff = vec![];
    for chunk in chunks {
        if chunk.is_empty() {
            panic!("chunk empty")
        } else if chunk.starts_with(HASH) {
            resolve_node(ir.remove(0), &mut buff)?;
            let cut = &chunk[HASH.len() + TAIL.len()..];
            if !cut.is_empty() {
                buff.push(HIR::Lit(cut.into()));
                ir.remove(0);
            }
        } else {
            buff.push(HIR::Lit(chunk.into()));
            ir.remove(0);
        }
    }

    // Standard or empty case (with only comments,...)
    assert!(ir.is_empty() || (ir.len() == 1 && ir[0] == HIR::Lit("".into())));

    Ok(buff)
}

fn resolve_node(ir: HIR, buff: &mut Vec<HIR>) -> ParseResult<()> {
    match ir {
        HIR::Each(each) => {
            let HEach { args, body, expr } = *each;
            buff.push(HIR::Each(Box::new(HEach {
                args,
                expr,
                body: to_domfmt(body)?,
            })))
        }
        HIR::IfElse(if_else) => {
            let HIfElse { ifs, if_else, els } = *if_else;
            let mut buf_if_else = vec![];
            for (expr, body) in if_else {
                buf_if_else.push((expr, to_domfmt(body)?));
            }
            let els = if let Some(els) = els {
                Some(to_domfmt(els)?)
            } else {
                None
            };
            buff.push(HIR::IfElse(Box::new(HIfElse {
                ifs: (ifs.0, to_domfmt(ifs.1)?),
                if_else: buf_if_else,
                els,
            })));
        }
        HIR::Lit(_) => panic!("Need some node"),
        ir => buff.push(ir),
    }
    Ok(())
}
