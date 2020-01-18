#![allow(warnings)]

use syn::visit::Visit;

use crate::dom::{DOMBuilder, VarId};
use yarte_hir::Each;

pub fn resolve_each<'a>(expr: &'a Each, id: usize, builder: &'a mut DOMBuilder) -> VarId {
    ResolveEach::new(builder, id).resolve(expr)
}

struct ResolveEach<'a> {
    builder: &'a mut DOMBuilder,
    id: usize,
}

impl<'a> ResolveEach<'a> {
    fn new(builder: &mut DOMBuilder, id: usize) -> ResolveEach {
        ResolveEach { builder, id }
    }

    fn resolve(mut self, expr: &'a Each) -> VarId {
        0
    }
}

impl<'a> Visit<'a> for ResolveEach<'a> {}
