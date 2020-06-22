#![allow(clippy::unknown_clippy_lints, clippy::match_on_vec_items)]
use proc_macro2::TokenStream;
use quote::quote;

use yarte_hir::{Each, IfElse, HIR};

#[macro_use]
mod macros;
#[cfg(feature = "bytes-buf")]
mod bytes;
#[cfg(feature = "fixed")]
mod fixed;
mod fmt;
mod fn_fmt;
mod html;
mod text;
pub mod wasm;

pub use self::{fmt::FmtCodeGen, fn_fmt::FnFmtCodeGen, html::HTMLCodeGen, text::TextCodeGen};

#[cfg(any(feature = "wasm-app", feature = "wasm-server"))]
pub use wasm::*;

#[cfg(all(feature = "bytes-buf", feature = "html-min"))]
pub use self::bytes::html_min::HTMLMinBytesCodeGen;
#[cfg(feature = "bytes-buf")]
pub use self::bytes::{BytesCodeGen, HTMLBytesCodeGen, TextBytesCodeGen};
#[cfg(all(feature = "fixed", feature = "html-min"))]
pub use self::fixed::html_min::HTMLMinFixedCodeGen;
#[cfg(feature = "fixed")]
pub use self::fixed::{FixedCodeGen, HTMLFixedCodeGen, TextFixedCodeGen};
#[cfg(feature = "html-min")]
pub use self::html::html_min::HTMLMinCodeGen;

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
