use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use yarte_hir::HIR;

use crate::CodeGen;

pub struct AttrBCodeGen<T: CodeGen> {
    arg: bool,
    codegen: T,
    parent: Ident,
}

impl<T: CodeGen> AttrBCodeGen<T> {
    pub fn new(codegen: T, parent: &'static str, arg: bool) -> AttrBCodeGen<T> {
        AttrBCodeGen {
            arg,
            codegen,
            parent: format_ident!("{}", parent),
        }
    }

    fn body(&mut self, nodes: Vec<HIR>) -> TokenStream {
        self.codegen.gen(nodes)
    }
}

impl<T: CodeGen> CodeGen for AttrBCodeGen<T> {
    fn gen(&mut self, v: Vec<HIR>) -> TokenStream {
        let body = self.body(v);
        let parent = &self.parent;
        let arg = self.arg;
        if arg {
            quote! {{
                #[allow(unused_imports)]
                use #parent::*;
                macro_rules! buf_ref {
                    ($b:expr) => { &mut $b };
                }

                #body
            }}
        } else {
            quote! {
                {
                    #[allow(unused_imports)]
                    use #parent::*;
                    macro_rules! buf_ref {
                        ($b:expr) => { &mut $b };
                    }
                    #[inline]
                    fn __yarte_context<F: yarte::Buffer, C: FnOnce(F) -> F>(f: C) -> F {
                        thread_local! {
                            static SIZE: std::cell::Cell<usize> = std::cell::Cell::new(0);
                        }
                        let tmp: F = yarte::Buffer::with_capacity(SIZE.with(|v| v.get()));
                        let tmp = f(tmp);
                        SIZE.with(|v| if v.get() < tmp.len() {
                            v.set(tmp.len())
                        });
                        tmp
                    }

                    __yarte_context(|mut __buf| {
                        #body
                        __buf
                    })
                }
            }
        }
    }
}
