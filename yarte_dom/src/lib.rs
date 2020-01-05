#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unreachable_code)]

use std::collections::HashMap;

use yarte_hir::{Each as HEach, IfElse as HIfElse, HIR};

#[macro_use]
mod macros;
mod driver;
mod parser;
mod serializer;
mod tree_builder;

use self::{
    parser::{parse_document, parse_fragment, ParseResult, Sink, HEAD, TAIL},
    serializer::{serialize, TreeElement},
};

pub type Document = Vec<Node>;
pub type ExprId = usize;
pub type VarId = usize;

pub enum Var {
    This(String),
    Local(ExprId, String),
}

pub struct DOM {
    doc: Document,
    tree_map: HashMap<ExprId, Vec<VarId>>,
    var_map: HashMap<VarId, Var>,
}

pub struct DOMFmt(pub Vec<HIR>);

impl From<Vec<HIR>> for DOM {
    fn from(ir: Vec<HIR>) -> Self {
        let doc = vec![];
        let tree_map = HashMap::new();
        let var_map = HashMap::new();

        todo!();

        DOM {
            doc,
            tree_map,
            var_map,
        }
    }
}

// TODO: to try from
impl From<Vec<HIR>> for DOMFmt {
    fn from(ir: Vec<HIR>) -> Self {
        DOMFmt(to_domfmt(ir).expect("correct html"))
    }
}

const HASH_LEN: usize = 10;
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
    for i in Into::<Vec<TreeElement>>::into(sink) {
        serialize(&mut writer, &i).expect("some serialize node")
    }

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

pub enum Node {
    Elem(Element),
    Expr(Expression),
}

pub enum Expression {
    Unsafe(ExprId, Box<syn::Expr>),
    Safe(ExprId, Box<syn::Expr>),
    Each(ExprId, Box<Each>),
    IfElse(ExprId, Box<IfElse>),
    Local(ExprId, VarId, Box<syn::Local>),
}

#[allow(clippy::type_complexity)]
pub struct IfElse {
    ifs: ((ExprId, Option<VarId>, syn::Expr), Vec<Node>),
    if_else: Vec<((ExprId, Option<VarId>, syn::Expr), Vec<Node>)>,
    els: Option<(ExprId, Vec<Node>)>,
}

/// `for expr in args `
///
pub struct Each {
    args: (ExprId, syn::Expr),
    body: Vec<Node>,
    expr: (ExprId, VarId, syn::Expr),
}

pub enum Ns {
    Html,
    Svg,
}

pub enum Element {
    Node {
        name: (Ns, String),
        attrs: Vec<Attribute>,
        children: Vec<Node>,
    },
    Text(String),
}

pub struct Attribute {
    name: String,
    value: Vec<ExprOrText>,
}

pub enum ExprOrText {
    Text(String),
    Expr(Expression),
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::serializer::serialize;

    #[test]
    fn test_div() {
        let src = "<div attr=\"some\" \t class=\"any\"    \n>Hi!<br   /></div><div \
                   some7Na=\"hola\">hi</div>";
        let expected =
            "<div attr=\"some\" class=\"any\">Hi!<br></div><div some7na=\"hola\">hi</div>";

        let a = parse_fragment(src).unwrap();
        let tree_elem: Vec<TreeElement> = a.into();
        let mut writer = Vec::new();
        for i in tree_elem {
            serialize(&mut writer, &i).expect("some serialize node")
        }

        let html = String::from_utf8(writer).expect("");

        assert_eq!(expected, html);
    }

    #[test]
    fn test_table() {
        let src = "<table><!--yarteHashHTMLExpressionsATTT0x00000000--></table>";
        let expected = "<table><!--yarteHashHTMLExpressionsATTT0x00000000--></table>";

        let a = parse_fragment(src).unwrap();
        let tree_elem: Vec<TreeElement> = a.into();
        let mut writer = Vec::new();
        for i in tree_elem {
            serialize(&mut writer, &i).expect("some serialize node")
        }

        let html = String::from_utf8(writer).expect("");

        assert_eq!(expected, html);
    }

