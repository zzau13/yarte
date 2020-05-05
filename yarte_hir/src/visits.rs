use quote::quote;
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    visit_mut::{self, VisitMut},
};

use super::Generator;
use crate::error::GError;

impl<'a> VisitMut for Generator<'a> {
    fn visit_arm_mut(
        &mut self,
        syn::Arm {
            pat, guard, body, ..
        }: &mut syn::Arm,
    ) {
        self.scp.push_scope(vec![]);
        self.visit_pat_mut(pat);
        if let Some((_, expr)) = guard {
            self.visit_expr_mut(expr);
        }
        self.visit_expr_mut(body);
        self.scp.pop();
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
                match self.resolve_path(&i) {
                    Ok(e) => {
                        *expr = e;
                    }
                    Err(e) => {
                        self.buf_err.push((e, expr.span()));
                    }
                }
            }
            a => visit_mut::visit_expr_mut(self, a),
        };
    }

    fn visit_expr_assign_mut(&mut self, syn::ExprAssign { left, right, .. }: &mut syn::ExprAssign) {
        if let Some(ident) = self.scp.get_by(&quote!(#left).to_string()) {
            *left = Box::new(ident.clone());
            self.visit_expr_mut(right);
        } else {
            self.buf_err.push((GError::NotExist, left.span()));
        };
    }

    fn visit_expr_assign_op_mut(
        &mut self,
        syn::ExprAssignOp { left, right, .. }: &mut syn::ExprAssignOp,
    ) {
        if let Some(ident) = self.scp.get_by(&quote!(#left).to_string()) {
            *left = Box::new(ident.clone());
            self.visit_expr_mut(right);
        } else {
            self.buf_err.push((GError::NotExist, left.span()));
        };
    }

    fn visit_expr_async_mut(&mut self, i: &mut syn::ExprAsync) {
        self.buf_err.push((GError::NotAvailable, i.span()));
    }

    fn visit_expr_call_mut(&mut self, syn::ExprCall { func, args, .. }: &mut syn::ExprCall) {
        if let Some(ident) = self.scp.get_by(&quote!(#func).to_string()) {
            *func = Box::new(ident.clone());
        }
        visit_punctuated!(self, args, visit_expr_mut);
    }

    fn visit_expr_closure_mut(
        &mut self,
        syn::ExprClosure {
            asyncness,
            inputs,
            body,
            ..
        }: &mut syn::ExprClosure,
    ) {
        if let Some(..) = asyncness {
            self.buf_err.push((GError::NotAvailable, asyncness.span()));
        };

        self.scp.push_scope(vec![]);
        visit_punctuated!(self, inputs, visit_pat_mut);
        self.visit_expr_mut(body);
        self.scp.pop();
    }

    fn visit_expr_for_loop_mut(
        &mut self,
        syn::ExprForLoop {
            pat, expr, body, ..
        }: &mut syn::ExprForLoop,
    ) {
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
            cond,
            then_branch,
            else_branch,
            ..
        }: &mut syn::ExprIf,
    ) {
        self.scp.push_scope(vec![]);

        self.visit_expr_mut(cond);

        self.visit_block_mut(then_branch);
        self.scp.pop();

        if let Some((_, it)) = else_branch {
            self.visit_expr_mut(it);
        };
    }

    fn visit_expr_let_mut(&mut self, syn::ExprLet { expr, pat, .. }: &mut syn::ExprLet) {
        self.scp.push_scope(vec![]);
        self.visit_pat_mut(pat);
        let last = self.scp.pops();

        self.visit_expr_mut(expr);
        self.scp.extend(last);
    }

    fn visit_expr_try_block_mut(&mut self, i: &mut syn::ExprTryBlock) {
        self.buf_err.push((GError::NotAvailable, i.span()));
    }

    fn visit_expr_yield_mut(&mut self, i: &mut syn::ExprYield) {
        self.buf_err.push((GError::NotAvailable, i.span()));
    }

    fn visit_local_mut(&mut self, syn::Local { pat, init, .. }: &mut syn::Local) {
        self.scp.push_scope(vec![]);
        self.visit_pat_mut(pat);
        let scope = self.scp.pops();
        if let Some((_, expr)) = init {
            self.visit_expr_mut(expr);
        }
        self.scp.extend(scope);
    }

    fn visit_pat_ident_mut(&mut self, syn::PatIdent { ident, subpat, .. }: &mut syn::PatIdent) {
        if let Some((at, pat)) = subpat {
            self.buf_err
                .push((GError::NotAvailable, at.span().join(pat.span()).unwrap()));
        }

        *ident = self.scp.push_ident(&ident.to_string());
    }

    fn visit_pat_lit_mut(&mut self, i: &mut syn::PatLit) {
        self.buf_err.push((GError::NotAvailable, i.span()));
    }

    fn visit_pat_macro_mut(&mut self, i: &mut syn::PatMacro) {
        self.buf_err.push((GError::NotAvailable, i.span()));
    }

    fn visit_pat_range_mut(&mut self, i: &mut syn::PatRange) {
        self.buf_err.push((GError::NotAvailable, i.span()));
    }

    fn visit_pat_struct_mut(&mut self, i: &mut syn::PatStruct) {
        self.buf_err.push((GError::NotAvailable, i.span()));
    }

    fn visit_stmt_mut(&mut self, i: &mut syn::Stmt) {
        use syn::Stmt::*;
        match i {
            Local(i) => {
                self.visit_local_mut(i);
            }
            Item(i) => {
                self.buf_err.push((GError::Unimplemented, i.span()));
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
