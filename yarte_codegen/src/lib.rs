#![allow(unknown_lints, clippy::match_on_vec_items)]
use proc_macro2::TokenStream;
use quote::quote;

use yarte_hir::{Each, IfElse, HIR};

#[cfg(feature = "bytes-buf")]
mod attr_b;
#[cfg(feature = "bytes-buf")]
mod bytes;
mod fmt;
mod fn_fmt;
mod html;
mod text;
#[cfg(feature = "bytes-buf")]
mod write_b;

pub use self::{fmt::FmtCodeGen, fn_fmt::FnFmtCodeGen, html::HTMLCodeGen, text::TextCodeGen};

#[cfg(feature = "bytes-buf")]
pub use self::attr_b::AttrBCodeGen;
#[cfg(feature = "bytes-buf")]
pub use self::bytes::{BytesCodeGen, HTMLBytesCodeGen, TextBytesCodeGen};
#[cfg(feature = "bytes-buf")]
pub use self::write_b::WriteBCodeGen;

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
