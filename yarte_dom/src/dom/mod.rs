#![allow(warnings)]

use std::{collections::HashMap, vec::Drain};

use markup5ever::{namespace_url, ns, LocalName, QualName};
use yarte_hir::{Each as HEach, IfElse as HIfElse, HIR};

mod visit_each;
mod visit_expr;
mod visit_if_else;
mod visit_local;

use crate::{
    sink::{
        parse_document, parse_fragment, ParseAttribute, ParseElement, ParseNodeId, ParseResult,
        Sink, HEAD, TAIL,
    },
    tree_builder::YARTE_TAG,
};

use self::{
    visit_each::resolve_each, visit_expr::resolve_expr, visit_if_else::resolve_if_else,
    visit_local::resolve_local,
};
use std::collections::BTreeMap;

pub type Document = Vec<Node>;
pub type ExprId = usize;
pub type VarId = usize;

pub enum Var {
    This(Option<VarId>, String),
    Local(Option<VarId>, ExprId, String),
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

pub struct IfBlock {
    pub vars: Vec<VarId>,
    pub expr: syn::Expr,
    pub block: Document,
}

pub struct IfElse {
    pub ifs: IfBlock,
    pub if_else: Vec<IfBlock>,
    pub els: Option<Document>,
}

/// `for expr in args `
///
pub struct Each {
    pub var: VarId,
    pub args: syn::Expr,
    pub body: Document,
    pub expr: syn::Expr,
}

pub enum Ns {
    Html,
    Svg,
}

pub enum Element {
    Node {
        name: (Ns, LocalName),
        attrs: Vec<Attribute>,
        children: Document,
    },
    Text(String),
}

pub struct Attribute {
    pub name: String,
    pub value: Vec<ExprOrText>,
}

pub enum ExprOrText {
    Text(String),
    Expr(Expression),
}

pub struct DOM {
    pub doc: Document,
    pub tree_map: HashMap<ExprId, Vec<VarId>>,
    pub var_map: HashMap<VarId, Var>,
}

impl From<Vec<HIR>> for DOM {
    fn from(ir: Vec<HIR>) -> Self {
        DOMBuilder::default().build(ir)
    }
}

#[derive(Default)]
pub struct DOMBuilder {
    inner: bool,
    count: usize,
    tree_map: HashMap<ExprId, Vec<VarId>>,
    var_map: BTreeMap<VarId, Var>,
}

// 0x00_00_00_00
const HASH_LEN: usize = 10;

impl DOMBuilder {
    fn build(mut self, ir: Vec<HIR>) -> DOM {
        DOM {
            doc: self.init(ir).expect("Dom builder"),
            tree_map: self.tree_map,
            var_map: self.var_map.into_iter().collect(),
        }
    }

    fn generate_html(&mut self, ir: Vec<HIR>) -> (Vec<HIR>, String) {
        let mut html = String::new();
        let ir: Vec<HIR> = ir
            .into_iter()
            .filter(|x| match x {
                HIR::Lit(x) => {
                    html.push_str(x);
                    false
                }
                _ => {
                    html.push_str(HEAD);
                    let id = self.count;
                    self.count += 1;
                    html.push_str(&format!("{:#08x?}", id));
                    html.push_str(TAIL);
                    true
                }
            })
            .collect();

        (ir, html)
    }

    fn init(&mut self, ir: Vec<HIR>) -> ParseResult<Document> {
        let (ir, html) = self.generate_html(ir);
        self.serialize(parse_document(&html)?, ir)
    }

    fn step(&mut self, ir: Vec<HIR>) -> ParseResult<Document> {
        let (ir, html) = self.generate_html(ir);
        self.serialize(parse_fragment(&html)?, ir)
    }

    fn serialize(&mut self, sink: Sink, mut ir: Vec<HIR>) -> ParseResult<Document> {
        let mut ir = ir.drain(..);

        let nodes = match sink.nodes.values().next() {
            Some(ParseElement::Document(children)) => {
                self.inner = true;
                self.get_children(children, &sink, &mut ir)?
            }
            Some(ParseElement::Node {
                name,
                attrs,
                children,
                ..
            }) => {
                if name == &*YARTE_TAG {
                    if self.inner {
                        panic!("not use <{}> tag", &*YARTE_TAG.local);
                    }
                    self.inner = true;
                    self.get_children(children, &sink, &mut ir)?
                } else {
                    vec![self.resolve_node(name, attrs, children, &sink, &mut ir)?]
                }
            }
            Some(ParseElement::Text(s)) => vec![self.resolve_text(s)],
            Some(ParseElement::Mark(s)) => vec![self.resolve_mark(s, &mut ir)?],
            None => vec![],
        };

        assert!(ir.next().is_none());

        Ok(nodes)
    }

