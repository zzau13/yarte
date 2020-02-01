use markup5ever::{local_name, namespace_url, ns};
use quote::quote;
use syn::parse2;

use yarte_hir::{Each as HEach, IfElse as HIfElse, Struct, HIR};
use yarte_html::{
    interface::{QualName, YName},
    serializer::SerializerOpt,
    utils::MARK,
    y_name,
};

use crate::{
    serialize::serialize,
    sink::{parse_document, parse_fragment, ParseAttribute, ParseElement, ParseResult, Sink},
};

pub struct DOMFmt(pub Vec<HIR>);

// TODO: to try from
impl From<Vec<HIR>> for DOMFmt {
    fn from(ir: Vec<HIR>) -> Self {
        DOMFmt(to_domfmt_init(ir).expect("correct html"))
    }
}

const HASH: &str = "0x00000000";

fn get_html(ir: &[HIR]) -> String {
    let mut html = String::new();
    for x in ir {
        match x {
            HIR::Lit(x) => html.push_str(x),
            _ => {
                html.push_str(MARK);
                html.push_str(HASH);
            }
        }
    }

    html
}

pub fn to_wasmfmt(mut ir: Vec<HIR>, s: &Struct) -> ParseResult<Vec<HIR>> {
    let html = get_html(&ir);
    let sink = match parse_document(&html) {
        Ok(mut sink) => {
            add_scripts(s, &mut sink, &mut ir);
            sink
        }
        Err(_) => parse_fragment(&html)?,
    };

    serialize_domfmt(sink, ir, SerializerOpt { wasm: true })
}

fn add_scripts(s: &Struct, sink: &mut Sink, ir: &mut Vec<HIR>) {
    let mut head: Option<usize> = None;
    use ParseElement::*;
    match sink.nodes.values().next() {
        Some(Document(children)) => {
            if let Some(Node { name, children, .. }) = sink.nodes.get(&children[0]) {
                if let y_name!("html") = name.local {
                    for i in children {
                        if let Some(Node { name, .. }) = sink.nodes.get(i) {
                            if let y_name!("head") = name.local {
                                head = Some(*i);
                            }
                        }
                    }
                }
            }
        }
        _ => panic!("Need <!doctype html>"),
    }

    let mut last = *sink.nodes.keys().last().unwrap() + 1;
    let get_state = format!(
        "function get_state(){{return JSON.stringify({}{});}}",
        MARK, HASH
    );

    let mut buf = vec![HIR::Safe(Box::new(
        parse2(quote!(
            yarte::serde_json::to_string(&self).map_err(|_| yarte::Error)?
        ))
        .unwrap(),
    ))];
    for i in ir.drain(..) {
        buf.push(i);
    }
    *ir = buf;

    let state = Node {
        name: QualName {
            prefix: None,
            ns: ns!(html),
            local: y_name!("script"),
        },
        attrs: vec![],
        children: vec![last],
        parent: None,
    };
    sink.nodes.insert(last, Text(get_state));
    last += 1;
    sink.nodes.insert(last, state);
    let state = last;
    last += 1;

    let init_s = format!(
        "import init from '{}';async function run(){{await init()}}run()",
        s.script.as_ref().expect("Need `script` attribute")
    );
    let init = Node {
        name: QualName {
            prefix: None,
            ns: ns!(html),
            local: y_name!("script"),
        },
        attrs: vec![ParseAttribute {
            name: QualName {
                prefix: None,
                ns: ns!(),
                local: y_name!("type"),
            },
            value: "module".to_string(),
        }],
        children: vec![last],
        parent: None,
    };
    sink.nodes.insert(last, Text(init_s));
    last += 1;
    sink.nodes.insert(last, init);
    let init = last;
    if let Some(head) = head {
        match sink.nodes.get_mut(&head).unwrap() {
            Node { children, .. } => {
                let mut n = vec![state, init];
                n.extend_from_slice(children);
                *children = n;
            }
            _ => unreachable!(),
        }
    } else {
        let head = Node {
            name: QualName {
                prefix: None,
                ns: ns!(html),
                local: y_name!("head"),
            },
            attrs: vec![],
            children: vec![state, init],
            parent: None,
        };
        last += 1;
        sink.nodes.insert(last, head);
        match sink.nodes.values_mut().next() {
            Some(Document(children)) => {
                let mut new = vec![last];
                new.extend_from_slice(children);
                *children = new;
            }
            _ => unreachable!(),
        }
    }
}

fn to_domfmt_init(ir: Vec<HIR>) -> ParseResult<Vec<HIR>> {
    let html = get_html(&ir);
    let sink = match parse_document(&html) {
        Ok(a) => a,
        Err(_) => parse_fragment(&html)?,
    };

    serialize_domfmt(sink, ir, Default::default())
}

fn to_domfmt(ir: Vec<HIR>, opts: SerializerOpt) -> ParseResult<Vec<HIR>> {
    let html = get_html(&ir);
    serialize_domfmt(parse_fragment(&html)?, ir, opts)
}

fn serialize_domfmt(sink: Sink, mut ir: Vec<HIR>, opts: SerializerOpt) -> ParseResult<Vec<HIR>> {
    let mut writer = Vec::new();
    serialize(&mut writer, &sink.into(), opts).expect("some serialize node");

    let html = String::from_utf8(writer).expect("");
    let mut chunks = html.split(MARK).peekable();

    if let Some(first) = chunks.peek() {
        if first.is_empty() {
            chunks.next();
        }
    }
    let mut ir = ir.drain(..).filter(|x| match x {
        HIR::Lit(_) => false,
        _ => true,
    });

    let mut buff = vec![];
    for chunk in chunks {
        if chunk.is_empty() {
            panic!("chunk empty")
        } else if chunk.starts_with(HASH) {
            resolve_node(ir.next().expect("Some HIR expression"), &mut buff, opts)?;
            let cut = &chunk[HASH.len()..];
            if !cut.is_empty() {
                buff.push(HIR::Lit(cut.into()));
            }
        } else {
            buff.push(HIR::Lit(chunk.into()));
        }
    }

    // Standard or empty case (with only comments,...)
    assert!(ir.next().is_none());

    Ok(buff)
}

fn resolve_node(ir: HIR, buff: &mut Vec<HIR>, opts: SerializerOpt) -> ParseResult<()> {
    match ir {
        HIR::Each(each) => {
            let HEach { args, body, expr } = *each;
            buff.push(HIR::Each(Box::new(HEach {
                args,
                expr,
                body: to_domfmt(body, opts)?,
            })))
        }
        HIR::IfElse(if_else) => {
            let HIfElse { ifs, if_else, els } = *if_else;
            let mut buf_if_else = vec![];
            for (expr, body) in if_else {
                buf_if_else.push((expr, to_domfmt(body, opts)?));
            }
            let els = if let Some(els) = els {
                Some(to_domfmt(els, opts)?)
            } else {
                None
            };
            buff.push(HIR::IfElse(Box::new(HIfElse {
                ifs: (ifs.0, to_domfmt(ifs.1, opts)?),
                if_else: buf_if_else,
                els,
            })));
        }
        HIR::Lit(_) => panic!("Need some node"),
        ir => buff.push(ir),
    }
    Ok(())
}
