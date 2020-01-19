#![allow(warnings)]

use proc_macro2::TokenStream;
use quote::quote;

use yarte_config::Config;
use yarte_dom::dom_fmt::{to_wasmfmt, MARK_SCRIPT};
use yarte_hir::{Struct, HIR};

use crate::{CodeGen, EachCodeGen, IfElseCodeGen};

pub struct WASMCodeGen<'a> {
    s: &'a Struct<'a>,
    config: &'a Config<'a>,
}

impl<'a> EachCodeGen for WASMCodeGen<'a> {}
impl<'a> IfElseCodeGen for WASMCodeGen<'a> {}

impl<'a> WASMCodeGen<'a> {
    pub fn new<'n>(config: &'n Config<'n>, s: &'n Struct<'n>) -> WASMCodeGen<'n> {
        WASMCodeGen { config, s }
    }
}

impl<'a> CodeGen for WASMCodeGen<'a> {
    fn gen(&mut self, ir: Vec<HIR>) -> TokenStream {
        let ir = to_wasmfmt(ir, self.s).expect("html");
        let mut tokens = TokenStream::new();
        use HIR::*;
        for i in ir {
            use HIR::*;
            tokens.extend(match i {
                Local(a) => quote!(#a),
                Lit(a) => {
                    let mut tokens = TokenStream::new();
                    let mut chunks = a.split(MARK_SCRIPT);
                    let head = chunks.next().unwrap();
                    tokens.extend(quote!(_fmt.write_str(#head)?;));
                    if let Some(tail) = chunks.next() {
                        tokens.extend(quote!(::std::fmt::Display::fmt(
                            &::yarte::serde_json::to_string(&self).map_err(|_| ::yarte::Error)?, _fmt)?;
                        ));
                        tokens.extend(quote!(_fmt.write_str(#tail)?;));
                    }
                    assert!(chunks.next().is_none());
                    tokens
                }
                // TODO
                Safe(a) => quote!(::std::fmt::Display::fmt(&(#a), _fmt)?;),
                Expr(a) => quote!(::yarte::Render::render(&(#a), _fmt)?;),
                Each(a) => self.gen_each(*a),
                IfElse(a) => self.gen_if_else(*a),
            })
        }
        tokens
    }
}
