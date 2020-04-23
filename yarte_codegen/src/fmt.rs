use proc_macro2::TokenStream;
use quote::quote;

use yarte_hir::{Struct, HIR};

use crate::CodeGen;

pub struct FmtCodeGen<'a, T: CodeGen> {
    codegen: T,
    s: &'a Struct<'a>,
}

impl<'a, T: CodeGen> FmtCodeGen<'a, T> {
    pub fn new<'n>(codegen: T, s: &'n Struct) -> FmtCodeGen<'n, T> {
        FmtCodeGen { codegen, s }
    }

    #[inline]
    fn template(&self, size_hint: usize, tokens: &mut TokenStream) {
        tokens.extend(self.s.implement_head(
            quote!(yarte::TemplateTrait),
            &quote!(
            fn size_hint() -> usize {
                #size_hint
            }),
        ));
    }

    fn display(&mut self, nodes: Vec<HIR>, tokens: &mut TokenStream) -> usize {
        let nodes = self.codegen.gen(nodes);
        // heuristic based on https://github.com/lfairy/maud
        let size_hint = nodes.to_string().len();
        let func = quote!(
            fn fmt(&self, _fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                #nodes
                Ok(())
            }
        );

        tokens.extend(self.s.implement_head(quote!(std::fmt::Display), &func));

        size_hint
    }
}

impl<'a, T: CodeGen> CodeGen for FmtCodeGen<'a, T> {
    fn gen(&mut self, v: Vec<HIR>) -> TokenStream {
        let mut tokens = TokenStream::new();

        let size_hint = self.display(v, &mut tokens);
        self.template(size_hint, &mut tokens);

        tokens
    }
}
