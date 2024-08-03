use std::{
    convert::{TryFrom, TryInto},
    path::{Path, PathBuf},
    rc::Rc,
};

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_str, visit::Visit, Data, Error, ItemEnum};

use yarte_helpers::config::Config;

// TODO:
const RECURSION_LIMIT: usize = 128;

pub fn visit_derive<'a>(
    i: &'a syn::DeriveInput,
    config: &Config,
) -> Result<(Struct<'a>, String), TokenStream> {
    StructBuilder::new(config).build(i)
}

#[derive(Debug)]
pub struct Struct<'a> {
    pub path: Rc<Path>,
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

        quote!(impl #impl_generics #t for #ident #orig_ty_generics #where_clause { #body })
    }
}

struct StructBuilder<'a> {
    fields: Vec<syn::Field>,
    path: Option<Rc<Path>>,
    print: Option<Print>,
    script: Option<String>,
    recursion_limit: Option<usize>,
    src: Option<String>,
    err: Vec<Error>,
    ident: String,
    config: &'a Config,
}

impl<'a> StructBuilder<'a> {
    fn new(config: &Config) -> StructBuilder {
        StructBuilder {
            config,
            ident: String::new(),
            fields: vec![],
            path: None,
            print: None,
            script: None,
            recursion_limit: None,
            src: None,
            err: vec![],
        }
    }

    fn build(mut self, i: &syn::DeriveInput) -> Result<(Struct, String), TokenStream> {
        let syn::DeriveInput {
            attrs,
            ident,
            generics,
            data,
            ..
        } = i;
        self.ident = ident.to_string();
        match data {
            Data::Struct(ref i) => {
                self.visit_data_struct(i);
            }
            Data::Enum(_) | Data::Union(_) => {
                self.err.push(Error::new_spanned(i, "need a `struc`"))
            }
        }
        let mut msgs = None;
        for i in attrs {
            if i.path.is_ident("template") {
                match i.parse_meta() {
                    Ok(ref m) => self.visit_meta(m),
                    Err(e) => {
                        self.err.push(e);
                        continue;
                    }
                };
            } else if i.path.is_ident("msg") {
                let tokens = i.tokens.to_string();
                // TODO:
                let tokens = tokens.get(1..tokens.len() - 1).expect("some");
                let enu: ItemEnum = parse_str(tokens).expect("valid enum");
                msgs = Some(enu);
            }
        }

        let (path, src) = match (self.path, self.src) {
            (Some(path), Some(src)) => (path, src),
            _ => {
                self.err.push(Error::new_spanned(
                    attrs.iter().find(|x| x.path.is_ident("template")).unwrap(),
                    "must specify 'src' or 'path'",
                ));
                (PathBuf::new().into(), String::new())
            }
        };

        if self.err.is_empty() {
            Ok((
                Struct {
                    path,
                    recursion_limit: self.recursion_limit.unwrap_or(RECURSION_LIMIT),
                    fields: self.fields,
                    generics,
                    ident,
                    msgs,
                    print: self.print.unwrap_or(Print::None),
                    script: self.script,
                },
                src,
            ))
        } else {
            Err(self.err.iter().flat_map(Error::to_compile_error).collect())
        }
    }
}

impl<'a, 'b> Visit<'a> for StructBuilder<'b> {
    fn visit_field(&mut self, e: &'a syn::Field) {
        self.fields.push(e.clone());
    }

    fn visit_meta_name_value(&mut self, i: &'a syn::MetaNameValue) {
        let syn::MetaNameValue { path, lit, .. } = i;
        if path.is_ident("path") {
            if let syn::Lit::Str(ref s) = lit {
                if self.src.is_some() {
                    self.err.push(Error::new_spanned(
                        i,
                        "must specify 'src' or 'path', not both",
                    ))
                }
                let mut path = PathBuf::from(s.value());
                if let Some(ext) = path.extension() {
                    if ext != DEFAULT_EXTENSION {
                        self.err.push(Error::new_spanned(
                            i,
                            "Default extension for yarte templates is `.hbs`",
                        ))
                    }
                } else {
                    path = path.with_extension(DEFAULT_EXTENSION);
                };
                let (path, src) = self.config.get_template(path.into());
                self.path = Some(path);
                self.src = Some(src);
            } else {
                self.err.push(Error::new_spanned(
                    i,
                    "attribute 'path' must be string literal",
                ))
            }
        } else if path.is_ident("src") {
            if let syn::Lit::Str(ref s) = lit {
                if self.path.is_some() {
                    self.err.push(Error::new_spanned(
                        i,
                        "must specify 'src' or 'path', not both",
                    ));
                }
                self.path = Some(
                    self.config
                        .get_dir()
                        .join(PathBuf::from(self.ident.clone()))
                        .with_extension(DEFAULT_EXTENSION)
                        .into(),
                );
                self.src = Some(s.value().trim_end().to_owned());
            } else {
                self.err.push(Error::new_spanned(
                    i,
                    "attribute 'src' must be string literal",
                ));
            }
        } else if path.is_ident("print") {
            if let syn::Lit::Str(ref s) = lit {
                match s.value().try_into() {
                    Ok(s) => self.print = Some(s),
                    Err(e) => {
                        self.err.push(Error::new_spanned(i, e));
                    }
                }
            } else {
                self.err.push(Error::new_spanned(
                    i,
                    "attribute 'print' must be string literal",
                ));
            }
        } else if path.is_ident("script") {
            if let syn::Lit::Str(ref s) = lit {
                self.script = Some(s.value());
            } else {
                self.err.push(Error::new_spanned(
                    i,
                    "attribute 'script' must be string literal",
                ));
            }
        } else if path.is_ident("recursion") {
            if let syn::Lit::Int(s) = lit {
                self.recursion_limit = Some(s.base10_parse().unwrap());
            } else {
                self.err.push(Error::new_spanned(
                    i,
                    "attribute 'recursion-limit' must be number literal",
                ));
            }
        } else {
            self.err.push(Error::new_spanned(
                i,
                format!("invalid attribute '{}'", path.get_ident().unwrap()),
            ));
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Print {
    All,
    Ast,
    Code,
    None,
}

impl TryFrom<String> for Print {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_ref() {
            "all" => Ok(Print::All),
            "ast" => Ok(Print::Ast),
            "code" => Ok(Print::Code),
            v => Err(format!("invalid value for print attribute: {v}")),
        }
    }
}

static DEFAULT_EXTENSION: &str = "hbs";
