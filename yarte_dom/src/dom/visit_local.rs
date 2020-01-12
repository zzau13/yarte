use syn::{visit::Visit, Local};

use crate::dom::DOMBuilder;

pub fn resolve_local<'a>(expr: &'a Local, id: usize, builder: &'a mut DOMBuilder) {
    ResolveLocal::new(builder, id).resolve(expr)
}

struct ResolveLocal<'a> {
    builder: &'a mut DOMBuilder,
    id: usize,
}

impl<'a> ResolveLocal<'a> {
    fn new<'n>(builder: &'n mut DOMBuilder, id: usize) -> ResolveLocal<'n> {
        ResolveLocal { builder, id }
    }

    fn resolve(mut self, expr: &'a Local) {
        todo!()
    }
}

impl<'a> Visit<'a> for ResolveLocal<'a> {}
