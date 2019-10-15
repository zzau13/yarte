use std::path::PathBuf;

use quote::quote;
use syn::visit::Visit;

use yarte_config::Config;

use super::EWrite;

pub(crate) fn visit_derive<'a>(i: &'a syn::DeriveInput, config: &Config) -> Struct<'a> {
    StructBuilder::default().build(i, config)
}

#[derive(Debug)]
pub(crate) struct Struct<'a> {
    pub src: String,
    pub path: PathBuf,
    pub print: Print,
    pub wrapped: bool,
    ident: &'a syn::Ident,
    generics: &'a syn::Generics,
}

impl<'a> Struct<'a> {
    pub fn implement_head(&self, t: &str, buf: &mut dyn EWrite) {
        let (impl_generics, orig_ty_generics, where_clause) = self.generics.split_for_impl();

        writeln!(
            buf,
            "{} {} for {}{} {{",
            quote!(impl#impl_generics),
            t,
            self.ident,
            quote!(#orig_ty_generics #where_clause)
        )
        .unwrap()
    }
}

struct StructBuilder {
    assured: Option<bool>,
    ext: Option<String>,
    path: Option<String>,
    print: Option<String>,
    src: Option<String>,
}

impl Default for StructBuilder {
    fn default() -> Self {
        StructBuilder {
            assured: None,
            ext: None,
            path: None,
            print: None,
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
        for i in attrs {
            self.visit_meta(&i.parse_meta().expect("valid meta attributes"));
        }

        self.visit_data(data);

        let (path, src) = match (self.src, self.ext) {
            (Some(src), ext) => (
                PathBuf::from(quote!(#ident).to_string())
                    .with_extension(ext.unwrap_or_else(|| DEFAULT_EXTENSION.to_owned())),
                src,
            ),
            (None, None) => config.get_template(&self.path.expect("some valid path")),
            (None, Some(_)) => panic!("'ext' attribute cannot be used with 'path' attribute"),
        };

        let wrapped = self.assured.unwrap_or_else(|| {
            if let Some(e) = path.extension() {
                if HTML_EXTENSIONS.contains(&e.to_str().unwrap()) {
                    return false;
                }
            }

            true
        });

        Struct {
            src,
            path,
            print: self.print.into(),
            wrapped,
            generics,
            ident,
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

    fn visit_meta_list(&mut self, syn::MetaList { path, nested, .. }: &'a syn::MetaList) {
        if path.is_ident("template") {
            use syn::punctuated::Punctuated;
            for el in Punctuated::pairs(nested) {
                let it = el.value();
                self.visit_nested_meta(it)
            }
        }
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
                panic!("attribute path must be string literal");
            }
        } else if path.is_ident("src") {
            if let syn::Lit::Str(ref s) = lit {
                if self.path.is_some() {
                    panic!("must specify 'src' or 'path', not both");
                }
                self.src = Some(s.value());
            } else {
                panic!("attribute src must be string literal");
            }
        } else if path.is_ident("print") {
            if let syn::Lit::Str(ref s) = lit {
                self.print = Some(s.value());
            } else {
                panic!("attribute print must be string literal");
            }
        } else if path.is_ident("assured") {
            if let syn::Lit::Bool(ref s) = lit {
                self.assured = Some(s.value);
            } else {
                panic!("attribute assured must be boolean literal");
            }
        } else if path.is_ident("ext") {
            if let syn::Lit::Str(ref s) = lit {
                self.ext = Some(s.value());
            } else {
                panic!("attribute ext must be string literal");
            }
        } else {
            panic!("invalid attribute '{:?}'", path.get_ident());
        }
    }
}

#[derive(PartialEq, Debug)]
pub(crate) enum Print {
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

static DEFAULT_EXTENSION: &str = "html";
static HTML_EXTENSIONS: [&str; 6] = [
    DEFAULT_EXTENSION,
    "htm",
    "xml",
    "hbs",
    "handlebars",
    "mustache",
];

#[cfg(test)]
mod test {
    use super::*;
    use syn::parse_str;

    #[test]
    #[should_panic]
    fn test_panic() {
        let src = r#"
            #[derive(Template)]
            #[template(path = "no-exist.html")]
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
            #[template(src = "", ext = "txt", print = "code")]
            struct Test;
        "#;
        let i = parse_str::<syn::DeriveInput>(src).unwrap();
        let config = Config::new("");
        let s = visit_derive(&i, &config);

        assert_eq!(s.src, "");
        assert_eq!(s.path, PathBuf::from("Test.txt"));
        assert_eq!(s.print, Print::Code);
        assert_eq!(s.wrapped, true);
    }
}
