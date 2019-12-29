use proc_macro2::TokenStream;
use quote::quote;

use super::{CodeGen, EachCodeGen, IfElseCodeGen, HIR};

pub struct HTMLCodeGen;

impl EachCodeGen for HTMLCodeGen {}

impl IfElseCodeGen for HTMLCodeGen {}

impl CodeGen for HTMLCodeGen {
    fn gen(&self, v: &[HIR]) -> TokenStream {
        let mut tokens = TokenStream::new();
        for i in v {
            use HIR::*;
            tokens.extend(match i {
                Local(a) => quote!(#a),
                Lit(a) => quote!(_fmt.write_str(#a)?;),
                Safe(a) => quote!(::std::fmt::Display::fmt(&(#a), _fmt)?;),
                Expr(a) => quote!(::yarte::Render::render(&(#a), _fmt)?;),
                Each(a) => self.gen_each(&a),
                IfElse(a) => self.gen_if_else(&a),
            })
        }
        tokens
    }
}
