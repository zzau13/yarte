use std::fmt::{self, Write};

use quote::{format_ident, quote};
use syn::{
    punctuated::Punctuated, visit_mut, visit_mut::VisitMut, ExprField, ExprMethodCall, ExprParen,
    ExprPath, ExprReference, PathSegment, Token,
};

use yarte_parser::StmtLocal;

use crate::{Each, IfElse, HIR};

#[inline]
pub fn serialize<'a, W, I>(ir: I, writer: &mut W) -> fmt::Result
where
    W: Write,
    I: Iterator<Item = &'a HIR>,
{
    _serialize(ir, writer, &mut Serialize)
}

#[inline]
pub fn serialize_resolved<'a, W, I>(ir: I, writer: &mut W) -> fmt::Result
where
    W: Write,
    I: Iterator<Item = &'a HIR>,
{
    _serialize(ir, writer, &mut SerializeResolve)
}

fn _serialize<'a, W, I, V>(ir: I, writer: &mut W, visitor: &mut V) -> fmt::Result
where
    W: Write,
    I: Iterator<Item = &'a HIR>,
    V: VisitMut,
{
    for i in ir {
        match i {
            HIR::Local(a) => {
                let mut local = *a.clone();
                visitor.visit_local_mut(&mut local);
                let local: StmtLocal = local.into();
                writer.write_str("{{ ")?;
                writer.write_str(&quote!(#local).to_string())?;
                writer.write_str(" }}")?
            }
            HIR::Lit(a) => writer.write_str(a)?,
            HIR::Safe(a) => {
                let mut expr = *a.clone();
                visitor.visit_expr_mut(&mut expr);
                writer.write_str("{{{ ")?;
                writer.write_str(&quote!(#expr).to_string())?;
                writer.write_str(" }}}")?
            }
            HIR::Expr(a) => {
                let mut expr = *a.clone();
                visitor.visit_expr_mut(&mut expr);
                writer.write_str("{{ ")?;
                writer.write_str(&quote!(#expr).to_string())?;
                writer.write_str(" }}")?
            }
            HIR::IfElse(a) => {
                let IfElse { ifs, if_else, els } = &**a;
                let (expr, ir) = ifs;
                let mut expr = expr.clone();
                visitor.visit_expr_mut(&mut expr);
                writer.write_str("{{#if ")?;
                writer.write_str(&quote!(#expr).to_string())?;
                writer.write_str(" }}")?;
                serialize(ir.iter(), writer)?;
                for (expr, ir) in if_else {
                    let mut expr = expr.clone();
                    visitor.visit_expr_mut(&mut expr);
                    writer.write_str("{{else if ")?;
                    writer.write_str(&quote!(#expr).to_string())?;
                    writer.write_str(" }}")?;
                    serialize(ir.iter(), writer)?;
                }

                if let Some(ir) = els {
                    writer.write_str("{{else}}")?;
                    serialize(ir.iter(), writer)?;
                }
                writer.write_str("{{/if}}")?;
            }
            HIR::Each(a) => {
                let Each { args, body, expr } = &**a;
                use syn::Expr::*;
                let args = if let Paren(ExprParen { expr, .. }) = args {
                    &**expr
                } else {
                    args
                };
                let receiver = if let Tuple(_) = expr {
                    if let MethodCall(syn::ExprMethodCall {
                        receiver, method, ..
                    }) = args
                    {
                        assert_eq!(method.to_string(), "enumerate");
                        &**receiver
                    } else {
                        unreachable!()
                    }
                } else {
                    args
                };

                let any = match receiver {
                    MethodCall(ExprMethodCall {
                        receiver, method, ..
                    }) => {
                        assert_eq!(method.to_string(), "__into_citer");
                        if let Paren(ExprParen { expr, .. }) = &**receiver {
                            if let Reference(ExprReference { expr, .. }) = &**expr {
                                if let Paren(ExprParen { expr, .. }) = &**expr {
                                    let mut expr = *expr.clone();
                                    visitor.visit_expr_mut(&mut expr);
                                    expr
                                } else {
                                    unreachable!()
                                }
                            } else {
                                unreachable!()
                            }
                        } else {
                            unreachable!()
                        }
                    }
                    Paren(ExprParen { expr, .. }) => {
                        let mut expr = *expr.clone();
                        visitor.visit_expr_mut(&mut expr);
                        expr
                    }
                    expr @ Range(_) => {
                        let mut expr = expr.clone();
                        visitor.visit_expr_mut(&mut expr);
                        expr
                    }
                    _ => unreachable!(),
                };
                writer.write_str("{{#each ")?;
                writer.write_str(&quote!(#any).to_string())?;
                writer.write_str(" }}")?;
                serialize(body.iter(), writer)?;
                writer.write_str("{{/each}}")?;
            }
        }
    }

    Ok(())
}

struct SerializeResolve;

impl VisitMut for SerializeResolve {}

struct Serialize;

impl VisitMut for Serialize {
    fn visit_expr_mut(&mut self, expr: &mut syn::Expr) {
        use syn::Expr::*;
        match expr {
            Field(ExprField {
                base,
                member,
                attrs,
                ..
            }) => {
                if let Path(ExprPath { path, .. }) = &mut **base {
                    if let Some(ident) = path.get_ident() {
                        let ident = ident.to_string();
                        if ident == "self" || ident.starts_with("__key") {
                            use syn::Member::*;
                            let path = match member {
                                Named(i) => {
                                    let mut segments = Punctuated::<PathSegment, Token![::]>::new();
                                    segments.push(PathSegment {
                                        ident: i.clone(),
                                        arguments: Default::default(),
                                    });
                                    syn::Path {
                                        leading_colon: None,
                                        segments,
                                    }
                                }
                                Unnamed(i) => {
                                    let mut segments = Punctuated::<PathSegment, Token![::]>::new();
                                    segments.push(PathSegment {
                                        ident: format_ident!("_{}", i.index),
                                        arguments: Default::default(),
                                    });
                                    syn::Path {
                                        leading_colon: None,
                                        segments,
                                    }
                                }
                            };
                            *expr = syn::Expr::Path(ExprPath {
                                attrs: std::mem::take(attrs),
                                qself: None,
                                path,
                            });
                        }
                    }
                }
            }
            Path(ExprPath { path, .. }) => {
                if let Some(ident) = path.get_ident() {
                    let ident = ident.to_string();
                    if ident.starts_with("__index__") {
                        let mut segments = Punctuated::<PathSegment, Token![::]>::new();
                        segments.push(PathSegment {
                            ident: format_ident!("index0"),
                            arguments: Default::default(),
                        });
                        *path = syn::Path {
                            leading_colon: None,
                            segments,
                        }
                    } else if ident.starts_with("__key__") {
                        let mut segments = Punctuated::<PathSegment, Token![::]>::new();
                        segments.push(PathSegment {
                            ident: format_ident!("this"),
                            arguments: Default::default(),
                        });
                        *path = syn::Path {
                            leading_colon: None,
                            segments,
                        }
                    }
                }
            }
            a => visit_mut::visit_expr_mut(self, a),
        };
    }
}

#[cfg(test)]
mod test {
    // TODO
}
