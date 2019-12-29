extern crate proc_macro;

use std::{
    collections::{hash_map::DefaultHasher, BTreeMap, BTreeSet},
    hash::{Hash, Hasher},
    path::PathBuf,
};

use proc_macro::TokenStream;

use yarte_config::{get_source, read_config_file, Config, PrintConfig};
use yarte_parser::{parse, parse_partials, source_map, Partial};

mod codegen;
mod error;
mod generator;
mod logger;

use self::{
    codegen::{html::HTMLCodeGen, text::TextCodeGen, CodeGen, FmtCodeGen},
    error::emitter,
    generator::{visit_derive, Print},
    logger::log,
};

#[proc_macro_derive(Template, attributes(template))]
pub fn derive(input: TokenStream) -> TokenStream {
    build(&syn::parse(input).unwrap())
}

type Sources<'a> = &'a BTreeMap<PathBuf, String>;

#[inline]
fn build(i: &syn::DeriveInput) -> TokenStream {
    let config_toml: &str = &read_config_file();
    let config = &Config::new(config_toml);

    let s = &visit_derive(i, config);

    let sources = &read(s.path.clone(), s.src.clone(), config);

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
        let hir = &generator::generate(config, s, &parsed)
            .unwrap_or_else(|e| emitter(sources, config, e));
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

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn read(path: PathBuf, src: String, config: &Config) -> BTreeMap<PathBuf, String> {
    fn _read(
        path: PathBuf,
        src: String,
        config: &Config,
        visited: &mut BTreeMap<PathBuf, String>,
        stack: &mut Vec<u64>,
    ) {
        stack.push(calculate_hash(&path));

        let partials = parse_partials(&src)
            .iter()
            .map(|Partial(_, partial, _)| config.resolve_partial(&path, partial.t()))
            .collect::<BTreeSet<_>>();

        visited.insert(path.clone(), src);

        for partial in partials {
            if !visited.contains_key(&partial) {
                let src = get_source(partial.as_path());
                _read(partial, src, config, visited, stack);
            } else if stack.contains(&calculate_hash(&partial)) {
                panic!(
                    "Partial cyclic dependency {:?} in template {:?}",
                    partial, path
                );
            }
        }

        stack.pop();
    }

    let mut visited = BTreeMap::new();

    _read(path, src, config, &mut visited, &mut Vec::new());

    visited
}
