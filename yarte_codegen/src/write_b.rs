use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use yarte_hir::HIR;

use crate::CodeGen;

pub struct WriteBCodeGen<T: CodeGen> {
    codegen: T,
    parent: Ident,
}

impl<T: CodeGen> WriteBCodeGen<T> {
    pub fn new(codegen: T, parent: &'static str) -> WriteBCodeGen<T> {
        WriteBCodeGen {
            codegen,
            parent: format_ident!("{}", parent),
        }
    }

    fn body(&mut self, nodes: Vec<HIR>) -> TokenStream {
        self.codegen.gen(nodes)
    }
}

impl<T: CodeGen> CodeGen for WriteBCodeGen<T> {
    fn gen(&mut self, v: Vec<HIR>) -> TokenStream {
        let body = self.body(v);
        let parent = &self.parent;
        quote! {
            {
                #[allow(unused_imports)]
                use #parent::*;
                macro_rules! buf_ref {
                    ($b:expr) => { &mut $b };
                }
                #body
            }
        }
    }
}
