use syn::visit::Visit;

use crate::dom::{DOMBuilder, Node};
use yarte_hir::Each;

pub fn resolve_each<'a>(expr: &'a Each, id: usize, builder: &'a mut DOMBuilder) {
    ResolveEach::new(builder, id).resolve(expr)
}

struct ResolveEach<'a> {
    builder: &'a mut DOMBuilder,
    id: usize,
}

impl<'a> ResolveEach<'a> {
    fn new<'n>(builder: &'n mut DOMBuilder, id: usize) -> ResolveEach<'n> {
        ResolveEach { builder, id }
    }

    fn resolve(mut self, expr: &'a Each) {
        todo!()
    }
}

impl<'a> Visit<'a> for ResolveEach<'a> {}
