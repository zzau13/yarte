use proc_macro2::TokenStream;
use quote::quote;

use super::{CodeGen, EachCodeGen, IfElseCodeGen, HIR};

fn gen<C>(codegen: &mut C, v: Vec<HIR>) -> TokenStream
where
    C: CodeGen + EachCodeGen + IfElseCodeGen,
{
    let mut tokens = TokenStream::new();
    for i in v {
        use HIR::*;
        tokens.extend(match i {
            Local(a) => quote!(#a),
            Lit(a) => quote!(_fmt.write_str(#a)?;),
            Safe(a) => quote!(&(#a).fmt(_fmt)?;),
            Expr(a) => quote!(&(#a).__renders_it(_fmt)?;),
            Each(a) => codegen.gen_each(*a),
            IfElse(a) => codegen.gen_if_else(*a),
        })
    }
    tokens
}

pub struct HTMLCodeGen;

impl EachCodeGen for HTMLCodeGen {}

impl IfElseCodeGen for HTMLCodeGen {}
impl CodeGen for HTMLCodeGen {
    fn gen(&mut self, v: Vec<HIR>) -> TokenStream {
        gen(self, v)
    }
}

#[cfg(feature = "html-min")]
pub mod html_min {
    use super::*;
    use yarte_dom::DOMFmt;

    pub struct HTMLMinCodeGen;
    impl EachCodeGen for HTMLMinCodeGen {}
    impl IfElseCodeGen for HTMLMinCodeGen {}

    impl CodeGen for HTMLMinCodeGen {
        fn gen(&mut self, v: Vec<HIR>) -> TokenStream {
            let dom: DOMFmt = v.into();
            gen(self, dom.0)
        }
    }
}
