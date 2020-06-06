use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use yarte_hir::{Struct, HIR};

use crate::{CodeGen, EachCodeGen, IfElseCodeGen};

pub struct FixedCodeGen<'a, T: CodeGen> {
    codegen: T,
    s: &'a Struct<'a>,
}

impl<'a, T: CodeGen> FixedCodeGen<'a, T> {
    pub fn new<'n>(codegen: T, s: &'n Struct) -> FixedCodeGen<'n, T> {
        FixedCodeGen { codegen, s }
    }

    #[inline]
    fn template(&mut self, nodes: Vec<HIR>, tokens: &mut TokenStream) {
        let nodes = self.codegen.gen(nodes);
        tokens.extend(self.s.implement_head(
            quote!(yarte::TemplateFixedTrait),
            &quote!(
                fn call(&self, buf: &mut [u8]) -> Option<usize> {
                    let mut buf_cur = 0;
                    macro_rules! __yarte_write_bytes {
                        ($b:ident) => {
                            if buf.len() < buf_cur + $b.len() {
                                return None;
                            } else {
                                (&mut buf[buf_cur..buf_cur + $b.len()]).copy_from_slice(&$b);
                                buf_cur += $b.len();
                            }
                        };
                    }
                    #nodes
                    Some(buf_cur)
                }
            ),
        ));
    }
}

impl<'a, T: CodeGen> CodeGen for FixedCodeGen<'a, T> {
    fn gen(&mut self, v: Vec<HIR>) -> TokenStream {
        let mut tokens = TokenStream::new();

        self.template(v, &mut tokens);

        tokens
    }
}

pub struct TextFixedCodeGen(pub &'static str);

impl EachCodeGen for TextFixedCodeGen {}
impl IfElseCodeGen for TextFixedCodeGen {}

impl CodeGen for TextFixedCodeGen {
    fn gen(&mut self, v: Vec<HIR>) -> TokenStream {
        let mut tokens = TokenStream::new();
        let parent = format_ident!("{}", self.0);
        for i in v {
            use HIR::*;
            tokens.extend(match i {
                Local(a) => quote!(#a),
                Lit(a) => {
                    let len = a.len();
                    let b = a.as_bytes();
                    quote! {{
                        #[doc = #a]
                        const YARTE_SLICE: [u8; #len] = [#(#b),*];
                        __yarte_write_bytes!(YARTE_SLICE);
                    }}
                }
                Safe(a) | Expr(a) => {
                    quote!(buf_cur += #parent::RenderSafe::render(&(#a), &mut buf[buf_cur..])?;)
                }
                Each(a) => self.gen_each(*a),
                IfElse(a) => self.gen_if_else(*a),
            });
        }
        tokens
    }
}

fn gen<C>(codegen: &mut C, v: Vec<HIR>, parent: &str) -> TokenStream
where
    C: CodeGen + EachCodeGen + IfElseCodeGen,
{
    let mut tokens = TokenStream::new();
    let parent = format_ident!("{}", parent);
    for i in v {
        use HIR::*;
        tokens.extend(match i {
            Local(a) => quote!(#a),
            Lit(a) => {
                let len = a.len();
                let b = a.as_bytes();
                quote! {{
                    #[doc = #a]
                    const YARTE_SLICE: [u8; #len] = [#(#b),*];
                    __yarte_write_bytes!(YARTE_SLICE);
                }}
            }
            Safe(a) => quote!(buf_cur += #parent::RenderSafe::render(&(#a), &mut buf[buf_cur..])?;),
            Expr(a) => {
                quote!(buf_cur += #parent::RenderFixed::render(&(#a), &mut buf[buf_cur..])?;)
            }
            Each(a) => codegen.gen_each(*a),
            IfElse(a) => codegen.gen_if_else(*a),
        })
    }
    tokens
}

pub struct HTMLFixedCodeGen(pub &'static str);

impl EachCodeGen for HTMLFixedCodeGen {}

impl IfElseCodeGen for HTMLFixedCodeGen {}
impl CodeGen for HTMLFixedCodeGen {
    fn gen(&mut self, v: Vec<HIR>) -> TokenStream {
        let parent = self.0;
        gen(self, v, parent)
    }
}
