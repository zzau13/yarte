use proc_macro2::{Span, TokenStream};
use quote::quote;

#[proc_macro]
pub fn asciis(t: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match syn::parse::<syn::LitStr>(t) {
        Ok(s) => _asciis(s.value()).expect("only ascii valid").into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn unsafe_asciis(t: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match syn::parse::<syn::LitStr>(t) {
        Ok(s) => _unsafe_asciis(s.value()).expect("only ascii valid").into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[inline]
fn _asciis(s: String) -> Option<TokenStream> {
    let mut tokens = Vec::with_capacity(s.len());
    for c in s.chars() {
        if c.len_utf8() == 1 {
            let c = format!("{:?}", c);
            let c: syn::LitChar = syn::parse_str(&c).ok()?;
            tokens.push(c);
        } else {
            return None;
        }
    }

    Some(quote!(&[#(yarte_lexer::ascii!(#tokens)),*]))
}

#[inline]
fn _unsafe_asciis(s: String) -> Option<TokenStream> {
    let mut tokens = Vec::with_capacity(s.len());
    for c in s.chars() {
        if c.len_utf8() == 1 {
            tokens.push(syn::LitByte::new(c as u8, Span::call_site()));
        } else {
            return None;
        }
    }

    Some(quote!(&[#(Ascii::new(#tokens)),*]))
}
