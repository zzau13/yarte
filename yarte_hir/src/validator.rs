use quote::quote;

use crate::error::GError;
use yarte_helpers::helpers::ErrorMessage;
use yarte_parser::SExpr;

pub(super) fn expression(e: &SExpr, out: &mut Vec<ErrorMessage<GError>>) {
    use syn::Expr::*;
    match **e.t() {
        Binary(..) | Call(..) | MethodCall(..) | Index(..) | Field(..) | Path(..) | Paren(..)
        | Macro(..) | Lit(..) | Try(..) | Unary(..) | Unsafe(..) | If(..) | Loop(..)
        | Match(..) => (),
        _ => out.push(ErrorMessage {
            message: GError::ValidatorExpression,
            span: *e.span(),
        }),
    }
}

pub(super) fn ifs(e: &SExpr, out: &mut Vec<ErrorMessage<GError>>) {
    use syn::Expr::*;
    match **e.t() {
        Binary(..) | Call(..) | MethodCall(..) | Index(..) | Field(..) | Path(..) | Paren(..)
        | Macro(..) | Lit(..) | Try(..) | Unary(..) | Unsafe(..) | If(..) | Loop(..)
        | Match(..) | Let(..) => (),
        _ => out.push(ErrorMessage {
            message: GError::ValidatorIfs,
            span: *e.span(),
        }),
    }
}

pub(super) fn each(e: &SExpr, out: &mut Vec<ErrorMessage<GError>>) {
    use syn::Expr::*;
    match **e.t() {
        Call(..) | MethodCall(..) | Index(..) | Field(..) | Path(..) | Paren(..) | Macro(..)
        | Try(..) | Unsafe(..) | If(..) | Loop(..) | Match(..) | Range(..) | Reference(..) => (),
        _ => out.push(ErrorMessage {
            message: GError::ValidatorEach,
            span: *e.span(),
        }),
    }
}

pub(super) fn unless(e: &SExpr, out: &mut Vec<ErrorMessage<GError>>) {
    use syn::Expr::*;
    match **e.t() {
        Binary(..) | Call(..) | MethodCall(..) | Index(..) | Field(..) | Path(..) | Paren(..)
        | Macro(..) | Lit(..) | Try(..) | Match(..) => (),
        Unary(syn::ExprUnary { op, .. }) => {
            if let syn::UnOp::Not(..) = op {
                out.push(ErrorMessage {
                    message: GError::ValidatorUnlessNegate,
                    span: *e.span(),
                })
            }
        }
        _ => out.push(ErrorMessage {
            message: GError::ValidatorUnless,
            span: *e.span(),
        }),
    }
}

pub(super) fn scope(e: &SExpr, out: &mut Vec<ErrorMessage<GError>>) {
    use syn::Expr::*;
    match **e.t() {
        Path(..) | Field(..) | Index(..) => (),
        _ => out.push(ErrorMessage {
            message: GError::ValidatorPartialScope,
            span: *e.span(),
        }),
    }
}

// TODO: #39 remove panic
pub(super) fn partial_assign(e: &syn::Expr) {
    use syn::Expr::*;
    match e {
        Path(..) | Field(..) | Index(..) | Lit(..) | Reference(..) | Array(..) | Range(..)
        | Binary(..) | Call(..) | MethodCall(..) | Paren(..) | Macro(..) | Try(..) | Unary(..)
        | Unsafe(..) => (),
        _ => panic!(
            "Not available Rust expression in partial assign argument:\n{}",
            quote!(#e)
        ),
    }
}
