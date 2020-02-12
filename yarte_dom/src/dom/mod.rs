#![allow(warnings)]

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    vec::Drain,
};

use markup5ever::{namespace_url, ns, LocalName};
use syn::parse_str;

use yarte_hir::{Each as HEach, IfElse as HIfElse, HIR};
use yarte_html::{
    interface::{QualName, YName},
    tree_builder::{get_marquee, is_marquee},
    utils::{get_mark_id, parse_id, HASH_LEN, MARK},
};

use crate::sink::{
    parse_document, parse_fragment, ParseAttribute, ParseElement, ParseNodeId, ParseResult, Sink,
};

mod resolve;

use self::resolve::{resolve_each, resolve_expr, resolve_if_block, resolve_local};

pub type Document = Vec<Node>;
pub type ExprId = usize;
pub type VarId = u64;

#[derive(Debug, PartialEq)]
pub struct VarInner {
    pub base: VarId,
    pub ident: String,
}

#[derive(Debug, PartialEq)]
pub enum Var {
    This(VarInner),
    Local(ExprId, VarInner),
}

#[derive(Debug, PartialEq)]
pub enum Node {
    Elem(Element),
    Expr(Expression),
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Unsafe(ExprId, Box<syn::Expr>),
    Safe(ExprId, Box<syn::Expr>),
    Each(ExprId, Box<Each>),
    IfElse(ExprId, Box<IfElse>),
    Local(ExprId, VarId, Box<syn::Local>),
}

#[derive(Debug, PartialEq)]
pub struct IfBlock {
    pub vars: Vec<VarId>,
    pub expr: syn::Expr,
    pub block: Document,
}

#[derive(Debug, PartialEq)]
pub struct IfElse {
    pub ifs: IfBlock,
    pub if_else: Vec<IfBlock>,
    pub els: Option<Document>,
}

/// `for expr in args `
///
#[derive(Debug, PartialEq)]
pub struct Each {
    pub var: (VarId, Option<VarId>),
    pub args: syn::Expr,
    pub body: Document,
    pub expr: syn::Expr,
}

#[derive(Debug, PartialEq)]
pub enum Ns {
    Html,
    Svg,
}

#[derive(Debug, PartialEq)]
pub enum Element {
    Node {
        name: (Ns, ExprOrText),
        attrs: Vec<Attribute>,
        children: Document,
    },
    Text(String),
}

#[derive(Debug, PartialEq)]
pub struct Attribute {
    pub name: ExprOrText,
    pub value: Vec<ExprOrText>,
}

#[derive(Debug, PartialEq)]
pub enum ExprOrText {
    Text(String),
    Expr(Expression),
}

pub type TreeMap = BTreeMap<ExprId, BTreeSet<VarId>>;
pub type VarMap = HashMap<VarId, Var>;

