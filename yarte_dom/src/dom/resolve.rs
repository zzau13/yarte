use quote::quote;
use syn::{
    punctuated::Punctuated, visit::Visit, Expr, ExprCall, ExprField, ExprLet, ExprMethodCall,
    ExprPath, FieldPat, Ident, Local, Pat, PatIdent, PatType,
};

use yarte_helpers::calculate_hash;
use yarte_hir::Each;

use crate::dom::{DOMBuilder, ExprId, Var, VarId, VarInner};

pub fn resolve_expr<'a>(expr: &'a Expr, builder: &'a mut DOMBuilder) -> Vec<VarId> {
    ResolveExpr::new(builder).resolve(expr)
}

struct ResolveExpr<'a> {
    builder: &'a mut DOMBuilder,
    buff: Vec<VarId>,
}

impl<'a> ResolveExpr<'a> {
    fn new(builder: &mut DOMBuilder) -> ResolveExpr {
        ResolveExpr {
            builder,
            buff: Vec::new(),
        }
    }

    fn resolve(mut self, expr: &'a Expr) -> Vec<VarId> {
        self.visit_expr(expr);
        self.buff
    }

    fn add(&mut self, var_id: VarId, ident: String, base: VarId) {
        let mut vars = vec![var_id];
        if let Some(x) = self.builder.var_map.get(&var_id) {
            if let Var::Local(id, _) = x {
                vars.extend(self.builder.tree_map.get(id).unwrap());
            }
        } else {
            self.builder
                .var_map
                .insert(var_id, Var::This(VarInner { ident, base }));
        }
        self.buff.extend(vars);
    }
}

impl<'a> Visit<'a> for ResolveExpr<'a> {
    // TODO:
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
        let ident = quote!(#base#dot_token#member).to_string();
        let base = calculate_hash(&quote!(#base).to_string());
        let var_id = calculate_hash(&ident);
        self.add(var_id, ident, base);
    }

    fn visit_expr_path(&mut self, ExprPath { path, .. }: &'a ExprPath) {
        if path.segments.len() == 1 {
            let name = quote!(#path).to_string();
            if !name.chars().next().unwrap().is_uppercase() {
                let base = calculate_hash(&name);
                self.add(base, name, base);
            }
        }
    }
}

pub fn resolve_each<'a>(
    Each { args, expr, .. }: &'a Each,
    id: ExprId,
    builder: &'a mut DOMBuilder,
) -> (VarId, Option<VarId>) {
    let vars_expr = resolve_expr(args, builder);
    let vars = resolve_expr(expr, builder);
    let l = vars.len();
    let mut vars = vars.iter().copied();
    builder.tree_map.insert(id, vars_expr.into_iter().collect());
    if l == 1 {
        (vars.next().unwrap(), None)
    } else if l == 2 {
        let index = vars.next();
        (vars.next().unwrap(), index)
    } else {
        unreachable!()
    }
}

pub fn resolve_if_block<'a>(expr: &'a Expr, id: usize, builder: &'a mut DOMBuilder) -> Vec<VarId> {
    ResolveIf::new(builder, id).resolve(expr)
}

struct ResolveIf<'a> {
    builder: &'a mut DOMBuilder,
    id: ExprId,
    buff: Vec<VarId>,
}

impl<'a> ResolveIf<'a> {
    fn new(builder: &mut DOMBuilder, id: ExprId) -> ResolveIf {
        ResolveIf {
            builder,
            id,
            buff: vec![],
        }
    }

    fn resolve(mut self, expr: &'a Expr) -> Vec<VarId> {
        self.visit_expr(expr);
        self.buff
    }
}

impl<'a> Visit<'a> for ResolveIf<'a> {
    fn visit_expr_let(&mut self, ExprLet { pat, expr, .. }: &'a ExprLet) {
        self.visit_pat(pat);
        let vars = resolve_expr(expr, self.builder);
        self.builder
            .tree_map
            .insert(self.id, vars.into_iter().collect());
    }

    fn visit_field_pat(&mut self, FieldPat { pat, .. }: &'a FieldPat) {
        self.visit_pat(pat);
    }

    // TODO:
    fn visit_ident(&mut self, ident: &'a Ident) {
        let ident = quote!(#ident).to_string();
        let var_id = calculate_hash(&ident);
        self.builder.var_map.insert(
            var_id,
            Var::This(VarInner {
                base: var_id,
                ident,
            }),
        );
        self.buff.push(var_id);
    }
}

pub fn resolve_local<'a>(expr: &'a Local, id: usize, builder: &'a mut DOMBuilder) -> VarId {
    ResolveLocal::new(builder, id).resolve(expr)
}

struct ResolveLocal<'a> {
    builder: &'a mut DOMBuilder,
    id: ExprId,
    var_id: Option<VarId>,
}

impl<'a> ResolveLocal<'a> {
    fn new(builder: &mut DOMBuilder, id: ExprId) -> ResolveLocal {
        ResolveLocal {
            builder,
            id,
            var_id: None,
        }
    }

    fn resolve(mut self, l: &'a Local) -> VarId {
        self.visit_local(l);
        self.var_id.expect("Local need pat ident")
    }
}

impl<'a> Visit<'a> for ResolveLocal<'a> {
    fn visit_local(&mut self, Local { pat, init, .. }: &'a Local) {
        self.visit_pat(pat);
        let vars = resolve_expr(&init.as_ref().expect("unreachable").1, self.builder);
        self.builder
            .tree_map
            .insert(self.id, vars.into_iter().collect());
    }

    fn visit_pat(&mut self, pat: &'a Pat) {
        match pat {
            Pat::Ident(p) => self.visit_pat_ident(p),
            Pat::Type(p) => self.visit_pat_type(p),
            _ => unreachable!(),
        }
    }

    fn visit_pat_ident(&mut self, PatIdent { ident, .. }: &'a PatIdent) {
        let ident = quote!(#ident).to_string();
        let var_id = calculate_hash(&ident);
        self.builder.var_map.insert(
            var_id,
            Var::Local(
                self.id,
                VarInner {
                    base: var_id,
                    ident,
                },
            ),
        );
        assert!(self.var_id.is_none());
        self.var_id.replace(var_id);
    }
    fn visit_pat_type(&mut self, PatType { pat, .. }: &'a PatType) {
        self.visit_pat(pat);
    }
}
