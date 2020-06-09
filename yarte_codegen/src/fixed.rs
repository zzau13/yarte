use proc_macro2::{Ident, TokenStream};
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
                unsafe fn call(&self, buf: &mut [u8]) -> Option<usize> {
                    macro_rules! buf_ptr {
                        () => { buf as *mut [u8] as * mut u8 };
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
                                std::ptr::copy_nonoverlapping((&$b as *const [u8] as *const u8), buf_ptr!().add(buf_cur), $b.len());
                                buf_cur += $b.len();
                            })
                        };
                    }

                    #nodes
                    Some(buf_cur)
                }
            ),
        ));
    }
}

fn literal(a: String, parent: &Ident) -> TokenStream {
    let len = a.len();
    let b = a.as_bytes();
    // https://github.com/torvalds/linux/blob/master/arch/x86/lib/memcpy_64.S
    // https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-optimization-manual.pdf
    match len {
        0 => unreachable!(),
        // For 1 to 7 bytes, is mostly faster write byte-by-byte
        1..=7 => {
            let range: TokenStream = write_bb(b);
            quote! {{
                #[doc = #a]
                __yarte_check_write!(#len, {
                    #range
                    buf_cur += #len;
                })
            }}
        }
        // For 8 to 15 bytes, is mostly faster write 4 bytes by 4 bytes
        // Duplicate data for improve global performance
        8..=15 => {
            let range: TokenStream = write_bb(b);
            let range32: TokenStream = b
                .chunks(4)
                .enumerate()
                .map(|(i, x)| {
                    if x.len() == 4 {
                        // Safe conversion because the chunk size is 4 and, yes clippy, read unaligned
                        #[allow(clippy::cast_ptr_alignment)]
                        let x = unsafe { (x as *const [u8] as *const u32).read_unaligned() };
                        let i = 4 * i;
                        quote!(*(buf_ptr!().add(buf_cur + #i) as *mut u32) = #x;)
                    } else {
                        let mut tokens = TokenStream::new();
                        for (j, x) in x.iter().enumerate() {
                            let i = 4 * i + j;
                            tokens.extend(quote!(*buf_ptr!().add(buf_cur + #i) = #x;));
                        }
                        tokens
                    }
                })
                .flatten()
                .collect();

            quote! {{
                #[doc = #a]
                __yarte_check_write!(#len, {
                    if (buf_ptr!() as usize).trailing_zeros() < 2 {
                        #range
                    } else {
                        debug_assert_eq!((buf_ptr!() as usize) % 4, 0);
                        #range32
                    }
                    buf_cur += #len;
                })
            }}
        }
        _ => {
            quote! {{
                #[doc = #a]
                const YARTE_SLICE: #parent::Aligned64<[u8; #len]> = #parent::Aligned64([#(#b),*]);
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
                    // TODO: check slice builder, raw or cut
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
            Lit(a) => literal(a, &parent),
            // TODO: check slice builder, raw or cut
            Safe(a) => quote!(buf_cur += #parent::RenderSafe::render(&(#a), &mut buf[buf_cur..])?;),
            Expr(a) => {
                // TODO: check slice builder, raw or cut
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
