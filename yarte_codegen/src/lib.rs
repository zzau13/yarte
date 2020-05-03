#![allow(clippy::match_on_vec_items)]
extern crate proc_macro2_impersonated as proc_macro2;
extern crate quote_impersonated as quote;
extern crate syn_impersonated as syn;

use proc_macro2::TokenStream;
use quote::quote;

use yarte_hir::{Each, IfElse, HIR};

#[macro_use]
mod macros;
mod fmt;
mod fn_fmt;
mod html;
mod text;
pub mod wasm;

#[cfg(feature = "html-min")]
pub use self::html::html_min::HTMLMinCodeGen;
pub use self::{fmt::FmtCodeGen, fn_fmt::FnFmtCodeGen, html::HTMLCodeGen, text::TextCodeGen};
pub use wasm::*;

pub trait CodeGen {
    fn gen(&mut self, v: Vec<HIR>) -> TokenStream;
}

pub trait EachCodeGen: CodeGen {
    fn gen_each(&mut self, Each { args, body, expr }: Each) -> TokenStream {
        let body = self.gen(body);
        quote!(for #expr in #args { #body })
    }
}

pub trait IfElseCodeGen: CodeGen {
    fn gen_if_else(&mut self, IfElse { ifs, if_else, els }: IfElse) -> TokenStream {
        let mut tokens = TokenStream::new();

        let (args, body) = ifs;
        let body = self.gen(body);
        tokens.extend(quote!(if #args { #body }));

        for (args, body) in if_else {
            let body = self.gen(body);
            tokens.extend(quote!(else if #args { #body }));
        }

        if let Some(body) = els {
            let body = self.gen(body);
            tokens.extend(quote!(else { #body }));
        }

        tokens
    }
}
