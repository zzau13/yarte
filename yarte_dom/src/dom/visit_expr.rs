use quote::quote;
use syn::{visit::Visit, Expr, ExprCall, ExprField, ExprPath};

use crate::dom::DOMBuilder;
use syn::punctuated::Punctuated;

pub fn resolve_expr<'a>(expr: &'a Expr, id: usize, builder: &'a mut DOMBuilder) {
    ResolveExpr::new(builder, id).resolve(expr)
}

struct ResolveExpr<'a> {
    builder: &'a mut DOMBuilder,
    id: usize,
}

impl<'a> ResolveExpr<'a> {
    fn new(builder: &mut DOMBuilder, id: usize) -> ResolveExpr {
        ResolveExpr { builder, id }
    }

    fn resolve(mut self, expr: &'a Expr) {
        self.visit_expr(expr);
    }
}

impl<'a> Visit<'a> for ResolveExpr<'a> {
    fn visit_expr_call(&mut self, ExprCall { args, .. }: &'a ExprCall) {
        for el in Punctuated::pairs(args) {
            self.visit_expr(el.value());
        }
    }

    fn visit_expr_field(
        &mut self,
        ExprField {
            base,
            dot_token,
            member,
            ..
        }: &'a ExprField,
    ) {
        let _name = quote!(#base#dot_token#member).to_string();
    }

    fn visit_expr_path(&mut self, ExprPath { path, .. }: &'a ExprPath) {
        if path.segments.len() == 1 {
            let _name = quote!(#path).to_string();
        }
    }
}
