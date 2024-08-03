#![allow(unused_imports, dead_code)]
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    iter,
    path::PathBuf,
    rc::Rc,
};

use proc_macro::TokenStream;
use quote::{format_ident, quote};

use syn::parse::{ParseBuffer, ParseStream};
use syn::spanned::Spanned;

use yarte_codegen::{CodeGen, FmtCodeGen, HTMLCodeGen, TextCodeGen};
use yarte_helpers::{
    config::{get_source, read_config_file, Config, PrintConfig},
    logger::log,
};
use yarte_hir::{generate, resolve_imports, visit_derive, HIROptions, Print, Struct};
use yarte_parser::{emitter, parse, source_map, OwnParsed, Partial};

#[cfg(feature = "json")]
mod ser_json;

macro_rules! build {
    ($i:ident, $codegen:ident, $opt:expr) => {{
        let config_toml: &str = &read_config_file();
        let config = &Config::new(config_toml);
        let (struct_, source) = match visit_derive($i, config) {
            Ok(s) => s,
            Err(ts) => return ts.into(),
        };
        // TODO: remove
        proc_macro2::fallback::force();
        sources_to_tokens(source, config, &struct_, $codegen(&struct_), $opt)
    }};
}

#[proc_macro_derive(TemplateText, attributes(template))]
/// Implements TemplateTrait without html escape functionality
pub fn template(input: TokenStream) -> TokenStream {
    fn get_codegen<'a>(s: &'a Struct) -> Box<dyn CodeGen + 'a> {
        Box::new(FmtCodeGen::new(TextCodeGen, s, "yarte"))
    }

    let i = &syn::parse(input).unwrap();
    build!(
        i,
        get_codegen,
        HIROptions {
            is_text: true,
            ..Default::default()
        }
    )
    .into()
}

#[proc_macro_derive(Template, attributes(template))]
/// Implements TemplateTrait with html escape functionality
pub fn template_html(input: TokenStream) -> TokenStream {
    fn get_codegen<'a>(s: &'a Struct) -> Box<dyn CodeGen + 'a> {
        Box::new(FmtCodeGen::new(HTMLCodeGen, s, "yarte"))
    }
    let i = &syn::parse(input).unwrap();
    build!(i, get_codegen, Default::default()).into()
}

