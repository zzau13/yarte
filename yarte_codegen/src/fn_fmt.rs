use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use yarte_hir::HIR;

use crate::CodeGen;

pub struct FnFmtCodeGen<T: CodeGen> {
    codegen: T,
    parent: Ident,
}

impl<T: CodeGen> FnFmtCodeGen<T> {
    pub fn new(codegen: T, parent: &'static str) -> FnFmtCodeGen<T> {
        FnFmtCodeGen {
            codegen,
            parent: format_ident!("{}", parent),
        }
    }

    fn body(&mut self, nodes: Vec<HIR>) -> (TokenStream, usize) {
        let body = self.codegen.gen(nodes);
        // heuristic based on https://github.com/lfairy/maud
        let size_hint = body.to_string().len();

        (body, size_hint)
    }
}

impl<T: CodeGen> CodeGen for FnFmtCodeGen<T> {
    fn gen(&mut self, v: Vec<HIR>) -> TokenStream {
        let (body, size_hint) = self.body(v);
        let parent = &self.parent;
        quote! {
            {
                #[allow(unused_imports)]
                use std::fmt::{Write, Display};
                #[allow(unused_imports)]
                use #parent::*;
                let mut buf = String::with_capacity(#size_hint);
                let _ = write!(buf, "{}", #parent::DisplayFn::new(|_fmt| {
                    #body
                    Ok(())
                }));
                buf
            }
        }
    }
}
