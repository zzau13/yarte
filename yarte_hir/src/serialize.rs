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
                                attrs: attrs.drain(..).collect(),
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
    use std::collections::BTreeMap;

    use quote::quote;
    use syn::parse2;

    use yarte_helpers::config::Config;
    use yarte_parser::{
        emitter, parse,
        source_map::{clean, get_cursor},
    };

    use crate::{generate, visit_derive};

    use super::*;

    fn test(src: &str) -> Vec<HIR> {
        let i = quote! {
            #[template(src = #src)]
            struct __Foo__;
        };
        let config = &Config::new("");
        let der = parse2(i).unwrap();
        let s = visit_derive(&der, config).unwrap();
        let mut src = BTreeMap::new();
        src.insert(s.path.clone(), s.src.clone());
        let sources = parse(get_cursor(&s.path, &s.src)).unwrap();
        let mut ctx = BTreeMap::new();
        ctx.insert(&s.path, sources);

        let ir = generate(config, &s, &ctx, Default::default())
            .unwrap_or_else(|e| emitter(&src, config, e.into_iter()));
        clean();

        ir
    }

    #[test]
    fn empty() {
        let src = "";
        let ir = test(src);
        let mut buff = String::new();
        serialize(ir.iter(), &mut buff).unwrap();
        assert_eq!(buff, "");
    }

    #[test]
    fn expr() {
        let src = "Hello, {{ world }}!";
        let ir = test(src);
        let mut buff = String::new();
        serialize(ir.iter(), &mut buff).unwrap();
        assert_eq!(buff, src);
    }

    #[test]
    fn if_else() {
        let src = "Hello, {{#if flag }}foo{{else if flag2 }}bar{{else}}fol{{/if}}!";
        let ir = test(src);
        let mut buff = String::new();
        serialize(ir.iter(), &mut buff).unwrap();
        assert_eq!(buff, src);
    }

    #[test]
    fn if_else_let() {
        let src = "Hello, {{#if let Some(foo) = flag }}foo{{ foo }}{{else if let Some(foo) = flag2 }}{{ foo }}bar{{/if}}!";
        let ir = test(src);
        let mut buff = String::new();
        serialize(ir.iter(), &mut buff).unwrap();
        assert_eq!(buff, "Hello, {{#if let Some ( foo__0x00000000 ) = flag }}foo{{ foo__0x00000000 }}{{else if let Some ( foo__0x00000001 ) = flag2 }}{{ foo__0x00000001 }}bar{{/if}}!");
    }

    #[test]
    fn each() {
        let src = "Hello, {{#each iter }}foo{{ this }}{{/each}}!";
        let ir = test(src);
        let mut buff = String::new();
        serialize(ir.iter(), &mut buff).unwrap();
        assert_eq!(buff, src);
    }

    #[test]
    fn each_0() {
        let src = "Hello, {{#each iter }}foo{{ index0 }}{{/each}}!";
        let ir = test(src);
        let mut buff = String::new();
        serialize(ir.iter(), &mut buff).unwrap();
        assert_eq!(buff, src);
    }

    #[test]
    fn each_1() {
        let src = "Hello, {{#each 0 .. n }}foo{{ this }}{{/each}}!";
        let ir = test(src);
        let mut buff = String::new();
        serialize(ir.iter(), &mut buff).unwrap();
        assert_eq!(buff, src);
    }

    #[test]
    fn each_2() {
        let src = "Hello, {{#each 0 .. n }}foo{{ index0 }}{{/each}}!";
        let ir = test(src);
        let mut buff = String::new();
        serialize(ir.iter(), &mut buff).unwrap();
        assert_eq!(buff, src);
    }

    #[test]
    fn each_3() {
        let src = "Hello, {{#each iter }}foo{{ index }}{{/each}}!";
        let ir = test(src);
        let mut buff = String::new();
        serialize(ir.iter(), &mut buff).unwrap();
        assert_eq!(
            buff,
            "Hello, {{#each iter }}foo{{ ( index0 + 1 ) }}{{/each}}!"
        );
    }

    #[test]
    fn each_4() {
        let src = "Hello, {{#each iter }}foo{{ first }}{{/each}}!";
        let ir = test(src);
        let mut buff = String::new();
        serialize(ir.iter(), &mut buff).unwrap();
        assert_eq!(
            buff,
            "Hello, {{#each iter }}foo{{ ( index0 == 0 ) }}{{/each}}!"
        );
    }
}
