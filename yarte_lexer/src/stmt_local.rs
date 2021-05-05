use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Semi;
use syn::{Expr, Local, Pat, PatOr, PatType, Result, Token, Type};

#[derive(Debug, PartialEq)]
pub struct StmtLocal {
    pub let_token: Token![let],
    pub pat: Pat,
    pub init: Option<(Token![=], Box<Expr>)>,
}

// Duplicated https://github.com/dtolnay/syn/blob/master/src/stmt.rs#L216
impl Parse for StmtLocal {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(StmtLocal {
            let_token: input.parse()?,
            pat: {
                let leading_vert: Option<Token![|]> = input.parse()?;
                let mut pat: Pat = input.parse()?;
                if leading_vert.is_some()
                    || input.peek(Token![|]) && !input.peek(Token![||]) && !input.peek(Token![|=])
                {
                    let mut cases = Punctuated::new();
                    cases.push_value(pat);
                    while input.peek(Token![|])
                        && !input.peek(Token![||])
                        && !input.peek(Token![|=])
                    {
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
                if input.peek(Token![:]) {
                    let colon_token: Token![:] = input.parse()?;
                    let ty: Type = input.parse()?;
                    pat = Pat::Type(PatType {
                        attrs: Vec::new(),
                        pat: Box::new(pat),
                        colon_token,
                        ty: Box::new(ty),
                    });
                }
                pat
            },
            init: {
                if input.peek(Token![=]) {
                    let eq_token: Token![=] = input.parse()?;
                    let init: Expr = input.parse()?;
                    Some((eq_token, Box::new(init)))
                } else {
                    None
                }
            },
        })
    }
}

impl ToTokens for StmtLocal {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let StmtLocal {
            let_token,
            pat,
            init,
        } = self;
        if let Some((eq, init)) = init {
            tokens.extend(quote!(#let_token #pat #eq #init));
        } else {
            tokens.extend(quote!(#let_token #pat));
        }
    }
}

// Use for no duplicate code in generator
impl From<StmtLocal> for Local {
    fn from(stmt: StmtLocal) -> Local {
        Local {
            let_token: stmt.let_token,
            attrs: Vec::new(),
            pat: stmt.pat,
            init: stmt.init,
            semi_token: Semi::default(),
        }
    }
}

impl From<Local> for StmtLocal {
    fn from(local: Local) -> Self {
        StmtLocal {
            let_token: local.let_token,
            pat: local.pat,
            init: local.init,
        }
    }
}

impl From<StmtLocal> for crate::Local {
    fn from(stmt: StmtLocal) -> crate::Local {
        crate::Local(stmt.into())
    }
}
