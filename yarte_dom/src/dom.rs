#![allow(warnings)]

use std::collections::HashMap;

use yarte_hir::HIR;

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
