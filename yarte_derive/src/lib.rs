extern crate proc_macro;

use std::collections::BTreeMap;

use proc_macro::TokenStream;

use yarte_codegen::{
    wasm::{client, server},
    CodeGen, FmtCodeGen, HTMLCodeGen, HTMLMinCodeGen, TextCodeGen,
};
use yarte_config::{read_config_file, Config, PrintConfig};
use yarte_helpers::helpers;
use yarte_hir::{generate, visit_derive, Mode, Print, Struct, HIR};
use yarte_parser::{parse, source_map};


mod logger;

use self::logger::log;
use yarte_helpers::helpers::Sources;

#[proc_macro_derive(Template, attributes(template))]
pub fn derive(input: TokenStream) -> TokenStream {
    build(&syn::parse(input).unwrap())
}

#[inline]
fn build(i: &syn::DeriveInput) -> TokenStream {
    let config_toml: &str = &read_config_file();
    let config = &Config::new(config_toml);
    let s = &visit_derive(i, config);
    let sources = &helpers::read(s.path.clone(), s.src.clone(), config);

    sources_to_tokens(sources, config, s).into()
}

fn sources_to_tokens(sources: Sources, config: &Config, s: &Struct) -> proc_macro2::TokenStream {
    let mut parsed = BTreeMap::new();
    for (p, src) in sources {
        parsed.insert(p, parse(source_map::get_cursor(p, src)));
    }

    if cfg!(debug_assertions) && config.print_override == PrintConfig::Ast
        || config.print_override == PrintConfig::All
        || s.print == Print::Ast
        || s.print == Print::All
    {
        eprintln!("{:?}\n", parsed);
    }

    let hir = generate(config, s, &parsed).unwrap_or_else(|e| helpers::emitter(sources, config, e));
    // when multiple templates
    source_map::clean();

    let tokens = hir_to_tokens(hir, config, s);

    if cfg!(debug_assertions) && config.print_override == PrintConfig::Code
        || config.print_override == PrintConfig::All
        || s.print == Print::Code
        || s.print == Print::All
    {
        log(
            &tokens.to_string(),
            s.path.to_str().unwrap().to_owned(),
            &config.debug,
        );
    }

    tokens
}

fn hir_to_tokens(hir: Vec<HIR>, config: &Config, s: &Struct) -> proc_macro2::TokenStream {
    let mut codegen: Box<dyn CodeGen> = match s.mode {
        Mode::Text => Box::new(FmtCodeGen::new(TextCodeGen, s)),
        Mode::HTML => Box::new(FmtCodeGen::new(HTMLCodeGen, s)),
        Mode::HTMLMin => Box::new(FmtCodeGen::new(HTMLMinCodeGen, s)),
        Mode::WASM => Box::new(client::WASMCodeGen::new(config, s)),
        Mode::WASMServer => Box::new(server::WASMCodeGen::new(config, s)),
    };

    CodeGen::gen(&mut *codegen, hir)
}
