use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use yarte_hir::{Struct, HIR};

use crate::{CodeGen, EachCodeGen, IfElseCodeGen};

pub struct FixedCodeGen<'a, T: CodeGen> {
    codegen: T,
    s: &'a Struct<'a>,
    parent: Ident,
}

impl<'a, T: CodeGen> FixedCodeGen<'a, T> {
    pub fn new<'n>(codegen: T, s: &'n Struct, parent: &'static str) -> FixedCodeGen<'n, T> {
        FixedCodeGen {
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
            quote!(yarte::TemplateFixedTrait),
            &quote!(
                fn call(&self, buf: &mut [std::mem::MaybeUninit<u8>]) -> Option<&[u8]> {
                    unsafe {
                    #[allow(unused_import)]
                    use #parent::*;
                    macro_rules! buf_ptr {
                        () => { buf as *mut _ as * mut u8 };
                    }
                    macro_rules! len {
                        () => { buf.len() };
                    }
                    let mut buf_cur = 0;

                    #[allow(unused_macros)]
                    macro_rules! __yarte_check_write {
                        ($len:expr, $write:block) => {
                            if len!() < buf_cur + $len {
                                return None;
                            } else $write
                        };
                    }
                    #[allow(unused_macros)]
                    macro_rules! __yarte_write_bytes_long {
                        ($b:expr) => {
                            __yarte_check_write!($b.len(), {
                                // Not use copy_from_slice for elide double checked
                                std::ptr::copy_nonoverlapping((&$b as *const _ as *const u8), buf_ptr!().add(buf_cur), $b.len());
                                buf_cur += $b.len();
                            })
                        };
                    }

                    #nodes
                    Some(std::slice::from_raw_parts(buf as *const _ as *const u8, buf_cur))
                    }
                }
            ),
        ));
    }
}

fn literal(a: String, parent: &Ident) -> TokenStream {
    let len = a.len();
    let b = a.as_bytes();
    // https://github.com/torvalds/linux/blob/master/arch/x86/lib/memcpy_64.S
    // https://software.intel.com/content/www/us/en/develop/download/intel-64-and-ia-32-architectures-optimization-reference-manual.html
    match len {
        0 => unreachable!(),
        // For 1 to 3 bytes, is mostly faster write byte-by-byte
        1..=3 => {
            let range: TokenStream = write_bb(b);
            quote! {{
                #[doc = #a]
                __yarte_check_write!(#len, {
                    #range
                    buf_cur += #len;
                })
            }}
        }
        4..=15 => {
            quote! {{
                #[doc = #a]
                const YARTE_SLICE: [u8; #len] = [#(#b),*];
                __yarte_write_bytes_long!(YARTE_SLICE);
            }}
        }
        _ => {
            quote! {{
                #[doc = #a]
                const YARTE_SLICE: #parent::Aligned256<[u8; #len]> = #parent::Aligned256([#(#b),*]);
                __yarte_write_bytes_long!(YARTE_SLICE.0);
            }}
        }
    }
}

fn write_bb(b: &[u8]) -> TokenStream {
    b.iter()
        .enumerate()
        .map(|(i, b)| {
            quote! {
                *buf_ptr!().add(buf_cur + #i) = #b;
            }
        })
        .flatten()
        .collect()
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
                Lit(a) => literal(a, &parent),
                Safe(a) | Expr(a) => {
                    quote!(buf_cur += &(#a).__render_it_safe(&mut buf[buf_cur..])?;)
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
            Lit(a) => literal(a, &parent),
            Safe(a) => quote!(buf_cur += &(#a).__render_it_safe(&mut buf[buf_cur..])?;),
            Expr(a) => quote!(buf_cur += &(#a).__render_it(&mut buf[buf_cur..])?;),
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

#[cfg(feature = "html-min")]
pub mod html_min {
    use super::*;
    use yarte_dom::DOMFmt;

    pub struct HTMLMinFixedCodeGen(pub &'static str);
    impl EachCodeGen for HTMLMinFixedCodeGen {}
    impl IfElseCodeGen for HTMLMinFixedCodeGen {}

    impl CodeGen for HTMLMinFixedCodeGen {
        fn gen(&mut self, v: Vec<HIR>) -> TokenStream {
            let parent = self.0;
            let dom: DOMFmt = v.into();
            gen(self, dom.0, parent)
        }
    }
}
