use proc_macro2::TokenStream;
use quote::quote;

use super::{CodeGen, EachCodeGen, IfElseCodeGen, HIR};

pub struct TextCodeGen;

impl EachCodeGen for TextCodeGen {}
impl IfElseCodeGen for TextCodeGen {}

impl CodeGen for TextCodeGen {
    fn gen(&mut self, v: Vec<HIR>) -> TokenStream {
        let mut tokens = TokenStream::new();
        for i in v {
            use HIR::*;
            tokens.extend(match i {
                Local(a) => quote!(#a),
                Lit(a) => quote!(_fmt.write_str(#a)?;),
                Safe(a) | Expr(a) => quote!(&(#a).fmt(_fmt)?;),
                Each(a) => self.gen_each(*a),
                IfElse(a) => self.gen_if_else(*a),
            });
        }
        tokens
    }
}
