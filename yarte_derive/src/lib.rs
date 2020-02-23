extern crate proc_macro;

use std::{
    collections::{BTreeMap, BTreeSet},
    iter,
    path::PathBuf,
};

use proc_macro::TokenStream;
use quote::quote;

use yarte_codegen::{wasm::server, CodeGen, FmtCodeGen, HTMLCodeGen, HTMLMinCodeGen, TextCodeGen};
use yarte_helpers::config::{get_source, read_config_file, Config, PrintConfig};
use yarte_hir::{generate, visit_derive, HIROptions, Mode, Print, Struct};
use yarte_parser::{emitter, parse, parse_partials, source_map, Partial};

mod logger;

use self::logger::log;

type Sources<'a> = &'a BTreeMap<PathBuf, String>;

macro_rules! build {
    ($i:ident, $codegen:path, $opt:expr) => {{
        let config_toml: &str = &read_config_file();
        let config = &Config::new(config_toml);
        let s = &visit_derive($i, config);
        let sources = &read(s.path.clone(), s.src.clone(), config);

        sources_to_tokens(sources, config, s, $codegen(s), $opt).into()
    }};
}

#[proc_macro_derive(Template, attributes(template))]
pub fn template(input: TokenStream) -> TokenStream {
    fn get_codegen<'a>(s: &'a Struct) -> Box<dyn CodeGen + 'a> {
        let codegen: Box<dyn CodeGen> = match s.mode {
            Mode::Text => Box::new(FmtCodeGen::new(TextCodeGen, s)),
            Mode::HTML => Box::new(FmtCodeGen::new(HTMLCodeGen, s)),
            Mode::HTMLMin => Box::new(FmtCodeGen::new(HTMLMinCodeGen, s)),
            Mode::WASMServer => Box::new(FmtCodeGen::new(server::WASMCodeGen::new(s), s)),
            #[cfg(feature = "wasm-app")]
            Mode::WASM => panic!("Use `yarte_wasm_app` crate instead"),
        };

        codegen
    }

    let i = &syn::parse(input).unwrap();
    build!(i, get_codegen, Default::default())
}

// TODO:
#[proc_macro_derive(App, attributes(template, msg, inner))]
#[cfg(feature = "wasm-app")]
pub fn app(input: TokenStream) -> TokenStream {
    fn build<'a>(s: &'a Struct<'a>) -> Box<dyn CodeGen + 'a> {
        Box::new(yarte_codegen::wasm::client::WASMCodeGen::new(s))
    }
    let i = &syn::parse(input).unwrap();
    build!(i, build, Default::default())
}

#[proc_macro]
pub fn ywrite(i: TokenStream) -> TokenStream {
    fn build<'a>(_s: &'a Struct<'a>) -> Box<dyn CodeGen + 'a> {
        Box::new(yarte_codegen::FnFmtCodeGen::new(HTMLCodeGen))
    }

    let src: syn::LitStr = syn::parse(i).unwrap();
    let input = quote! {
        #[template(src = #src)]
        struct __Foo__ {}
    };

    let i = &syn::parse2(input).unwrap();
    build!(
        i,
        build,
        HIROptions {
            resolve_to_self: false
        }
    )
}

fn sources_to_tokens<'a>(
    sources: Sources,
    config: &Config,
    s: &'a Struct<'a>,
    mut codegen: Box<dyn CodeGen + 'a>,
    opt: HIROptions,
) -> proc_macro2::TokenStream {
    let mut parsed = BTreeMap::new();
    for (p, src) in sources {
        let nodes = match parse(source_map::get_cursor(p, src)) {
            Ok(n) => n,
            Err(e) => emitter(sources, config, iter::once(e)),
        };
        parsed.insert(p, nodes);
    }

    if cfg!(debug_assertions) && config.print_override == PrintConfig::Ast
        || config.print_override == PrintConfig::All
        || s.print == Print::Ast
        || s.print == Print::All
    {
        eprintln!("{:?}\n", parsed);
    }

    let hir = generate(config, s, &parsed, opt)
        .unwrap_or_else(|e| emitter(sources, config, e.into_iter()));
    // when multiple templates
    source_map::clean();

    let tokens = codegen.gen(hir);

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

fn read(path: PathBuf, src: String, config: &Config) -> BTreeMap<PathBuf, String> {
    let mut stack = vec![(path, src)];
    let mut visited = BTreeMap::new();

    while let Some((path, src)) = stack.pop() {
        let partials = parse_partials(&src);

        let partials = match partials {
            Ok(n) => n
                .iter()
                .map(|Partial(_, partial, _)| config.resolve_partial(&path, partial.t()))
                .collect::<BTreeSet<_>>(),
            Err(e) => {
                visited.insert(path, src);
                emitter(&visited, config, iter::once(e))
            }
        };
        visited.insert(path, src);

        for partial in partials {
            if !visited.contains_key(&partial) {
                let src = get_source(partial.as_path());
                stack.push((partial, src));
            }
        }
    }

    visited
}
