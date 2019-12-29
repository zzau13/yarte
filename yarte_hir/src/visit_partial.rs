use std::{collections::BTreeMap, mem};

use quote::quote;
use syn::visit::Visit;

use super::{is_tuple_index, validator};

pub fn visit_partial(e: &[syn::Expr]) -> (BTreeMap<String, &syn::Expr>, Option<&syn::Expr>) {
    PartialBuilder::default().build(e)
}

struct PartialBuilder<'a> {
    ident: String,
    ctx: BTreeMap<String, &'a syn::Expr>,
    scope: Option<&'a syn::Expr>,
}

impl<'a> Default for PartialBuilder<'a> {
    fn default() -> Self {
        Self {
            ident: String::new(),
            ctx: BTreeMap::new(),
            scope: None,
        }
    }
}

macro_rules! panic_attrs {
    ($attrs:expr) => {
        if !$attrs.is_empty() {
            panic!("Not available attributes in a template expression");
        }
    };
}

macro_rules! panic_some {
    ($some:expr) => {
        if $some.is_some() {
            panic!("Not available in a template expression");
        }
    };
}

impl<'a> PartialBuilder<'a> {
    fn build(
        mut self,
        e: &'a [syn::Expr],
    ) -> (BTreeMap<String, &'a syn::Expr>, Option<&'a syn::Expr>) {
        debug_assert_ne!(e.len(), 0);
        use syn::Expr::*;
        match &e[0] {
            Assign(assign) => self.visit_expr_assign(&assign),
            e @ Path(..) => self.scope = Some(e),
            _ => panic!("Not available in partial argument:\n{}", quote!(#(#e ,)*)),
        }

        for i in (&e[1..]).iter() {
            match i {
                Assign(assign) => self.visit_expr_assign(&assign),
                Path(..) => panic!("place scope argument `{}` at first position", quote!(#i)),
                _ => panic!("Not available in partial argument:\n{}", quote!(#i)),
            }
        }

        (self.ctx, self.scope)
    }
}

impl<'a> Visit<'a> for PartialBuilder<'a> {
    fn visit_expr(&mut self, i: &'a syn::Expr) {
        use syn::Expr::*;
        match *i {
            Path(ref e) => {
                panic_attrs!(e.attrs);
                panic_some!(e.qself);
                panic_some!(e.path.leading_colon);
                if !self.ident.is_empty() {
                    panic!("Empty buffer before");
                }
                if e.path.segments.len() != 1 {
                    panic!(
                        "Not available Rust expression in partial scope argument:\n{}",
                        quote!(#i)
                    )
                }
                let ident = e.path.segments[0].ident.to_string();
                if RESERVED_WORDS.contains(&ident.as_str()) || is_tuple_index(ident.as_bytes()) {
                    panic!(
                        "Reserved word `{}` in partial assign argument:\n{}",
                        ident,
                        quote!(#i)
                    );
                }

                self.ident = ident;
            }
            _ => panic!(
                "Not available Rust expression in partial argument:\n{}",
                quote!(#i)
            ),
        }
    }

    fn visit_expr_assign(&mut self, i: &'a syn::ExprAssign) {
        validator::partial_assign(&i.right);

        panic_attrs!(i.attrs);
        self.visit_expr(&i.left);
        panic_some!(self
            .ctx
            .insert(mem::replace(&mut self.ident, String::new()), &i.right));
    }
}

static RESERVED_WORDS: &[&str; 2] = &["self", "super"];
