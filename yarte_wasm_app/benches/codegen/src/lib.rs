use proc_macro::TokenStream;

use std::slice::Iter;
use syn::{
    parse,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Expr, Token,
};

use quote::quote;

struct ExprList {
    list: Punctuated<Expr, Token![,]>,
}

impl Parse for ExprList {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ExprList {
            list: input.parse_terminated(Expr::parse)?,
        })
    }
}

impl Into<Vec<Vec<String>>> for ExprList {
    fn into(self) -> Vec<Vec<String>> {
        use syn::Expr::*;
        self.list
            .into_pairs()
            .map(|p| p.into_value())
            .map(|x| -> Vec<String> {
                match x {
                    Array(a) => a
                        .elems
                        .into_pairs()
                        .map(|p| p.into_value())
                        .map(|x| match x {
                            Lit(a) => match a.lit {
                                syn::Lit::Str(a) => a.value(),
                                _ => panic!("Array items isn't string"),
                            },
                            _ => panic!("Array items isn't string"),
                        })
                        .collect(),
                    _ => panic!("Input is coma separated arrays of string `[...], [...],...`"),
                }
            })
            .collect()
    }
}

#[proc_macro]
pub fn zip_with_spaces(token: TokenStream) -> TokenStream {
    let expr: ExprList = parse(token).unwrap();
    let expr: Vec<Vec<String>> = expr.into();
    let mut expr = expr.iter();
    let first = expr.next().expect("Need minimum two element");
    let mut buff = vec![];
    for i in first {
        _zip(i, &mut expr.clone(), &mut buff)
    }
    let len = buff.len();

    quote!(static ZIPPED: [&str; #len] = [#(#buff),*];).into()
}

fn _zip(head: &str, expr: &mut Iter<Vec<String>>, buff: &mut Vec<String>) {
    if let Some(first) = expr.next() {
        for i in first {
            let head = head.to_owned() + " " + &i;
            _zip(&head, &mut expr.clone(), buff)
        }
    } else {
        buff.push(head.into());
    }
}
