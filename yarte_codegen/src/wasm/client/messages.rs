use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    punctuated::Punctuated, visit_mut::VisitMut, Expr, Fields, FieldsNamed, FieldsUnnamed, Ident,
    ItemEnum, Path, Token, Variant,
};

fn gen_messages(e: &ItemEnum) -> (TokenStream, TokenStream) {
    let mut e = e.clone();
    let msgs = MsgBuilder::default().build(&mut e);
    let i = &e.ident;
    (
        quote! {
            #[inline]
            fn __dispatch(&mut self, __msg: Self::Message, __addr: &Addr<Self>) {
                use #i::*;
                match __msg {
                    #(#msgs), *
                }
            }
        },
        quote!(#e),
    )
}

struct Msg {
    ident: Ident,
    func: Path,
    fields: Fields,
}

impl ToTokens for Msg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Msg {
            ident,
            func,
            fields,
        } = self;
        let (args, pat) = fields_to_args(fields, ident);
        tokens.extend(if args.is_empty() {
            quote!(#pat => #func(self, __addr))
        } else {
            quote!(#pat => #func(self, #args, __addr))
        })
    }
}

fn fields_to_args(f: &Fields, i: &Ident) -> (Punctuated<Ident, Token![,]>, TokenStream) {
    let mut pun = Punctuated::new();
    match f {
        Fields::Named(FieldsNamed { named, .. }) => {
            let mut buff: Punctuated<Ident, Token![,]> = Punctuated::new();
            for i in named {
                let ident = i.ident.as_ref().unwrap();
                buff.push(ident.clone());
                pun.push(ident.clone());
            }

            (pun, quote!(#i { #buff }))
        }
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            let mut count = 0usize;
            let mut buff: Punctuated<Ident, Token![,]> = Punctuated::new();
            for _ in unnamed {
                let ident = format_ident!("_{}", count);
                count += 1;
                buff.push(ident.clone());
                pun.push(ident);
            }

            (pun, quote!(#i( #buff )))
        }
        Fields::Unit => (pun, quote!(#i)),
    }
}

#[derive(Default)]
struct MsgBuilder {
    paths: Vec<Msg>,
}

impl MsgBuilder {
    fn build(mut self, e: &mut ItemEnum) -> Vec<Msg> {
        self.visit_item_enum_mut(e);
        self.paths
    }
}

impl VisitMut for MsgBuilder {
    fn visit_variant_mut(
        &mut self,
        Variant {
            attrs,
            ident,
            fields,
            discriminant,
        }: &mut Variant,
    ) {
        assert_eq!(attrs.len(), 1);
        let attrs = attrs.remove(0);
        self.paths.push(Msg {
            func: attrs.path,
            fields: fields.clone(),
            ident: ident.clone(),
        });

        if discriminant.is_some() {
            panic!("No use enum discriminants in `msg` attribute")
        }
    }
}