#[derive(Debug)]
pub struct DOM {
    pub doc: Document,
    pub tree_map: TreeMap,
    pub var_map: VarMap,
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
    tree_map: TreeMap,
    var_map: VarMap,
}

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
                    html.push_str(MARK);
                    let id = self.count;
                    self.count += 1;
                    html.push_str(&format!("{:#010x?}", id));
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
        self.inner = false;
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
                if is_marquee(name) {
                    if self.inner {
                        panic!("not use <{}> tag", &*get_marquee().local);
                    }
                    self.inner = true;
                    self.get_children(children, &sink, &mut ir)?
                } else {
                    vec![self.resolve_node(name, attrs, children, &sink, &mut ir)?]
                }
            }
            Some(ParseElement::Text(s)) => vec![self.resolve_text(s)],
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
            name: (ns, self.resolve_y_name(&name.local, ir)?),
            attrs: self.resolve_attrs(attrs, ir)?,
            children: self.get_children(children, sink, ir)?,
        }))
    }

    fn resolve_y_name(&mut self, name: &YName, ir: &mut Drain<HIR>) -> ParseResult<ExprOrText> {
        Ok(match name {
            YName::Expr(s) => {
                let id = get_mark_id(&*s).expect("Valid mark") as usize;
                ExprOrText::Expr(self.resolve_expr(id, ir)?)
            }
            YName::Local(s) => ExprOrText::Text((&*s).to_string()),
        })
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
        let name = self.resolve_y_name(&attr.name.local, ir)?;
        // Event
        if let ExprOrText::Text(s) = &name {
            if s.starts_with("on") {
                let msg: syn::Expr = parse_str(&attr.value).expect("expression in on attribute");
                let var = resolve_expr(&msg, self);
                let id = self.count;
                self.count += 1;
                self.tree_map.insert(id, var.into_iter().collect());

                return Ok(Attribute {
                    name,
                    value: vec![ExprOrText::Expr(Expression::Safe(id, Box::new(msg)))],
                });
            }
        }
        // Attribute
        let mut chunks = attr.value.split(MARK).peekable();
        if let Some(first) = chunks.peek() {
            if first.is_empty() {
                chunks.next();
            }
        }
        let mut value = vec![];
        for chunk in chunks {
            if HASH_LEN < chunk.len() && &chunk[..2] == "0x" {
                if let Ok(id) = u32::from_str_radix(&chunk[2..HASH_LEN], 16).map(|x| x as usize) {
                    if self.tree_map.contains_key(&id) {
                        value.push(ExprOrText::Expr(self.resolve_expr(id, ir)?));
                        if !&chunk[HASH_LEN..].is_empty() {
                            value.push(ExprOrText::Text(chunk[HASH_LEN..].into()))
                        }

                        continue;
                    }
                }
            }

            value.push(ExprOrText::Text(chunk.into()))
        }

        Ok(Attribute { name, value })
    }

    #[inline]
    fn resolve_mark(&mut self, id: usize, ir: &mut Drain<HIR>) -> ParseResult<Node> {
        Ok(Node::Expr(self.resolve_expr(id, ir)?))
    }

    fn resolve_expr(&mut self, id: ExprId, ir: &mut Drain<HIR>) -> ParseResult<Expression> {
        let ir = ir.next().expect("Some HIR");

        match ir {
            HIR::Expr(e) => {
                let var = resolve_expr(&e, self);
                self.tree_map.insert(id, var.into_iter().collect());
                Ok(Expression::Unsafe(id, e))
            }
            HIR::Safe(e) => {
                let var = resolve_expr(&e, self);
                self.tree_map.insert(id, var.into_iter().collect());
                Ok(Expression::Safe(id, e))
            }
            HIR::Local(e) => {
                let var_id = resolve_local(&e, id, self);
                Ok(Expression::Local(id, var_id, e))
            }
            HIR::Each(e) => {
                let var = resolve_each(&e, id, self);
                let HEach { args, body, expr } = *e;
                Ok(Expression::Each(
                    id,
                    Box::new(Each {
                        var,
                        args,
                        body: self.step(body)?,
                        expr,
                    }),
                ))
            }
            HIR::IfElse(e) => {
                let HIfElse { ifs, if_else, els } = *e;
                let (expr, body) = ifs;
                let vars = resolve_if_block(&expr, id, self);
                let ifs = IfBlock {
                    vars,
                    expr,
                    block: self.step(body)?,
                };

                let mut buff = vec![];
                for (expr, body) in if_else {
                    let vars = resolve_if_block(&expr, id, self);
                    buff.push(IfBlock {
                        vars,
                        expr,
                        block: self.step(body)?,
                    });
                }

                let els = if let Some(body) = els {
                    Some(self.step(body)?)
                } else {
                    None
                };

                Ok(Expression::IfElse(
                    id,
                    Box::new(IfElse {
                        ifs,
                        if_else: buff,
                        els,
                    }),
                ))
            }
            HIR::Lit(_) => unreachable!(),
        }
    }

    #[inline]
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
            match child {
                ParseElement::Text(s) => {
                    let mut chunks = s.split(MARK).peekable();

                    if let Some(first) = chunks.peek() {
                        if first.is_empty() {
                            chunks.next();
                        }
                    }
                    for chunk in chunks {
                        if chunk.is_empty() {
                            panic!("chunk empty")
                        } else if HASH_LEN <= chunk.len() {
                            if let Some(id) = parse_id(&chunk[..HASH_LEN]) {
                                buff.push(self.resolve_mark(id as usize, ir)?);
                                let cut = &chunk[HASH_LEN..];
                                if !cut.is_empty() {
                                    buff.push(self.resolve_text(cut));
                                }
                            } else {
                                buff.push(self.resolve_text(chunk));
                            }
                        } else {
                            buff.push(self.resolve_text(chunk));
                        }
                    }
                }
                ParseElement::Node {
                    name,
                    attrs,
                    children,
                    ..
                } => buff.push(self.resolve_node(name, attrs, children, sink, ir)?),
                ParseElement::Document(_) => unreachable!(),
            }
        }

        Ok(buff)
    }
}
