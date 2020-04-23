use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::{CodeGen, EachCodeGen, IfElseCodeGen, HIR};

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
            Lit(a) => quote!(_fmt.write_str(#a)?;),
            Safe(a) => quote!(std::fmt::Display::fmt(&(#a), _fmt)?;),
            Expr(a) => quote!(#parent::Render::render(&(#a), _fmt)?;),
            Each(a) => codegen.gen_each(*a),
            IfElse(a) => codegen.gen_if_else(*a),
        })
    }
    tokens
}

pub struct HTMLCodeGen(pub &'static str);

impl EachCodeGen for HTMLCodeGen {}

impl IfElseCodeGen for HTMLCodeGen {}
impl CodeGen for HTMLCodeGen {
    fn gen(&mut self, v: Vec<HIR>) -> TokenStream {
        let parent = self.0;
        gen(self, v, parent)
    }
}

#[cfg(feature = "html-min")]
pub mod html_min {
    use super::*;
    use yarte_dom::DOMFmt;

    pub struct HTMLMinCodeGen(pub &'static str);
    impl EachCodeGen for HTMLMinCodeGen {}
    impl IfElseCodeGen for HTMLMinCodeGen {}

    impl CodeGen for HTMLMinCodeGen {
        fn gen(&mut self, v: Vec<HIR>) -> TokenStream {
            let parent = self.0;
            let dom: DOMFmt = v.into();
            gen(self, dom.0, parent)
        }
    }
}
