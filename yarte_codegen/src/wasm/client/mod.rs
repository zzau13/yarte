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
    helpers: TokenStream,
    black_box: Vec<(Ident, Expr)>,
}

impl<'a> WASMCodeGen<'a> {
    pub fn new<'n>(config: &'n Config<'n>, s: &'n Struct<'n>) -> WASMCodeGen<'n> {
        WASMCodeGen { config, build: TokenStream::new(), render: TokenStream::new(), hydrate: TokenStream::new(), helpers: TokenStream::new(),s, black_box: vec![] }
    }
}

impl<'a> CodeGen for WASMCodeGen<'a> {
    fn gen(&mut self, ir: Vec<HIR>) -> TokenStream {
        let dom: DOM = ir.into();

        let initial_state = quote! {};
        let black_box = quote! {};
        let default = self.s.implement_head(quote!(std::default::Default), self.build);
        let render = self.render;
        let hydrate = self.hydrate;
        let app = self.s.implement_head(quote!(yarte::Template), quote! {
            #[doc(hidden)]
            fn __render(&mut self, __addr: &Addr<Self>) { #render }
            #[doc(hidden)]
            fn __hydrate(&mut self, __addr: &Addr<Self>) { #hydrate }
        });
        let helpers = self.helpers;

        quote! {
            #[wasm_bindgen]
            extern "C" {
                fn get_state() -> String;
            }

            #initial_state
            #black_box
            #default
            #app
            #helpers
        }
    }
}
