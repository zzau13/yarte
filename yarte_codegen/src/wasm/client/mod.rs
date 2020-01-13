use proc_macro2::TokenStream;
use syn::{Ident, Expr};

use yarte_dom::dom::DOM;
use yarte_hir::{Struct, HIR};
use yarte_config::Config;

use crate::CodeGen;

mod each;
mod if_else;

pub struct WASMCodeGen<'a> {
    s: &'a Struct<'a>,
    config: &'a Config<'a>,
    build: TokenStream,
    render: TokenStream,
    hydrate: TokenStream,
    struct_: Vec<(Ident, Expr)>,
}

impl<'a> WASMCodeGen<'a> {
    pub fn new<'n>(config: &'n Config<'n>, s: &'n Struct<'n>) -> WASMCodeGen<'n> {
        WASMCodeGen { config, build: TokenStream::new(), render: TokenStream::new(), hydrate: TokenStream::new(), s, struct_: vec![] }
    }
}

impl<'a> CodeGen for WASMCodeGen<'a> {
    fn gen(&self, ir: Vec<HIR>) -> TokenStream {
        let dom: DOM = ir.into();

        let default = self.s.implement_head(quote!(Default), quote!());
        let app = self.s.implement_head(quote!(yarte::Template), quote!());
        let initial_state = quote! {};
        let black_box = quote! {};
        let get_state_fn = quote! {
            #[wasm_bindgen]
            extern "C" {
                fn get_state() -> String;
            }
        };
        todo!()
    }
}
