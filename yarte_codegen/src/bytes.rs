use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use yarte_hir::{Struct, HIR};

use crate::CodeGen;

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
                fn call(&self, capacity: usize) -> Option<#parent::Bytes> {
                    use #parent::*;
                    let mut bytes_mut = #parent::BytesMut::with_capacity(capacity);
                    let buf = #parent::BufMut::bytes_mut(&mut bytes_mut);
                    let mut buf_cur = 0;
                    unsafe {
                        macro_rules! buf_ptr {
                            () => { buf as *mut _ as * mut u8 };
                        }
                        macro_rules! len {
                            () => { buf.len() };
                        }

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
                        #parent::BufMut::advance_mut(&mut bytes_mut, buf_cur)
                    }
                    Some(bytes_mut.freeze())
                }

                fn ccall(self, capacity: usize) -> Option<#parent::Bytes> {
                    use #parent::*;
                    let mut bytes_mut = #parent::BytesMut::with_capacity(capacity);
                    let buf = #parent::BufMut::bytes_mut(&mut bytes_mut);
                    let mut buf_cur = 0;
                    unsafe {
                        macro_rules! buf_ptr {
                            () => { buf as *mut _ as * mut u8 };
                        }
                        macro_rules! len {
                            () => { buf.len() };
                        }

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
                        #parent::BufMut::advance_mut(&mut bytes_mut, buf_cur)
                    }
                    Some(bytes_mut.freeze())
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
