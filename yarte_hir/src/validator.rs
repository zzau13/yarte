use syn::spanned::Spanned;

use yarte_parser::{source_map::Span, ErrorMessage, SExpr};

use crate::error::{GError, MiddleError};

pub(super) fn expression(e: &SExpr, out: &mut Vec<ErrorMessage<GError>>) {
    use syn::Expr::*;
    match **e.t() {
        Binary(..) | Call(..) | MethodCall(..) | Index(..) | Field(..) | Path(..) | Paren(..)
        | Macro(..) | Lit(..) | Try(..) | Unary(..) | Unsafe(..) | If(..) | Loop(..)
        | Match(..) | Block(..) => (),
        _ => out.push(ErrorMessage {
            message: GError::ValidatorExpression,
            span: e.span(),
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
            span: e.span(),
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
            span: e.span(),
        }),
    }
}

pub(super) fn unless(e: &SExpr, out: &mut Vec<ErrorMessage<GError>>) {
    use syn::Expr::*;
    match **e.t() {
        Binary(..) | Call(..) | MethodCall(..) | Index(..) | Field(..) | Path(..) | Paren(..)
        | Macro(..) | Lit(..) | Try(..) | Match(..) => (),
        Unary(syn::ExprUnary { op, .. }) => {
            if let syn::UnOp::Not(t) = op {
                out.push(MiddleError::new(GError::ValidatorUnlessNegate, t.span(), e.span()).into())
            }
        }
        _ => out.push(ErrorMessage {
            message: GError::ValidatorUnless,
            span: e.span(),
        }),
    }
}

pub(super) fn scope(e: &SExpr, out: &mut Vec<ErrorMessage<GError>>) {
    use syn::Expr::*;
    match **e.t() {
        Path(..) | Field(..) | Index(..) => (),
        _ => out.push(ErrorMessage {
            message: GError::ValidatorPartialScope,
            span: e.span(),
        }),
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
pub(super) fn partial_assign(e: &syn::Expr, span: Span, out: &mut Vec<ErrorMessage<GError>>) {
    use syn::Expr::*;
    match e {
        Path(..) | Field(..) | Index(..) | Lit(..) | Reference(..) | Array(..) | Range(..)
        | Binary(..) | Call(..) | MethodCall(..) | Paren(..) | Macro(..) | Try(..) | Unary(..)
        | Unsafe(..) => (),
        _ => out.push(ErrorMessage {
            message: GError::ValidatorPartialAssign,
            span,
        }),
    }
}
