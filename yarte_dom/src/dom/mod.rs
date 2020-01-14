#![allow(warnings)]

use std::{collections::HashMap, vec::Drain};

use markup5ever::{LocalName, QualName};
use syn::Local;
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
use crate::serializer::ElemInfo;

pub type Document = Vec<Node>;
pub type ExprId = usize;
pub type VarId = usize;

pub enum Var {
    This(String),
    Local(ExprId, String),
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
    ifs: ((ExprId, Option<VarId>, syn::Expr), Document),
    if_else: Vec<((ExprId, Option<VarId>, syn::Expr), Document)>,
    els: Option<Document>,
}

/// `for expr in args `
///
pub struct Each {
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
    stack: Vec<ElemInfo>,
    inner: bool,
    count: usize,
    tree_map: HashMap<ExprId, Vec<VarId>>,
    var_map: HashMap<VarId, Var>,
}

// 0x00_00_00_00
const HASH_LEN: usize = 10;

impl DOMBuilder {
    fn build(mut self, ir: Vec<HIR>) -> DOM {
        DOM {
            doc: self.init(ir).expect("Dom builder"),
            tree_map: self.tree_map,
            var_map: self.var_map,
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
                h => {
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

        let sink = match parse_document(&html) {
            Ok(a) => a,
            Err(_) => parse_fragment(&html)?,
        };

        self.serialize(sink, ir)
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
                    vec![self.resolve_node(name, attrs, children)?]
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
    ) -> ParseResult<Node> {
        todo!()
    }

    fn resolve_mark(&mut self, id: &str, ir: &mut Drain<HIR>) -> ParseResult<Node> {
        assert_eq!(id.len(), 10);
        assert_eq!(&id[..2], "0x");
        let id = u32::from_str_radix(&id[2..], 16).unwrap() as usize;
        let ir = ir.next().expect("Some HIR");

        match ir {
            HIR::Expr(e) => {
                resolve_expr(&e, id, self);
                Ok(Node::Expr(Expression::Unsafe(id, e)))
            }
            HIR::Safe(e) => {
                resolve_expr(&e, id, self);
                Ok(Node::Expr(Expression::Safe(id, e)))
            }
            HIR::Local(e) => {
                let var_id = resolve_local(&e, id, self);
                Ok(Node::Expr(Expression::Local(id, var_id, e)))
            }
            HIR::Each(e) => {
                resolve_each(&e, id, self);
                let HEach { args, body, expr } = *e;
                Ok(Node::Expr(Expression::Each(
                    id,
                    Box::new(Each {
                        args,
                        body: self.step(body)?,
                        expr,
                    }),
                )))
            }
            HIR::IfElse(e) => {
                resolve_if_else(&e, id, self);
                let HIfElse { ifs, if_else, els } = *e;
                todo!()
            }
            HIR::Lit(_) => unreachable!(),
        }
    }

    fn resolve_text(&mut self, s: &str) -> Node {
        todo!()
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
                } => self.resolve_node(name, attrs, children)?,
                ParseElement::Document(_) => unreachable!(),
            })
        }

        Ok(buff)
    }
}
