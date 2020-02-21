#[cfg(feature = "wasm-app")]
pub mod client;
pub mod server {
    use proc_macro2::TokenStream;
    use quote::quote;

    use yarte_dom::dom_fmt::to_wasmfmt;
    use yarte_hir::{Struct, HIR};

    use crate::{CodeGen, EachCodeGen, IfElseCodeGen};

    pub struct WASMCodeGen<'a> {
        s: &'a Struct<'a>,
    }

    impl<'a> EachCodeGen for WASMCodeGen<'a> {}
    impl<'a> IfElseCodeGen for WASMCodeGen<'a> {}

    impl<'a> WASMCodeGen<'a> {
        pub fn new<'n>(s: &'n Struct<'n>) -> WASMCodeGen<'n> {
            WASMCodeGen { s }
        }
    }

    impl<'a> CodeGen for WASMCodeGen<'a> {
        fn gen(&mut self, ir: Vec<HIR>) -> TokenStream {
            let ir = to_wasmfmt(ir, self.s).expect("html");
            let mut tokens = TokenStream::new();
            for i in ir {
                use HIR::*;
                tokens.extend(match i {
                    Local(a) => quote!(#a),
                    Lit(a) => quote!(_fmt.write_str(#a)?;),
                    Safe(a) => quote!(::std::fmt::Display::fmt(&(#a), _fmt)?;),
                    Expr(a) => quote!(::yarte::Render::render(&(#a), _fmt)?;),
                    Each(a) => self.gen_each(*a),
                    IfElse(a) => self.gen_if_else(*a),
                })
            }
            tokens
        }
    }
}
