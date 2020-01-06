use quote::quote;
use syn::{
    punctuated::Punctuated,
    visit_mut::{self, VisitMut},
};

use super::Generator;

macro_rules! visit_attrs {
    ($_self:ident, $attrs:ident) => {
        for it in $attrs {
            $_self.visit_attribute_mut(it)
        }
    };
}

macro_rules! visit_punctuated {
    ($_self:ident, $ele:expr, $method:ident) => {
        for mut el in Punctuated::pairs_mut($ele) {
            $_self.$method(el.value_mut());
        }
    };
}

impl<'a> VisitMut for Generator<'a> {
    fn visit_arm_mut(
        &mut self,
        syn::Arm {
            attrs,
            pat,
            guard,
            body,
            ..
        }: &mut syn::Arm,
    ) {
        visit_attrs!(self, attrs);

        self.scp.push_scope(vec![]);
        self.visit_pat_mut(pat);
        if let Some((_, expr)) = guard {
            self.visit_expr_mut(expr);
        }
        self.visit_expr_mut(body);
        self.scp.pop();
    }

    fn visit_attribute_mut(&mut self, _i: &mut syn::Attribute) {
        panic!("Not available attributes in a template expression");
    }

    fn visit_block_mut(&mut self, i: &mut syn::Block) {
        self.scp.push_scope(vec![]);
        visit_mut::visit_block_mut(self, i);
        self.scp.pop();
    }

    fn visit_expr_mut(&mut self, expr: &mut syn::Expr) {
        use syn::Expr::*;
        match expr {
            Path(i) => {
                debug_assert!(!self.scp.is_empty() && !self.scp[0].is_empty());
                *expr = self
                    .resolve_path(&i)
                    .expect("Correct resolve path expression");
            }
            a => visit_mut::visit_expr_mut(self, a),
        };
    }

    fn visit_expr_assign_mut(
        &mut self,
        syn::ExprAssign {
            attrs, left, right, ..
        }: &mut syn::ExprAssign,
    ) {
        visit_attrs!(self, attrs);
        if let Some(ident) = self.scp.get_by(&quote!(#left).to_string()) {
            *left = Box::new(ident.clone());
            self.visit_expr_mut(right);
        } else {
            panic!("Not exist in current scope");
        };
    }

    fn visit_expr_assign_op_mut(
        &mut self,
        syn::ExprAssignOp {
            attrs, left, right, ..
        }: &mut syn::ExprAssignOp,
    ) {
        visit_attrs!(self, attrs);
        if let Some(ident) = self.scp.get_by(&quote!(#left).to_string()) {
            *left = Box::new(ident.clone());
            self.visit_expr_mut(right);
        } else {
            panic!("Not exist in current scope");
        };
    }

    fn visit_expr_async_mut(&mut self, _i: &mut syn::ExprAsync) {
        panic!("Not available async in a template expression");
    }

    fn visit_expr_call_mut(
        &mut self,
        syn::ExprCall {
            attrs, func, args, ..
        }: &mut syn::ExprCall,
    ) {
        visit_attrs!(self, attrs);
        if let Some(ident) = self.scp.get_by(&quote!(#func).to_string()) {
            *func = Box::new(ident.clone());
        }
        visit_punctuated!(self, args, visit_expr_mut);
    }

    fn visit_expr_closure_mut(
        &mut self,
        syn::ExprClosure {
            attrs,
            asyncness,
            inputs,
            body,
            ..
        }: &mut syn::ExprClosure,
    ) {
        visit_attrs!(self, attrs);
        if let Some(..) = asyncness {
            panic!("Not available async in template expression");
        };

        self.scp.push_scope(vec![]);
        visit_punctuated!(self, inputs, visit_pat_mut);
        self.visit_expr_mut(body);
        self.scp.pop();
    }

    fn visit_expr_for_loop_mut(
        &mut self,
        syn::ExprForLoop {
            attrs,
            pat,
            expr,
            body,
            ..
        }: &mut syn::ExprForLoop,
    ) {
        visit_attrs!(self, attrs);
        self.scp.push_scope(vec![]);
        self.visit_pat_mut(pat);
        let last = self.scp.pops();
        self.visit_expr_mut(expr);
        self.scp.push_scope(last);
        self.visit_block_mut(body);
        self.scp.pop();
    }

    fn visit_expr_if_mut(
        &mut self,
        syn::ExprIf {
            attrs,
            cond,
            then_branch,
            else_branch,
            ..
        }: &mut syn::ExprIf,
    ) {
        visit_attrs!(self, attrs);

        self.scp.push_scope(vec![]);

        self.visit_expr_mut(cond);

        self.visit_block_mut(then_branch);
        self.scp.pop();

        if let Some((_, it)) = else_branch {
            self.visit_expr_mut(it);
        };
    }

    fn visit_expr_let_mut(
        &mut self,
        syn::ExprLet {
            attrs, expr, pat, ..
        }: &mut syn::ExprLet,
    ) {
        visit_attrs!(self, attrs);

        self.scp.push_scope(vec![]);
        self.visit_pat_mut(pat);
        let last = self.scp.pops();

        self.visit_expr_mut(expr);
        self.scp.extend(last);
    }

    fn visit_expr_try_block_mut(&mut self, _i: &mut syn::ExprTryBlock) {
        panic!("Not allowed try block expression in a template expression");
    }

    fn visit_expr_yield_mut(&mut self, _i: &mut syn::ExprYield) {
        panic!("Not allowed yield expression in a template expression");
    }

    fn visit_local_mut(
        &mut self,
        syn::Local {
            attrs, pat, init, ..
        }: &mut syn::Local,
    ) {
        visit_attrs!(self, attrs);
        self.scp.push_scope(vec![]);
        self.visit_pat_mut(pat);
        let scope = self.scp.pops();
        if let Some((_, expr)) = init {
            self.visit_expr_mut(expr);
        }
        self.scp.extend(scope);
    }

    fn visit_pat_ident_mut(
        &mut self,
        syn::PatIdent {
            attrs,
            ident,
            subpat,
            ..
        }: &mut syn::PatIdent,
    ) {
        visit_attrs!(self, attrs);

        if subpat.is_some() {
            panic!("Subpat is not allowed");
        }

        *ident = self.scp.push_ident(&ident.to_string());
    }

    fn visit_pat_lit_mut(&mut self, _i: &mut syn::PatLit) {
        panic!("Not allowed pat lit");
    }

    fn visit_pat_macro_mut(&mut self, _i: &mut syn::PatMacro) {
        panic!("Not allowed pat macro");
    }

    fn visit_pat_path_mut(&mut self, _i: &mut syn::PatPath) {}

    fn visit_pat_range_mut(&mut self, _i: &mut syn::PatRange) {
        panic!("Not allowed pat range");
    }

    fn visit_pat_struct_mut(&mut self, _i: &mut syn::PatStruct) {
        panic!("Not available let struct decompose, use `with` helper instead");
    }

    fn visit_stmt_mut(&mut self, i: &mut syn::Stmt) {
        use syn::Stmt::*;
        match i {
            Local(i) => {
                self.visit_local_mut(i);
            }
            Item(_i) => {
                unimplemented!();
            }
            Expr(i) => {
                self.visit_expr_mut(i);
            }
            Semi(i, _) => {
                self.visit_expr_mut(i);
            }
        }
    }
}
