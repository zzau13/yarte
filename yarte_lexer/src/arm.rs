use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Pat, PatOr, Token};

#[derive(Debug, PartialEq, Clone)]
pub struct Arm {
    pat: syn::Pat,
    guard: Option<(syn::token::If, Box<syn::Expr>)>,
    fat_arrow_token: syn::token::FatArrow,
}

#[cfg(feature = "test")]
impl<'de> serde::Deserialize<'de> for Arm {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as serde::Deserializer<'de>>::Error>
    where
        D: serde::Deserializer<'de>,
    {
        <&str>::deserialize(deserializer)
            .and_then(|x| syn::parse_str(x).map_err(|_| serde::de::Error::custom("Parse error")))
    }
}

pub fn multi_pat_with_leading_vert(input: ParseStream) -> syn::Result<Pat> {
    let leading_vert: Option<Token![|]> = input.parse()?;
    multi_pat_impl(input, leading_vert)
}

fn multi_pat_impl(input: ParseStream, leading_vert: Option<Token![|]>) -> syn::Result<Pat> {
    let mut pat: Pat = input.parse()?;
    if leading_vert.is_some()
        || input.peek(Token![|]) && !input.peek(Token![||]) && !input.peek(Token![|=])
    {
        let mut cases = Punctuated::new();
        cases.push_value(pat);
        while input.peek(Token![|]) && !input.peek(Token![||]) && !input.peek(Token![|=]) {
            let punct = input.parse()?;
            cases.push_punct(punct);
            let pat: Pat = input.parse()?;
            cases.push_value(pat);
        }
        pat = Pat::Or(PatOr {
            attrs: Vec::new(),
            leading_vert,
            cases,
        });
    }
    Ok(pat)
}

impl Parse for Arm {
    fn parse(input: ParseStream) -> syn::Result<Arm> {
        Ok(Arm {
            pat: multi_pat_with_leading_vert(input)?,
            guard: {
                if input.peek(Token![if]) {
                    let if_token: Token![if] = input.parse()?;
                    let guard: syn::Expr = input.parse()?;
                    Some((if_token, Box::new(guard)))
                } else {
                    None
                }
            },
            fat_arrow_token: input.parse()?,
        })
    }
}
