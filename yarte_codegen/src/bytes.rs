use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use yarte_hir::{Struct, HIR};

use crate::EachCodeGen;
use crate::{CodeGen, IfElseCodeGen};

pub struct BytesCodeGen<'a, T: CodeGen> {
    codegen: T,
    s: &'a Struct<'a>,
    parent: Ident,
    buf: Ident,
}

impl<'a, T: CodeGen> BytesCodeGen<'a, T> {
    pub fn new<'n>(
        codegen: T,
        s: &'n Struct,
        buf: Ident,
        parent: &'static str,
    ) -> BytesCodeGen<'n, T> {
        BytesCodeGen {
            codegen,
            s,
            parent: format_ident!("{}", parent),
            buf,
        }
    }

    #[inline]
    fn template(&mut self, nodes: Vec<HIR>, tokens: &mut TokenStream) {
        let nodes = self.codegen.gen(nodes);
        let parent = &self.parent;
        let buf = &self.buf;
        tokens.extend(self.s.implement_head(
            quote!(#parent::TemplateBytesTrait),
            &quote!(
                fn call<B: #parent::Buffer>(&self, capacity: usize) -> B::Freeze {
                    use #parent::*;
                    let mut #buf: B = #parent::Buffer::with_capacity(capacity);
                    #nodes
                    #buf.freeze()
                }

                fn ccall<B: #parent::Buffer>(self, capacity: usize) -> B::Freeze {
                    use #parent::*;
                    let mut #buf: B = #parent::Buffer::with_capacity(capacity);
                    #nodes
                    #buf.freeze()
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

pub struct TextBytesCodeGen<'a> {
    buf: &'a syn::Expr,
    parent: Ident,
}

impl<'a> TextBytesCodeGen<'a> {
    pub fn new<'n>(buf: &'n syn::Expr, parent: &'static str) -> TextBytesCodeGen<'n> {
        TextBytesCodeGen {
            buf,
            parent: format_ident!("{}", parent),
        }
    }
}

impl<'a> EachCodeGen for TextBytesCodeGen<'a> {}
impl<'a> IfElseCodeGen for TextBytesCodeGen<'a> {}

impl<'a> CodeGen for TextBytesCodeGen<'a> {
    fn gen(&mut self, v: Vec<HIR>) -> TokenStream {
        let mut tokens = TokenStream::new();
        for i in v {
            use HIR::*;
            tokens.extend(match i {
                Local(a) => quote!(#a),
                Lit(a) => {
                    let parent = &self.parent;
                    let buf = &self.buf;
                    quote!(#parent::Buffer::extend_from_slice(&mut #buf, #a.as_bytes());)
                }
                Safe(a) | Expr(a) => {
                    let buf = &self.buf;
                    quote!(&(#a).__render_itb_safe(&mut #buf);)
                }
                Each(a) => self.gen_each(*a),
                IfElse(a) => self.gen_if_else(*a),
            });
        }
        tokens
    }
}

fn gen<C>(codegen: &mut C, v: Vec<HIR>, parent: Ident, buf: TokenStream) -> TokenStream
where
    C: CodeGen + EachCodeGen + IfElseCodeGen,
{
    let mut tokens = TokenStream::new();
    for i in v {
        use HIR::*;
        tokens.extend(match i {
            Local(a) => quote!(#a),
            Lit(a) => quote!(#parent::Buffer::extend_from_slice(&mut #buf, #a.as_bytes());),
            Safe(a) => quote!(&(#a).__render_itb_safe(&mut #buf);),
            Expr(a) => quote!(&(#a).__render_itb(&mut #buf);),
            Each(a) => codegen.gen_each(*a),
            IfElse(a) => codegen.gen_if_else(*a),
        })
    }
    tokens
}

pub struct HTMLBytesCodeGen<'a> {
    buf: &'a syn::Expr,
    parent: Ident,
}

impl<'a> HTMLBytesCodeGen<'a> {
    pub fn new<'n>(buf: &'n syn::Expr, parent: &'static str) -> HTMLBytesCodeGen<'n> {
        HTMLBytesCodeGen {
            buf,
            parent: format_ident!("{}", parent),
        }
    }
}

impl<'a> EachCodeGen for HTMLBytesCodeGen<'a> {}

impl<'a> IfElseCodeGen for HTMLBytesCodeGen<'a> {}
impl<'a> CodeGen for HTMLBytesCodeGen<'a> {
    fn gen(&mut self, v: Vec<HIR>) -> TokenStream {
        let parent = self.parent.clone();
        let buf = self.buf;
        gen(self, v, parent, quote!(#buf))
    }
}

#[cfg(feature = "html-min")]
pub mod html_min {
    use super::*;
    use yarte_dom::DOMFmt;

    pub struct HTMLMinBytesCodeGen<'a> {
        buf: &'a syn::Expr,
        parent: Ident,
    }

    impl<'a> HTMLMinBytesCodeGen<'a> {
        pub fn new<'n>(buf: &'n syn::Expr, parent: &'static str) -> HTMLMinBytesCodeGen<'n> {
            HTMLMinBytesCodeGen {
                buf,
                parent: format_ident!("{}", parent),
            }
        }
    }

    impl<'a> EachCodeGen for HTMLMinBytesCodeGen<'a> {}
    impl<'a> IfElseCodeGen for HTMLMinBytesCodeGen<'a> {}

    impl<'a> CodeGen for HTMLMinBytesCodeGen<'a> {
        fn gen(&mut self, v: Vec<HIR>) -> TokenStream {
            let dom: DOMFmt = v.into();
            let parent = self.parent.clone();
            let buf = self.buf;
            gen(self, dom.0, parent, quote!(#buf))
        }
    }
}
