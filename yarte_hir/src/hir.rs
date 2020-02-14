/// High level intermediate representation after lowering Ast
#[derive(Debug, Clone, PartialEq)]
pub enum HIR {
    Lit(String),
    Expr(Box<syn::Expr>),
    Safe(Box<syn::Expr>),
    Each(Box<Each>),
    IfElse(Box<IfElse>),
    Local(Box<syn::Local>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfElse {
    pub ifs: (syn::Expr, Vec<HIR>),
    pub if_else: Vec<(syn::Expr, Vec<HIR>)>,
    pub els: Option<Vec<HIR>>,
}

/// for expr in args { body }
#[derive(Debug, Clone, PartialEq)]
pub struct Each {
    pub args: syn::Expr,
    pub body: Vec<HIR>,
    pub expr: syn::Expr,
}
