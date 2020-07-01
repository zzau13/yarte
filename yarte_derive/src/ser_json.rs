// Adapted from [`simd-json-derive`](https://github.com/simd-lite/simd-json-derive)

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Data, DataStruct, DeriveInput, Fields, FieldsNamed, FieldsUnnamed, Variant};
use v_jsonescape::escape;

struct StrT(String);

impl ToTokens for StrT {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let this = &self.0;
        let t = match this.len() {
            0 => unreachable!(),
            len @ 1..=3 => {
                let range: TokenStream = this
                    .as_bytes()
                    .iter()
                    .enumerate()
                    .map(|(i, x)| quote!(unsafe { *yarte::buf_ptr(buf).add(#i) = #x; }))
                    .flatten()
                    .collect();
                quote! {
                    buf.reserve(#len);
                    #range
                    unsafe { yarte::BufMut::advance_mut(buf, #len); }
                }
            }
            _ => quote!(buf.extend_from_slice(#this.as_bytes());),
        };
        tokens.extend(t)
    }
}

pub(crate) fn serialize_json(i: syn::DeriveInput) -> TokenStream {
    match i {
        // Unnamed struct
        DeriveInput {
            ident,
            data:
                Data::Struct(DataStruct {
                    fields: Fields::Unnamed(FieldsUnnamed { unnamed, .. }),
                    ..
                }),
            generics,
            ..
        } => {
            if unnamed.len() == 1 {
                quote! {
                    impl #generics yarte::Serialize for #ident #generics {
                        fn to_bytes_mut(&self, buf: &mut yarte::BytesMut) {
                            self.0.to_bytes_mut(buf)
                        }
                    }
                }
            } else {
                let keys: Vec<_> = unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, _)| syn::Index::from(i))
                    .skip(1)
                    .collect();
                quote! {
                    impl #generics yarte::Serialize for #ident #generics {
                        fn to_bytes_mut(&self, buf: &mut yarte::BytesMut) {
                            yarte::begin_array(buf);
                            (&self.0).to_bytes_mut(buf);
                            #(
                                yarte::write_comma(buf);
                                (&self.#keys).to_bytes_mut(buf);
                            )*
                            yarte::end_array(buf);
                        }
                    }
                }
            }
        }
        DeriveInput {
            ident,
            data:
                Data::Struct(DataStruct {
                    fields: Fields::Named(FieldsNamed { named, .. }),
                    ..
                }),
            generics,
            ..
        } => {
            let (mut keys, values): (Vec<_>, Vec<_>) = named
                .iter()
                .filter_map(|f| {
                    f.ident
                        .clone()
                        .map(|ident| (format!("\"{}\":", escape(&ident.to_string())), ident))
                })
                .unzip();

            if let Some((first, rest)) = keys.split_first_mut() {
                *first = format!("{{{}", first);
                for r in rest {
                    *r = format!(",{}", r);
                }
            };

            let keys = keys.into_iter().map(StrT);
            quote! {
                impl #generics yarte::Serialize for #ident #generics {
                    #[inline]
                    fn to_bytes_mut(&self, buf: &mut yarte::BytesMut) {
                        #(
                            #keys
                            self.#values.to_bytes_mut(buf);
                        )*
                        yarte::end_object(buf);
                    }
                }
            }
        }
        DeriveInput {
            ident,
            data: Data::Enum(d),
            generics,
            ..
        } => {
            let mut body_elements = Vec::new();
            let (simple, variants): (Vec<_>, Vec<_>) =
                d.variants.into_iter().partition(|v| v.fields.is_empty());
            let (named, unnamed): (Vec<_>, Vec<_>) = variants.iter().partition(|v| {
                if let Variant {
                    fields: Fields::Named(_),
                    ..
                } = v
                {
                    true
                } else {
                    false
                }
            });

            let (unnamed1, unnamed): (Vec<_>, Vec<_>) =
                unnamed.into_iter().partition(|v| v.fields.len() == 1);

            // enum no fields of Enum::Variant
            // They serialize as: "Variant"
            let (simple_keys, simple_values): (Vec<_>, Vec<_>) = simple
                .iter()
                .map(|s| {
                    (
                        &s.ident,
                        StrT(format!("\"{}\"", escape(&s.ident.to_string()))),
                    )
                })
                .unzip();
            let simple = quote! {
                #(
                    #ident::#simple_keys => { #simple_values }
                ),*
            };

            if !simple.is_empty() {
                body_elements.push(simple);
            }

            // Unnamed enum variants with exactly 1 field of Enum::Variant(type1)
            // They serialize as: {"Variant":..}
            let (unnamed1_idents, unnamed1_keys): (Vec<_>, Vec<_>) = unnamed1
                .iter()
                .map(|v| {
                    (
                        &v.ident,
                        StrT(format!("{{\"{}\":", escape(&v.ident.to_string()))),
                    )
                })
                .unzip();
            let unnamed1 = quote! {
                #(
                    #ident::#unnamed1_idents(v) => {
                        #unnamed1_keys
                        v.to_bytes_mut(buf);
                        yarte::end_object(buf);
                    }
                ),*
            };
            if !unnamed1.is_empty() {
                body_elements.push(unnamed1);
            }

            // Unnamed enum variants with more then 1 field of Enum::Variant(type1, type2, type3)
            // They serialize as: {"Variant":[.., .., ..]}
            let (unnamed_ident_and_vars, unnamed_keys): (Vec<_>, Vec<_>) = unnamed
                .iter()
                .map(|v| {
                    (
                        (
                            &v.ident,
                            (0..v.fields.len())
                                .map(|i| Ident::new(&format!("v{}", i), Span::call_site()))
                                .collect::<Vec<_>>(),
                        ),
                        StrT(format!("{{\"{}\":[", escape(&v.ident.to_string()))),
                    )
                })
                .unzip();

            let (unnamed_idents, unnamed_var_names): (Vec<_>, Vec<_>) =
                unnamed_ident_and_vars.into_iter().unzip();

            let unnamed_vecs = unnamed_var_names.iter().map(|vs| {
                let (first, rest) = vs.split_first().unwrap();
                quote! {
                    #first.to_bytes_mut(buf);
                    #(
                        yarte::write_comma(buf);
                        #rest.to_bytes_mut(buf);
                    )*
                }
            });

