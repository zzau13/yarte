#![allow(warnings)]

use syn::visit::Visit;

use crate::dom::{DOMBuilder, VarId};
use syn::Expr;
use yarte_hir::IfElse;

pub fn resolve_if_block<'a>(expr: &'a Expr, id: usize, builder: &'a mut DOMBuilder) -> Vec<VarId> {
    ResolveIfElse::new(builder, id).resolve(expr)
}

struct ResolveIfElse<'a> {
    builder: &'a mut DOMBuilder,
    id: usize,
}

impl<'a> ResolveIfElse<'a> {
    fn new<'n>(builder: &'n mut DOMBuilder, id: usize) -> ResolveIfElse<'n> {
        ResolveIfElse { builder, id }
    }

    fn resolve(mut self, expr: &'a Expr) -> Vec<VarId> {
        vec![]
    }
}

impl<'a> Visit<'a> for ResolveIfElse<'a> {}
