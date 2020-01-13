use proc_macro2::TokenStream;
use syn::{
    parse2, punctuated::Punctuated, token::Comma, Attribute, Expr, Field, Ident, Token, Type,
    VisPublic, Visibility,
};

use yarte_config::Config;
use yarte_dom::dom::DOM;
use yarte_hir::{Struct, HIR};

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
    black_box: Vec<(Ident, Type)>,
}

fn is_state(f: &Field) -> bool {
    todo!()
}

impl<'a> WASMCodeGen<'a> {
    pub fn new<'n>(config: &'n Config<'n>, s: &'n Struct<'n>) -> WASMCodeGen<'n> {
        WASMCodeGen {
            config,
            build: TokenStream::new(),
            render: TokenStream::new(),
            hydrate: TokenStream::new(),
            helpers: TokenStream::new(),
            s,
            black_box: vec![],
        }
    }

    fn initial_state(&self) -> TokenStream {
        let attr: Attribute = parse2(quote!(#[serde(default)])).unwrap();
        let fields = self
            .s
            .fields
            .iter()
            .filter(|x| is_state(x))
            .map(|x| {
                let mut f = x.clone();
                f.attrs.push(attr.clone());
                f
            })
            .fold(Punctuated < Field, Comma > ::new(), |mut acc, x| {
                acc.push(x);
                acc
            });

        quote! {
            #[derive(Default, Deserialize)]
            struct InitialState {
                #fields
            }
        }
    }

    fn black_box(&self) -> TokenStream {
        let fields = self
            .black_box
            .iter()
            .map(|(ident, ty)| Field {
                attrs: vec![],
                vis: Visibility::Inherited,
                ident: ident.clone(),
                colon_token: Some(<Token![;]>::default()),
                ty: ty.clone(),
            })
            .fold(Punctuated < Field, Comma > ::new(), |mut acc, x| {
                acc.push(x);
                acc
            });

        quote! {
            struct BlackBox {
                #fields
            }
        }
    }
}

impl<'a> CodeGen for WASMCodeGen<'a> {
    fn gen(&mut self, ir: Vec<HIR>) -> TokenStream {
        let dom: DOM = ir.into();

        let initial_state = self.initial_state();
        let black_box = self.black_box();
        let default = self
            .s
            .implement_head(quote!(std::default::Default), self.build);
        let render = self.render;
        let hydrate = self.hydrate;
        let app = self.s.implement_head(
            quote!(yarte::Template),
            quote! {
            # [doc(hidden)]
            fn __render(& mut self, __addr: & Addr < Self> ) { # render }
            # [doc(hidden)]
            fn __hydrate(& mut self, __addr: & Addr < Self> ) { # hydrate }
            },
        );
        let helpers = self.helpers;

        quote! {
            # [wasm_bindgen]
            extern "C" {
            fn get_state() -> String;
            }

            # initial_state
            # black_box
            # default
            # app
            # helpers
        }
    }
}