    #[test]
    fn test_attributes() {
        let src = "<div class=\"<!--yarteHashHTMLExpressionsATTT0x00000000-->\"></div>";
        let expected = "<div class=\"<!--yarteHashHTMLExpressionsATTT0x00000000-->\"></div>";

        let a = parse_fragment(src).unwrap();
        let tree_elem: Vec<TreeElement> = a.into();
        let mut writer = Vec::new();
        for i in tree_elem {
            serialize(&mut writer, &i).expect("some serialize node")
        }

        let html = String::from_utf8(writer).expect("");

        assert_eq!(expected, html);
    }

    #[test]
    fn test_document_err() {
        let src = "<div class=\"<!--yarteHashHTMLExpressionsATTT0x00000000-->\"></div>";

        assert!(parse_document(src).is_err());
    }

    #[test]
    fn test_document_ok() {
        let src = "<html><body><div \
                   class=\"<!--yarteHashHTMLExpressionsATTT0x00000000-->\"></div></body></html>";
        let expected = "<!DOCTYPE html><html><body><div \
                        class=\"<!--yarteHashHTMLExpressionsATTT0x00000000-->\"></div></body></\
                        html>";

        let a = parse_document(src).unwrap();
        let tree_elem: Vec<TreeElement> = a.into();
        let mut writer = Vec::new();
        for i in tree_elem {
            serialize(&mut writer, &i).expect("some serialize node")
        }

        let html = String::from_utf8(writer).expect("");

        assert_eq!(expected, html);
    }

    #[test]
    fn test_document_ok_doctype() {
        let src = "<!DOCTYPE html><html><body><div \
                   class=\"<!--yarteHashHTMLExpressionsATTT0x00000000-->\"></div></body></html>";
        let expected = "<!DOCTYPE html><html><body><div \
                        class=\"<!--yarteHashHTMLExpressionsATTT0x00000000-->\"></div></body></\
                        html>";

        let a = parse_document(src).unwrap();
        let tree_elem: Vec<TreeElement> = a.into();
        let mut writer = Vec::new();
        for i in tree_elem {
            serialize(&mut writer, &i).expect("some serialize node")
        }

        let html = String::from_utf8(writer).expect("");

        assert_eq!(expected, html);
    }

    #[test]
    fn test_document_ok_table() {
        let src = "<html><body><table><!--yarteHashHTMLExpressionsATTT0x00000000--></table></\
                   body></html>";
        let expected = "<!DOCTYPE html><html><body><table><!\
                        --yarteHashHTMLExpressionsATTT0x00000000--></table></body></html>";

        let a = parse_document(src).unwrap();
        let tree_elem: Vec<TreeElement> = a.into();
        let mut writer = Vec::new();
        for i in tree_elem {
            serialize(&mut writer, &i).expect("some serialize node")
        }

        let html = String::from_utf8(writer).expect("");

        assert_eq!(expected, html);
    }

    #[test]
    fn test_document_ok_head() {
        let src = "<html><head><title><!--yarteHashHTMLExpressionsATTT0x00000000--></title></\
                   head><body><div attr=\"some\" \t class=\"any\"    \n>Hi!<br   /></div><div \
                   some7Na=\"hola\">hi</div></body></html>";
        let expected = "<!DOCTYPE html><html><head><title><!\
                        --yarteHashHTMLExpressionsATTT0x00000000--></title></head><body><div \
                        attr=\"some\" class=\"any\">Hi!<br></div><div \
                        some7na=\"hola\">hi</div></body></html>";

        let a = parse_document(src).unwrap();
        let tree_elem: Vec<TreeElement> = a.into();
        let mut writer = Vec::new();
        for i in tree_elem {
            serialize(&mut writer, &i).expect("some serialize node")
        }

        let html = String::from_utf8(writer).expect("");

        assert_eq!(expected, html);
    }
}
