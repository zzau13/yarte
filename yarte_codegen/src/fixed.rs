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
                        ($b:expr, $len:expr, { $($t:tt)+ }) => {
                            if len!() < buf_cur + $len {
                                return None;
                            } else {
                                $($t)+
                            }
                        };
                    }
                    #[allow(unused_macros)]
                    macro_rules! __yarte_write_bytes_long {
                        ($b:expr) => {
                            __yarte_check_write!($b, $b.len(), {
                                // Not use copy_from_slice for elide double checked
                                std::ptr::copy_nonoverlapping($b.as_ptr(), buf_ptr!().add(buf_cur), $b.len());
                                buf_cur += $b.len();
                            })
                        };
                    }
                    // between 2-7 bytes
                    #[allow(unused_macros)]
                    macro_rules! __yarte_write_bytes_short {
                        ($b:ident) => {
                            __yarte_check_write!($b, $b.len(), {
                                for i in 0..$b.len() {
                                    buf_ptr!().add(buf_cur).write($b.as_ptr().add(i).read());
                                    buf_cur += 1;
                                }
                            })
                        };
                    }
                    #[allow(unused_macros)]
                    macro_rules! __yarte_write_bytes_one {
                        ($b:ident) => {
                            __yarte_check_write!($b, 1, {
                                buf_ptr!().add(buf_cur).write($b);
                                buf_cur += 1;
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
    match len {
        0 => unreachable!(),
        1 => {
            let b = b[0];
            quote! {{
                #[doc = #a]
                const YARTE_BYTE: u8 = #b;
                __yarte_write_bytes_one!(YARTE_BYTE);
            }}
        }
        // memcopy writes long-by-long (8 bytes) but pointer should be aligned.
        // For 2 to 7 bytes, is mostly faster write byte-by-byte
        // https://github.com/torvalds/linux/blob/master/arch/alpha/lib/memcpy.c#L128
        2..=7 => {
            quote! {{
                #[doc = #a]
                const YARTE_SLICE: [u8; #len] = [#(#b),*];
                __yarte_write_bytes_short!(YARTE_SLICE);
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
