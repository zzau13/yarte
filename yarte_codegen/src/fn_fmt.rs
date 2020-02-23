use proc_macro2::TokenStream;
use quote::quote;

use yarte_hir::HIR;

use crate::CodeGen;

pub struct FnFmtCodeGen<T: CodeGen> {
    codegen: T,
}

impl<T: CodeGen> FnFmtCodeGen<T> {
    pub fn new(codegen: T) -> FnFmtCodeGen<T> {
        FnFmtCodeGen { codegen }
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
        quote! {
            {
                use std::fmt::Write;
                let mut buf = String::with_capacity(#size_hint);
                write!(buf, "{}", yarte_write::DisplayFn::new(|_fmt| {
                    #body
                    Ok(())
                })).map(|_| buf)
            }
        }
    }
}
