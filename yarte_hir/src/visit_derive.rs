use std::path::PathBuf;

use quote::quote;
use syn::visit::Visit;

use yarte_helpers::config::Config;

use proc_macro2::TokenStream;
use syn::{parse_str, ItemEnum};

const RECURSION_LIMIT: usize = 2048;

pub fn visit_derive<'a>(i: &'a syn::DeriveInput, config: &Config) -> Struct<'a> {
    StructBuilder::default().build(i, config)
}

#[derive(Debug)]
pub struct Struct<'a> {
    pub src: String,
    pub path: PathBuf,
    pub print: Print,
    pub recursion_limit: usize,
    pub msgs: Option<ItemEnum>,
    pub script: Option<String>,
    pub fields: Vec<syn::Field>,
    pub ident: &'a syn::Ident,
    generics: &'a syn::Generics,
}

impl<'a> Struct<'a> {
    pub fn implement_head(&self, t: TokenStream, body: &TokenStream) -> TokenStream {
        let Struct {
            ident, generics, ..
        } = *self;
        let (impl_generics, orig_ty_generics, where_clause) = generics.split_for_impl();

        quote!(impl#impl_generics #t for #ident #orig_ty_generics #where_clause { #body })
    }
}

struct StructBuilder {
    fields: Vec<syn::Field>,
    path: Option<String>,
    print: Option<String>,
    script: Option<String>,
    recursion_limit: Option<usize>,
    src: Option<String>,
}

impl Default for StructBuilder {
    fn default() -> Self {
        StructBuilder {
            fields: vec![],
            path: None,
            print: None,
            script: None,
            recursion_limit: None,
            src: None,
        }
    }
}

impl StructBuilder {
    fn build<'n>(
        mut self,
        syn::DeriveInput {
            attrs,
            ident,
            generics,
            data,
            ..
        }: &'n syn::DeriveInput,
        config: &Config,
    ) -> Struct<'n> {
        let mut msgs = None;
        for i in attrs {
            if i.path.is_ident("template") {
                self.visit_meta(&i.parse_meta().expect("valid meta attributes"));
            } else if i.path.is_ident("msg") {
                let tokens = i.tokens.to_string();
                let tokens = tokens.get(1..tokens.len() - 1).expect("some");
                let enu: ItemEnum = parse_str(tokens).expect("valid enum");
                msgs = Some(enu);
            }
        }

        self.visit_data(data);

        let (path, src) = if let Some(src) = self.src {
            (
                config.get_dir().join(
                    PathBuf::from(quote!(#ident).to_string()).with_extension(DEFAULT_EXTENSION),
                ),
                src.trim_end().to_owned(),
            )
        } else {
            let path = PathBuf::from(self.path.expect("some valid path"));
            let path = if let Some(ext) = path.extension() {
                if ext == DEFAULT_EXTENSION {
                    path
                } else {
                    panic!("Default extension for yarte templates is `.hbs`")
                }
            } else {
                path.with_extension(DEFAULT_EXTENSION)
            };
            config.get_template(&path)
        };

        Struct {
            recursion_limit: self.recursion_limit.unwrap_or(RECURSION_LIMIT),
            fields: self.fields,
            generics,
            ident,
            msgs,
            path,
            print: self.print.into(),
            script: self.script,
            src,
        }
    }
}

impl<'a> Visit<'a> for StructBuilder {
    fn visit_data(&mut self, i: &'a syn::Data) {
        use syn::Data::*;
        match i {
            Struct(ref i) => {
                self.visit_data_struct(i);
            }
            Enum(_) | Union(_) => panic!("Not valid need a `struc`"),
        }
    }

    fn visit_field(&mut self, e: &'a syn::Field) {
        self.fields.push(e.clone());
    }

    fn visit_meta_name_value(
        &mut self,
        syn::MetaNameValue { path, lit, .. }: &'a syn::MetaNameValue,
    ) {
        if path.is_ident("path") {
            if let syn::Lit::Str(ref s) = lit {
                if self.src.is_some() {
                    panic!("must specify 'src' or 'path', not both");
                }
                self.path = Some(s.value());
            } else {
                panic!("attribute 'path' must be string literal");
            }
        } else if path.is_ident("src") {
            if let syn::Lit::Str(ref s) = lit {
                if self.path.is_some() {
                    panic!("must specify 'src' or 'path', not both");
                }
                self.src = Some(s.value());
            } else {
                panic!("attribute 'src' must be string literal");
            }
        } else if path.is_ident("print") {
            if let syn::Lit::Str(ref s) = lit {
                self.print = Some(s.value());
            } else {
                panic!("attribute 'print' must be string literal");
            }
        } else if path.is_ident("script") {
            if let syn::Lit::Str(ref s) = lit {
                self.script = Some(s.value());
            } else {
                panic!("attribute 'script' must be string literal");
            }
        } else if path.is_ident("recursion-limit") {
            if let syn::Lit::Int(s) = lit {
                self.recursion_limit = Some(s.base10_parse().unwrap());
            } else {
                panic!("attribute 'script' must be string literal");
            }
        } else {
            panic!("invalid attribute '{:?}'", path.get_ident());
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Print {
    All,
    Ast,
    Code,
    None,
}

impl From<Option<String>> for Print {
    fn from(s: Option<String>) -> Print {
        match s {
            Some(s) => match s.as_ref() {
                "all" => Print::All,
                "ast" => Print::Ast,
                "code" => Print::Code,
                v => panic!("invalid value for print attribute: {}", v),
            },
            None => Print::None,
        }
    }
}

static DEFAULT_EXTENSION: &str = "hbs";

#[cfg(test)]
mod test {
    use super::*;
    use syn::parse_str;

    #[test]
    #[should_panic]
    fn test_panic() {
        let src = r#"
            #[derive(Template)]
            #[template(path = "no-exist")]
            struct Test;
        "#;
        let i = parse_str::<syn::DeriveInput>(src).unwrap();
        let config = Config::new("");
        let _ = visit_derive(&i, &config);
    }

    #[test]
    fn test() {
        let src = r#"
            #[derive(Template)]
            #[template(src = "", print = "code")]
            struct Test;
        "#;
        let i = parse_str::<syn::DeriveInput>(src).unwrap();
        let config = Config::new("");
        let s = visit_derive(&i, &config);

        assert_eq!(s.src, "");
        assert_eq!(s.path, config.get_dir().join(PathBuf::from("Test.hbs")));
        assert_eq!(s.print, Print::Code);
    }
}
