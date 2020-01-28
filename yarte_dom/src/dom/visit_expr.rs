use std::{collections::HashSet, hash::Hash};

use quote::quote;
use syn::{punctuated::Punctuated, visit::Visit, Expr, ExprCall, ExprField, ExprPath};

use yarte_helpers::helpers::calculate_hash;

use crate::dom::{DOMBuilder, ExprId, Var, VarId};

pub fn resolve_expr<'a>(expr: &'a Expr, id: usize, builder: &'a mut DOMBuilder) {
    ResolveExpr::new(builder).resolve(expr, id)
}

struct ResolveExpr<'a> {
    builder: &'a mut DOMBuilder,
    buff: HashSet<VarId>,
}

impl<'a> ResolveExpr<'a> {
    fn new(builder: &mut DOMBuilder) -> ResolveExpr {
        ResolveExpr {
            builder,
            buff: HashSet::new(),
        }
    }

    fn resolve(mut self, expr: &'a Expr, id: ExprId) {
        self.visit_expr(expr);
        self.builder.tree_map.insert(id, self.buff);
    }

    fn add(&mut self, name: String) {
        let var_id = calculate_hash(&name);
        if !self.builder.var_map.contains_key(&var_id) {
            self.builder.var_map.insert(var_id, Var::This(name));
        }
        self.buff.insert(var_id);
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
        let name = quote!(#base#dot_token#member).to_string();
        self.add(name);
    }

    fn visit_expr_path(&mut self, ExprPath { path, .. }: &'a ExprPath) {
        if path.segments.len() == 1 {
            let name = quote!(#path).to_string();
            if !name.chars().next().unwrap().is_uppercase() {
                self.add(name);
            }
        }
    }
}