    fn resolve_node(
        &mut self,
        name: &QualName,
        attrs: &[ParseAttribute],
        children: &[ParseNodeId],
        sink: &Sink,
        ir: &mut Drain<HIR>,
    ) -> ParseResult<Node> {
        let ns = match name.ns {
            ns!(html) => Ns::Html,
            ns!(svg) => Ns::Svg,
            _ => panic!("Name space"),
        };

        Ok(Node::Elem(Element::Node {
            name: (ns, name.local.clone()),
            attrs: self.resolve_attrs(attrs, ir)?,
            children: self.get_children(children, sink, ir)?,
        }))
    }

    fn resolve_attrs(
        &mut self,
        attrs: &[ParseAttribute],
        ir: &mut Drain<HIR>,
    ) -> ParseResult<Vec<Attribute>> {
        let mut buff = vec![];
        for attr in attrs {
            buff.push(self.resolve_attr(attr, ir)?);
        }

        Ok(buff)
    }

    fn resolve_attr(
        &mut self,
        attr: &ParseAttribute,
        ir: &mut Drain<HIR>,
    ) -> ParseResult<Attribute> {
        let name = attr.name.local.to_string();
        let mut chunks = attr.value.split(HEAD).peekable();
        if let Some(first) = chunks.peek() {
            if first.is_empty() {
                chunks.next();
            }
        }
        let mut value = vec![];
        for chunk in chunks {
            if HASH_LEN < chunk.len() && &chunk[..2] == "0x" {
                if let Ok(id) = u32::from_str_radix(&chunk[2..HASH_LEN], 16).map(|x| x as usize) {
                    if self.tree_map.contains_key(&id) && chunk[HASH_LEN..].starts_with(TAIL) {
                        value.push(ExprOrText::Expr(self.resolve_expr(id, ir)?));
                        if !&chunk[HASH_LEN + TAIL.len()..].is_empty() {
                            value.push(ExprOrText::Text(chunk[HASH_LEN + TAIL.len()..].into()))
                        }

                        continue;
                    }
                }
            }

            value.push(ExprOrText::Text(chunk.into()))
        }

        Ok(Attribute { name, value })
    }

    fn resolve_mark(&mut self, id: &str, ir: &mut Drain<HIR>) -> ParseResult<Node> {
        assert_eq!(id.len(), 10);
        assert_eq!(&id[..2], "0x");
        let id = u32::from_str_radix(&id[2..], 16).unwrap() as usize;

        Ok(Node::Expr(self.resolve_expr(id, ir)?))
    }

    fn resolve_expr(&mut self, id: ExprId, ir: &mut Drain<HIR>) -> ParseResult<Expression> {
        let ir = ir.next().expect("Some HIR");

        match ir {
            HIR::Expr(e) => {
                resolve_expr(&e, id, self);
                Ok(Expression::Unsafe(id, e))
            }
            HIR::Safe(e) => {
                resolve_expr(&e, id, self);
                Ok(Expression::Safe(id, e))
            }
            HIR::Local(e) => {
                let var_id = resolve_local(&e, id, self);
                Ok(Expression::Local(id, var_id, e))
            }
            HIR::Each(e) => {
                resolve_each(&e, id, self);
                let HEach { args, body, expr } = *e;

                todo!()
            }
            HIR::IfElse(e) => {
                resolve_if_else(&e, id, self);
                let HIfElse { ifs, if_else, els } = *e;

                let (_expr, body) = ifs;
                let _body = self.step(body)?;

                let mut buff = vec![];
                for (expr, body) in if_else {
                    buff.push((expr, self.step(body)?))
                }

                let _els = if let Some(body) = els {
                    Some(self.step(body)?)
                } else {
                    None
                };

                todo!()
            }
            HIR::Lit(_) => unreachable!(),
        }
    }

    fn resolve_text(&mut self, s: &str) -> Node {
        Node::Elem(Element::Text(s.to_owned()))
    }

    fn get_children(
        &mut self,
        children: &[ParseNodeId],
        sink: &Sink,
        ir: &mut Drain<HIR>,
    ) -> ParseResult<Document> {
        let mut buff = vec![];
        for child in children.iter().map(|x| sink.nodes.get(x).unwrap()) {
            buff.push(match child {
                ParseElement::Text(s) => self.resolve_text(s),
                ParseElement::Mark(s) => self.resolve_mark(s, ir)?,
                ParseElement::Node {
                    name,
                    attrs,
                    children,
                    ..
                } => self.resolve_node(name, attrs, children, sink, ir)?,
                ParseElement::Document(_) => unreachable!(),
            })
        }

        Ok(buff)
    }
}
