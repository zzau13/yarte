extern crate proc_macro;

use std::collections::BTreeMap;

use proc_macro::TokenStream;

use yarte_codegen::{html::HTMLCodeGen, text::TextCodeGen, CodeGen, FmtCodeGen};
use yarte_config::{read_config_file, Config, PrintConfig};
use yarte_helpers::helpers;
use yarte_hir::{generate, visit_derive, Print};
use yarte_parser::{parse, source_map};

mod logger;

use self::logger::log;

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

    let tokens = {
        let hir =
            &generate(config, s, &parsed).unwrap_or_else(|e| helpers::emitter(sources, config, e));
        if s.wrapped {
            FmtCodeGen::new(TextCodeGen, s).gen(hir)
        } else {
            FmtCodeGen::new(HTMLCodeGen, s).gen(hir)
        }
    };

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

    // when multiple templates
    source_map::clean();
    tokens.into()
}
