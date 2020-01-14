#![allow(warnings)]

use syn::visit::Visit;

use crate::dom::DOMBuilder;
use yarte_hir::IfElse;

pub fn resolve_if_else<'a>(expr: &'a IfElse, id: usize, builder: &'a mut DOMBuilder) {
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

    fn resolve(mut self, expr: &'a IfElse) {
        todo!()
    }
}

impl<'a> Visit<'a> for ResolveIfElse<'a> {}