#[proc_macro_derive(TemplateBytesText, attributes(template))]
#[cfg(feature = "bytes-buf")]
/// Implements TemplateBytesTrait without html escape functionality
pub fn template_bytes(input: TokenStream) -> TokenStream {
    const PARENT: &str = "yarte";

    let buf_i = format_ident!("bytes_mut");
    let buf: syn::Expr = syn::parse2(quote!(#buf_i)).unwrap();
    let get_codegen = |s| {
        Box::new(yarte_codegen::BytesCodeGen::new(
            yarte_codegen::TextBytesCodeGen::new(&buf),
            s,
            buf_i,
            PARENT,
        ))
    };

    let i = &syn::parse(input).unwrap();
    build!(
        i,
        get_codegen,
        HIROptions {
            is_text: true,
            ..Default::default()
        }
    )
    .into()
}

#[proc_macro_derive(TemplateBytes, attributes(template))]
#[cfg(feature = "bytes-buf")]
/// Implements TemplateBytesTrait with html escape functionality
pub fn template_html_bytes(input: TokenStream) -> TokenStream {
    const PARENT: &str = "yarte";

    let buf_i = format_ident!("bytes_mut");
    let buf: syn::Expr = syn::parse2(quote!(#buf_i)).unwrap();
    let get_codegen = |s| {
        Box::new(yarte_codegen::BytesCodeGen::new(
            yarte_codegen::HTMLBytesCodeGen::new(&buf),
            s,
            buf_i,
            PARENT,
        ))
    };
    let i = &syn::parse(input).unwrap();
    build!(i, get_codegen, Default::default()).into()
}

#[proc_macro_derive(Serialize)]
#[cfg(feature = "json")]
pub fn serialize_json(i: TokenStream) -> TokenStream {
    let i = syn::parse(i).unwrap();
    let tokens = ser_json::serialize_json(i);
    tokens.into()
}

#[proc_macro]
/// Format handlebars string in this scope with html escape functionality
pub fn yformat_html(i: TokenStream) -> TokenStream {
    const PARENT: &str = "yarte";
    fn get_codegen<'a>(_s: &'a Struct<'a>) -> Box<dyn CodeGen + 'a> {
        Box::new(yarte_codegen::FnFmtCodeGen::new(HTMLCodeGen, PARENT))
    }

    let src: syn::LitStr = syn::parse(i).unwrap();
    let input = quote! {
        #[template(src = #src)]
        struct __Main__;
    };

    let i = &syn::parse2(input).unwrap();
    build!(
        i,
        get_codegen,
        HIROptions {
            resolve_to_self: false,
            parent: PARENT,
            ..Default::default()
        }
    )
    .into()
}

#[proc_macro]
/// Format handlebars string in this scope without html escape functionality
pub fn yformat(i: TokenStream) -> TokenStream {
    const PARENT: &str = "yarte";
    fn get_codegen<'a>(_s: &'a Struct<'a>) -> Box<dyn CodeGen + 'a> {
        Box::new(yarte_codegen::FnFmtCodeGen::new(TextCodeGen, PARENT))
    }

    let src: syn::LitStr = syn::parse(i).unwrap();
    let input = quote! {
        #[template(src = #src)]
        struct __Main__;
    };

    let i = &syn::parse2(input).unwrap();
    build!(
        i,
        get_codegen,
        HIROptions {
            resolve_to_self: false,
            is_text: true,
            parent: PARENT,
        }
    )
    .into()
}

struct AutoArg {
    path: syn::Ident,
    _a: syn::Token![!],
    _b: syn::token::Paren,
    ty: syn::Type,
    _c: syn::token::Comma,
    lit: syn::LitStr,
}

#[allow(clippy::mixed_read_write_in_expression)]
impl syn::parse::Parse for AutoArg {
    fn parse(input: &ParseBuffer) -> syn::Result<Self> {
        let content;
        Ok(AutoArg {
            path: input.parse()?,
            _a: input.parse()?,
            _b: syn::parenthesized!(content in input),
            ty: content.parse()?,
            _c: content.parse()?,
            lit: content.parse()?,
        })
    }
}

#[proc_macro]
pub fn auto(i: TokenStream) -> TokenStream {
    let AutoArg { path, ty, lit, .. } = match syn::parse(i) {
        Ok(arg) => arg,
        Err(e) => return e.to_compile_error().into(),
    };

    let token = quote! {{
        thread_local! {
            static SIZE: std::cell::Cell<usize> = std::cell::Cell::new(0);
        }
        let mut __buf: #ty = yarte::Buffer::with_capacity(SIZE.with(|v| v.get()));

        #path!(__buf, #lit);

        SIZE.with(|v| if v.get() < __buf.len() {
            v.set(__buf.len())
        });

        __buf
    }};

    token.into()
}

#[proc_macro]
#[cfg(feature = "bytes-buf")]
/// Write handlebars template to `buf-min::Buffer` in this scope without html escape functionality
pub fn ywrite(i: TokenStream) -> TokenStream {
    const PARENT: &str = "yarte";

    let WriteArg { buf, src, .. } = match syn::parse(i) {
        Ok(arg) => arg,
        Err(e) => return e.to_compile_error().into(),
    };
    // Check correct syn::Expr
    {
        use syn::Expr::*;
        match &buf {
            Field(_) | Path(_) => (),
            _ => {
                return syn::Error::new(buf.span(), "first argument should be a ident or field")
                    .to_compile_error()
                    .into()
            }
        }
    }

    let get_codegen = |_| {
        Box::new(yarte_codegen::WriteBCodeGen::new(
            yarte_codegen::TextBytesCodeGen::new(&buf),
            PARENT,
        ))
    };

    let input = quote! {
        #[template(src = #src)]
        struct __Main__;
    };

    let i = &syn::parse2(input).unwrap();
    build!(
        i,
        get_codegen,
        HIROptions {
            resolve_to_self: false,
            is_text: true,
            parent: PARENT,
        }
    )
    .into()
}

#[proc_macro]
#[cfg(feature = "bytes-buf")]
/// Write handlebars template to `buf-min::Buffer` in this scope with html escape functionality
pub fn ywrite_html(i: TokenStream) -> TokenStream {
    const PARENT: &str = "yarte";

    let WriteArg { buf, src, .. } = match syn::parse(i) {
        Ok(arg) => arg,
        Err(e) => return e.to_compile_error().into(),
    };
    // Check correct syn::Expr
    {
        use syn::Expr::*;
        match &buf {
            Field(_) | Path(_) => (),
            _ => {
                return syn::Error::new(buf.span(), "first argument should be a ident or field")
                    .to_compile_error()
                    .into()
            }
        }
    }

    let get_codegen = |_| {
        Box::new(yarte_codegen::WriteBCodeGen::new(
            yarte_codegen::HTMLBytesCodeGen::new(&buf),
            PARENT,
        ))
    };

    let input = quote! {
        #[template(src = #src)]
        struct __Main__;
    };

    let i = &syn::parse2(input).unwrap();
    build!(
        i,
        get_codegen,
        HIROptions {
            resolve_to_self: false,
            parent: PARENT,
            ..Default::default()
        }
    )
    .into()
}

#[proc_macro]
#[cfg(all(feature = "html-min", feature = "bytes-buf"))]
/// Write handlebars template to `buf-min::Buffer` in this scope without html escape functionality
pub fn ywrite_min(i: TokenStream) -> TokenStream {
    const PARENT: &str = "yarte";

    let WriteArg { buf, src, .. } = match syn::parse(i) {
        Ok(arg) => arg,
        Err(e) => return e.to_compile_error().into(),
    };
    // Check correct syn::Expr
    {
        use syn::Expr::*;
        match &buf {
            Field(_) | Path(_) => (),
            _ => {
                return syn::Error::new(buf.span(), "first argument should be a ident or field")
                    .to_compile_error()
                    .into()
            }
        }
    }

    let get_codegen = |_| {
        Box::new(yarte_codegen::WriteBCodeGen::new(
            yarte_codegen::HTMLMinBytesCodeGen::new(&buf),
            PARENT,
        ))
    };

    let input = quote! {
        #[template(src = #src)]
        struct __Main__;
    };

    let i = &syn::parse2(input).unwrap();
    build!(
        i,
        get_codegen,
        HIROptions {
            resolve_to_self: false,
            parent: PARENT,
            ..Default::default()
        }
    )
    .into()
}

struct TemplateArg {
    pub s: syn::LitStr,
    _d: Option<syn::Token![;]>,
    _c: Option<syn::Token![,]>,
}

impl syn::parse::Parse for TemplateArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(TemplateArg {
            s: input.parse()?,
            _d: input.parse()?,
            _c: input.parse()?,
        })
    }
}

#[proc_macro_attribute]
#[cfg(feature = "bytes-buf")]
pub fn yarte(args: TokenStream, input: TokenStream) -> TokenStream {
    const PARENT: &str = "yarte";

    let args_is_empty = args.is_empty();
    let buf: syn::Expr = if args_is_empty {
        syn::parse_str("__buf").unwrap()
    } else {
        match syn::parse(args) {
            Ok(i) => i,
            Err(e) => return e.into_compile_error().into(),
        }
    };
    // Check correct syn::Expr
    {
        use syn::Expr::*;
        match &buf {
            Field(_) | Path(_) => (),
            _ => {
                return syn::Error::new(buf.span(), "first argument should be a ident or field")
                    .to_compile_error()
                    .into()
            }
        }
    }
    let get_codegen = |_| {
        Box::new(yarte_codegen::AttrBCodeGen::new(
            yarte_codegen::HTMLBytesCodeGen::new(&buf),
            PARENT,
            !args_is_empty,
        ))
    };
    let input = match syn::parse::<TemplateArg>(input) {
        Ok(i) => i.s,
        Err(e) => return e.into_compile_error().into(),
    };
    let input = quote! {
        #[template(src = #input)]
        struct __Main__;
    };

    let i = &syn::parse2(input).unwrap();
    let code = build!(
        i,
        get_codegen,
        HIROptions {
            resolve_to_self: false,
            parent: PARENT,
            ..Default::default()
        }
    );

    code.into()
}

struct WriteArg {
    buf: syn::Expr,
    _comma: syn::Token![,],
    src: syn::LitStr,
    _end_comma: Option<syn::Token![,]>,
}

impl syn::parse::Parse for WriteArg {
    fn parse(input: &ParseBuffer) -> syn::parse::Result<Self> {
        Ok(WriteArg {
            buf: input.parse()?,
            _comma: input.parse()?,
            src: input.parse()?,
            _end_comma: input.parse()?,
        })
    }
}

fn sources_to_tokens<'a>(
    src: String,
    config: &Config,
    s: &'a Struct<'a>,
    mut codegen: Box<dyn CodeGen + 'a>,
    opt: HIROptions,
) -> proc_macro2::TokenStream {
    let mut parsed: OwnParsed = HashMap::new();
    resolve_imports(src, Rc::clone(&s.path), config, &mut parsed)
        .unwrap_or_else(|e| emitter(&parsed, config, e));

    if cfg!(debug_assertions) && config.print_override == PrintConfig::Ast
        || config.print_override == PrintConfig::All
        || s.print == Print::Ast
        || s.print == Print::All
    {
        eprintln!("{parsed:?}\n");
    }

    let hir = generate(config, s, &parsed, opt).unwrap_or_else(|e| emitter(&parsed, config, e));
    // when multiple templates
    source_map::clean();

    let tokens = codegen.gen(hir);

    if cfg!(debug_assertions) && config.print_override == PrintConfig::Code
        || config.print_override == PrintConfig::All
        || s.print == Print::Code
        || s.print == Print::All
    {
        log(&tokens.to_string());
    }

    tokens
}
