use quote::quote;

pub(super) fn expression(e: &syn::Expr) {
    use syn::Expr::*;
    match e {
        Binary(..) | Call(..) | MethodCall(..) | Index(..) | Field(..) | Path(..) | Paren(..)
        | Macro(..) | Lit(..) | Try(..) | Unary(..) | Unsafe(..) | If(..) | Loop(..)
        | Match(..) => (),
        _ => panic!(
            "Not available Rust expression in a template expression:\n{}",
            quote!(#e)
        ),
    }
}

pub(super) fn statement(e: &syn::Stmt) {
    use syn::Stmt::*;
    match e {
        Local(..) => (),
        _ => panic!(
            "Not available Rust statement in a template local:\n{}",
            quote!(#e)
        ),
    }
}

pub(super) fn ifs(e: &syn::Expr) {
    use syn::Expr::*;
    match e {
        Binary(..) | Call(..) | MethodCall(..) | Index(..) | Field(..) | Path(..) | Paren(..)
        | Macro(..) | Lit(..) | Try(..) | Unary(..) | Unsafe(..) | If(..) | Loop(..)
        | Match(..) | Let(..) => (),
        _ => panic!(
            "Not available Rust expression in a template `if helper` arguments:\n{}",
            quote!(#e)
        ),
    }
}

pub(super) fn each(e: &syn::Expr) {
    use syn::Expr::*;
    match e {
        Call(..) | MethodCall(..) | Index(..) | Field(..) | Path(..) | Paren(..) | Macro(..)
        | Try(..) | Unsafe(..) | If(..) | Loop(..) | Match(..) | Range(..) => (),
        _ => panic!(
            "Not available Rust expression in a template `each helper` argument:\n{}",
            quote!(#e)
        ),
    }
}

pub(super) fn unless(e: &syn::Expr) {
    use syn::Expr::*;
    match e {
        Binary(..) | Call(..) | MethodCall(..) | Index(..) | Field(..) | Path(..) | Paren(..)
        | Macro(..) | Lit(..) | Try(..) | Match(..) => (),
        Unary(syn::ExprUnary { op, .. }) => {
            if let syn::UnOp::Not(..) = op {
                panic!(
                    "Unary negate operator in `unless helper`, use `if helper` instead:\n{}",
                    quote!(#e)
                )
            }
        }
        _ => panic!(
            "Not available Rust expression in a template `unless helper` expression:\n{}",
            quote!(#e)
        ),
    }
}

pub(super) fn scope(e: &syn::Expr) {
    use syn::Expr::*;
    match e {
        Path(..) | Field(..) | Index(..) => (),
        _ => panic!(
            "Not available Rust expression in scope argument:\n{}",
            quote!(#e)
        ),
    }
}

pub(super) fn partial_assign(e: &syn::Expr) {
    use syn::Expr::*;
    match e {
        Path(..) | Field(..) | Index(..) | Lit(..) => (),
        _ => panic!(
            "Not available Rust expression in partial assign argument:\n{}",
            quote!(#e)
        ),
    }
}
