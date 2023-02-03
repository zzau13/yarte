use std::{collections::BTreeMap, mem};

use syn::visit::Visit;

use yarte_parser::{ErrorMessage, SVExpr};

use crate::{error::GError, is_tuple_index, validator};

pub fn visit_partial<'a, 'b>(
    e: &'a SVExpr,
    // TODO: #39
    err: &'b mut Vec<ErrorMessage<GError>>,
) -> (BTreeMap<String, &'a syn::Expr>, Option<&'a syn::Expr>) {
    PartialBuilder::new(e, err).build()
}

// TODO: Adjust span at error
struct PartialBuilder<'a, 'b> {
    e: &'a SVExpr,
    ident: String,
    ctx: BTreeMap<String, &'a syn::Expr>,
    scope: Option<&'a syn::Expr>,
    err: &'b mut Vec<ErrorMessage<GError>>,
}

macro_rules! panic_some {
    ($_self:ident, $some:expr) => {
        if $some.is_some() {
            $_self.err.push(ErrorMessage {
                message: GError::NotAvailable,
                span: $_self.e.span(),
            });
        }
    };
}

impl<'a, 'b> PartialBuilder<'a, 'b> {
    fn new<'n, 'm>(
        e: &'n SVExpr,
        err: &'m mut Vec<ErrorMessage<GError>>,
    ) -> PartialBuilder<'n, 'm> {
        PartialBuilder {
            ident: Default::default(),
            ctx: Default::default(),
            scope: Default::default(),
            err,
            e,
        }
    }

    fn build(mut self) -> (BTreeMap<String, &'a syn::Expr>, Option<&'a syn::Expr>) {
        let e = self.e.t();
        debug_assert_ne!(e.len(), 0);
        use syn::Expr::*;
        match &e[0].as_ref() {
            Assign(assign) => self.visit_expr_assign(assign),
            e @ Path(..) => self.scope = Some(e),
            _ => self.err.push(ErrorMessage {
                message: GError::PartialArguments,
                span: self.e.span(),
            }),
        }

        for i in e[1..].iter() {
            match i.as_ref() {
                Assign(assign) => self.visit_expr_assign(assign),
                Path(..) => self.err.push(ErrorMessage {
                    message: GError::PartialArgumentsScopeFirst,
                    span: self.e.span(),
                }),
                _ => self.err.push(ErrorMessage {
                    message: GError::PartialArguments,
                    span: self.e.span(),
                }),
            }
        }

        (self.ctx, self.scope)
    }
}

impl<'a, 'b> Visit<'a> for PartialBuilder<'a, 'b> {
    fn visit_expr(&mut self, i: &'a syn::Expr) {
        use syn::Expr::*;
        match *i {
            Path(ref e) => {
                assert!(self.ident.is_empty());
                panic_some!(self, e.qself);
                panic_some!(self, e.path.leading_colon);

                if e.path.segments.len() != 1 {
                    self.err.push(ErrorMessage {
                        message: GError::PartialArgumentsScope,
                        span: self.e.span(),
                    })
                }
                let ident = e.path.segments[0].ident.to_string();
                if RESERVED_WORDS.contains(&ident.as_str()) || is_tuple_index(ident.as_bytes()) {
                    self.err.push(ErrorMessage {
                        message: GError::ReservedWord,
                        span: self.e.span(),
                    })
                }

                self.ident = ident;
            }
            _ => self.err.push(ErrorMessage {
                message: GError::PartialArguments,
                span: self.e.span(),
            }),
        }
    }

    fn visit_expr_assign(&mut self, i: &'a syn::ExprAssign) {
        validator::partial_assign(&i.right, self.e.span(), self.err);

        self.visit_expr(&i.left);
        panic_some!(self, self.ctx.insert(mem::take(&mut self.ident), &i.right));
    }
}

static RESERVED_WORDS: &[&str; 2] = &["self", "super"];
