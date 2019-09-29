extern crate proc_macro;

#[macro_use]
extern crate nom;
#[macro_use]
extern crate quote;

use std::{
    collections::{hash_map::DefaultHasher, BTreeMap, BTreeSet},
    hash::{Hash, Hasher},
    path::PathBuf,
};

use proc_macro::TokenStream;

use yarte_config::{get_source, read_config_file, Config, PrintConfig};

mod generator;
mod logger;
mod parser;

use crate::generator::{visit_derive, Print};
use crate::logger::log;
use crate::parser::{parse, parse_partials, Node};

#[proc_macro_derive(Template, attributes(template))]
pub fn derive(input: TokenStream) -> TokenStream {
    build(&syn::parse(input).unwrap())
}

#[inline]
fn build(i: &syn::DeriveInput) -> TokenStream {
    let config_toml: &str = &read_config_file();
    let config = &Config::new(config_toml);

    let s = visit_derive(i, &config);

    let sources = read(s.path.clone(), s.src.clone(), config);

    let mut parsed = BTreeMap::new();
    for (p, src) in &sources {
        parsed.insert(p, parse(src));
    }

    if cfg!(debug_assertions) && config.print_override == PrintConfig::Ast
        || config.print_override == PrintConfig::All
        || s.print == Print::Ast
        || s.print == Print::All
    {
        eprintln!("{:?}\n", parsed);
    }

    let code = generator::generate(&config, &s, &parsed);
    if cfg!(debug_assertions) && config.print_override == PrintConfig::Code
        || config.print_override == PrintConfig::All
        || s.print == Print::Code
        || s.print == Print::All
    {
        log(&code, s.path.to_str().unwrap().to_owned(), &config.debug);
    }

    code.parse().unwrap()
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
            .map(|n| match n {
                Node::Partial(_, partial, _) => config.resolve_partial(&path, partial),
                _ => unreachable!(),
            })
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
    let mut stack = Vec::new();

    _read(path, src, config, &mut visited, &mut stack);

    visited
}
