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
                    let mut #buf: B = B::with_capacity(capacity);
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
}

impl<'a> TextBytesCodeGen<'a> {
    pub fn new(buf: &syn::Expr) -> TextBytesCodeGen {
        TextBytesCodeGen { buf }
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
                    let buf = &self.buf;
                    let buf = &quote!(#buf);
                    literal(a, buf)
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

fn gen<C>(codegen: &mut C, v: Vec<HIR>, buf: TokenStream) -> TokenStream
where
    C: CodeGen + EachCodeGen + IfElseCodeGen,
{
    let mut tokens = TokenStream::new();
    for i in v {
        use HIR::*;
        tokens.extend(match i {
            Local(a) => quote!(#a),
            Lit(a) => literal(a, &buf),
            Safe(a) => quote!(&(#a).__render_itb_safe(&mut #buf);),
            Expr(a) => quote!(&(#a).__render_itb(&mut #buf);),
            Each(a) => codegen.gen_each(*a),
            IfElse(a) => codegen.gen_if_else(*a),
        })
    }
    tokens
}

fn literal(a: String, buf: &TokenStream) -> TokenStream {
    let len = a.len();
    let b = a.as_bytes();
    // https://github.com/torvalds/linux/blob/master/arch/x86/lib/memcpy_64.S
    // https://software.intel.com/content/www/us/en/develop/download/intel-64-and-ia-32-architectures-optimization-reference-manual.html
    match len {
        0 => unreachable!(),
        // For 1 to 3 bytes, is mostly faster write byte-by-byte
        1..=3 => {
            let range: TokenStream = write_bb(b, buf);
            quote! {{
                #[doc = #a]
                #buf.reserve(#len);
                unsafe {
                    #range
                    #buf.advance(#len);
                }
            }}
        }
        _ => {
            quote! {
                #buf.extend_from_slice(#a.as_bytes());
            }
        }
    }
}

fn write_bb(b: &[u8], buf: &TokenStream) -> TokenStream {
    b.iter()
        .enumerate()
        .map(|(i, b)| {
            quote! {
                *#buf.buf_ptr().add(#i) = #b;
            }
        })
        .flatten()
        .collect()
}

pub struct HTMLBytesCodeGen<'a> {
    buf: &'a syn::Expr,
}

impl<'a> HTMLBytesCodeGen<'a> {
    pub fn new(buf: &syn::Expr) -> HTMLBytesCodeGen {
        HTMLBytesCodeGen { buf }
    }
}

impl<'a> EachCodeGen for HTMLBytesCodeGen<'a> {}

impl<'a> IfElseCodeGen for HTMLBytesCodeGen<'a> {}
impl<'a> CodeGen for HTMLBytesCodeGen<'a> {
    fn gen(&mut self, v: Vec<HIR>) -> TokenStream {
        let buf = self.buf;
        gen(self, v, quote!(#buf))
    }
}

#[cfg(feature = "html-min")]
pub mod html_min {
    use super::*;
    use yarte_dom::DOMFmt;

    pub struct HTMLMinBytesCodeGen<'a> {
        buf: &'a syn::Expr,
    }

    impl<'a> HTMLMinBytesCodeGen<'a> {
        pub fn new(buf: &syn::Expr) -> HTMLMinBytesCodeGen {
            HTMLMinBytesCodeGen { buf }
        }
    }

    impl<'a> EachCodeGen for HTMLMinBytesCodeGen<'a> {}
    impl<'a> IfElseCodeGen for HTMLMinBytesCodeGen<'a> {}

    impl<'a> CodeGen for HTMLMinBytesCodeGen<'a> {
        fn gen(&mut self, v: Vec<HIR>) -> TokenStream {
            let dom: DOMFmt = v.into();
            let buf = self.buf;
            gen(self, dom.0, quote!(#buf))
        }
    }
}
