use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use yarte_hir::{Struct, HIR};

use crate::EachCodeGen;
use crate::{CodeGen, IfElseCodeGen};

pub struct BytesCodeGen<'a, T: CodeGen> {
    codegen: T,
    s: &'a Struct<'a>,
    parent: Ident,
}

impl<'a, T: CodeGen> BytesCodeGen<'a, T> {
    pub fn new<'n>(codegen: T, s: &'n Struct, parent: &'static str) -> BytesCodeGen<'n, T> {
        BytesCodeGen {
            codegen,
            s,
            parent: format_ident!("{}", parent),
        }
    }

    #[inline]
    fn template(&mut self, nodes: Vec<HIR>, tokens: &mut TokenStream) {
        let nodes = self.codegen.gen(nodes);
        let parent = &self.parent;
        tokens.extend(self.s.implement_head(
            quote!(#parent::TemplateBytesTrait),
            &quote!(
                fn call(&self, capacity: usize) -> #parent::Bytes {
                    use #parent::*;
                    let mut bytes_mut = #parent::BytesMut::with_capacity(capacity);
                    #nodes
                    bytes_mut.freeze()
                }

                fn ccall(self, capacity: usize) -> #parent::Bytes {
                    use #parent::*;
                    let mut bytes_mut = #parent::BytesMut::with_capacity(capacity);
                    #nodes
                    bytes_mut.freeze()
                }
            ),
        ));
    }
}

impl<'a, T: CodeGen> CodeGen for BytesCodeGen<'a, T> {
    fn gen(&mut self, v: Vec<HIR>) -> TokenStream {
        let mut tokens = TokenStream::new();

        self.template(v, &mut tokens);

        tokens
    }
}

pub struct TextBytesCodeGen;

impl EachCodeGen for TextBytesCodeGen {}
impl IfElseCodeGen for TextBytesCodeGen {}

impl CodeGen for TextBytesCodeGen {
    fn gen(&mut self, v: Vec<HIR>) -> TokenStream {
        let mut tokens = TokenStream::new();
        for i in v {
            use HIR::*;
            tokens.extend(match i {
                Local(a) => quote!(#a),
                Lit(a) => quote!(bytes_mut.extend_from_slice(#a.as_bytes());),
                Safe(a) | Expr(a) => quote!(&(#a).__render_itb_safe(&mut bytes_mut);),
                Each(a) => self.gen_each(*a),
                IfElse(a) => self.gen_if_else(*a),
            });
        }
        tokens
    }
}

fn gen<C>(codegen: &mut C, v: Vec<HIR>) -> TokenStream
where
    C: CodeGen + EachCodeGen + IfElseCodeGen,
{
    let mut tokens = TokenStream::new();
    for i in v {
        use HIR::*;
        tokens.extend(match i {
            Local(a) => quote!(#a),
            Lit(a) => quote!(bytes_mut.extend_from_slice(#a.as_bytes());),
            Safe(a) => quote!(&(#a).__render_itb_safe(&mut bytes_mut);),
            Expr(a) => quote!(&(#a).__render_itb(&mut bytes_mut);),
            Each(a) => codegen.gen_each(*a),
            IfElse(a) => codegen.gen_if_else(*a),
        })
    }
    tokens
}

pub struct HTMLBytesCodeGen;

impl EachCodeGen for HTMLBytesCodeGen {}

impl IfElseCodeGen for HTMLBytesCodeGen {}
impl CodeGen for HTMLBytesCodeGen {
    fn gen(&mut self, v: Vec<HIR>) -> TokenStream {
        gen(self, v)
    }
}

#[cfg(feature = "html-min")]
pub mod html_min {
    use super::*;
    use yarte_dom::DOMFmt;

    pub struct HTMLMinBytesCodeGen;
    impl EachCodeGen for HTMLMinBytesCodeGen {}
    impl IfElseCodeGen for HTMLMinBytesCodeGen {}

    impl CodeGen for HTMLMinBytesCodeGen {
        fn gen(&mut self, v: Vec<HIR>) -> TokenStream {
            let dom: DOMFmt = v.into();
            gen(self, dom.0)
        }
    }
}
