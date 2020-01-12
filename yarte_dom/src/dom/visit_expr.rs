use syn::{visit::Visit, Expr};

use crate::dom::DOMBuilder;

pub fn resolve_expr<'a>(expr: &'a Expr, id: usize, builder: &'a mut DOMBuilder) {
    ResolveExpr::new(builder, id).resolve(expr)
}

struct ResolveExpr<'a> {
    builder: &'a mut DOMBuilder,
    id: usize,
}

impl<'a> ResolveExpr<'a> {
    fn new<'n>(builder: &'n mut DOMBuilder, id: usize) -> ResolveExpr<'n> {
        ResolveExpr { builder, id }
    }

    fn resolve(mut self, expr: &'a Expr) {
        self.visit_expr(expr);
    }
}

impl<'a> Visit<'a> for ResolveExpr<'a> {}
