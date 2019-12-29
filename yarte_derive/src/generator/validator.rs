use quote::quote;

use yarte_helpers::helpers::ErrorMessage;
use yarte_parser::SExpr;

pub(super) fn expression(e: &SExpr, out: &mut Vec<ErrorMessage>) {
    use syn::Expr::*;
    match **e.t() {
        Binary(..) | Call(..) | MethodCall(..) | Index(..) | Field(..) | Path(..) | Paren(..)
        | Macro(..) | Lit(..) | Try(..) | Unary(..) | Unsafe(..) | If(..) | Loop(..)
        | Match(..) => (),
        _ => out.push(ErrorMessage {
            message: "Not available Rust expression in a template expression".to_string(),
            span: *e.span(),
        }),
    }
}

pub(super) fn ifs(e: &SExpr, out: &mut Vec<ErrorMessage>) {
    use syn::Expr::*;
    match **e.t() {
        Binary(..) | Call(..) | MethodCall(..) | Index(..) | Field(..) | Path(..) | Paren(..)
        | Macro(..) | Lit(..) | Try(..) | Unary(..) | Unsafe(..) | If(..) | Loop(..)
        | Match(..) | Let(..) => (),
        _ => out.push(ErrorMessage {
            message: "Not available Rust expression in a template `if helper` arguments"
                .to_string(),
            span: *e.span(),
        }),
    }
}

pub(super) fn each(e: &SExpr, out: &mut Vec<ErrorMessage>) {
    use syn::Expr::*;
    match **e.t() {
        Call(..) | MethodCall(..) | Index(..) | Field(..) | Path(..) | Paren(..) | Macro(..)
        | Try(..) | Unsafe(..) | If(..) | Loop(..) | Match(..) | Range(..) | Reference(..) => (),
        _ => out.push(ErrorMessage {
            message: "Not available Rust expression in a template `each helper` argument"
                .to_string(),
            span: *e.span(),
        }),
    }
}

pub(super) fn unless(e: &SExpr, out: &mut Vec<ErrorMessage>) {
    use syn::Expr::*;
    match **e.t() {
        Binary(..) | Call(..) | MethodCall(..) | Index(..) | Field(..) | Path(..) | Paren(..)
        | Macro(..) | Lit(..) | Try(..) | Match(..) => (),
        Unary(syn::ExprUnary { op, .. }) => {
            if let syn::UnOp::Not(..) = op {
                out.push(ErrorMessage {
                    message: "Unary negate operator in `unless helper`, use `if helper` instead"
                        .to_string(),
                    span: *e.span(),
                })
            }
        }
        _ => out.push(ErrorMessage {
            message: "Not available Rust expression in a template `unless helper` expression"
                .to_string(),
            span: *e.span(),
        }),
    }
}

pub(super) fn scope(e: &SExpr, out: &mut Vec<ErrorMessage>) {
    use syn::Expr::*;
    match **e.t() {
        Path(..) | Field(..) | Index(..) => (),
        _ => out.push(ErrorMessage {
            message: "Not available Rust expression in scope argument".to_string(),
            span: *e.span(),
        }),
    }
}

// TODO:
pub(super) fn partial_assign(e: &syn::Expr) {
    use syn::Expr::*;
    match e {
        Path(..) | Field(..) | Index(..) | Lit(..) | Reference(..) | Array(..) | Range(..) => (),
        _ => panic!(
            "Not available Rust expression in partial assign argument:\n{}",
            quote!(#e)
        ),
    }
}