            let unnamed_vars = unnamed_var_names.iter().map(|vs| quote! { #(#vs),* });

            let unnamed = quote! {
                #(
                    #ident::#unnamed_idents(#unnamed_vars) =>
                    {
                        #unnamed_keys
                        #unnamed_vecs
                        yarte::end_array_object(buf);
                    }
                ),*
            };
            if !unnamed.is_empty() {
                body_elements.push(unnamed);
            }

            // Named enum variants of the form Enum::Variant{key1: type, key2: type...}
            // They serialize as: {"Variant":{"key1":..,"key2":..}}
            let mut named_bodies = Vec::new();
            for v in named {
                let named_ident = &v.ident;
                let fields: Vec<_> = v.fields.iter().cloned().map(|f| f.ident.unwrap()).collect();
                let (first, rest) = fields.split_first().unwrap();

                let start = StrT(format!(
                    "{{\"{}\":{{\"{}\":",
                    escape(&v.ident.to_string()),
                    escape(&first.to_string())
                ));

                let rest_keys = rest
                    .iter()
                    .map(|f| StrT(format!(",\"{}\":", escape(&f.to_string()))));

                named_bodies.push(quote! {
                    #ident::#named_ident{#(#fields),*} => {
                        #start
                        #first.to_bytes_mut(buf);
                        #(
                            #rest_keys
                            #rest.to_bytes_mut(buf);
                        )*
                        yarte::end_object_object(buf);
                    }
                });
            }
            let named = quote! {#(#named_bodies),*};

            if !named.is_empty() {
                body_elements.push(named);
            }

            let match_body = quote! {
                #(#body_elements),*
            };

            quote! {
                impl #generics yarte::Serialize for #ident #generics {
                    #[inline]
                    fn to_bytes_mut(&self, buf: &mut yarte::BytesMut) {
                        match self {
                            #match_body
                        }
                    }
                }
            }
        }
        _ => quote! {},
    }
}
