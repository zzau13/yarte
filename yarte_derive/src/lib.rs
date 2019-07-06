extern crate proc_macro;

#[macro_use]
extern crate nom;
#[macro_use]
extern crate quote;

mod generator;
mod logger;
mod parser;

use proc_macro::TokenStream;

use std::collections::BTreeMap;

use yarte_config::{read_config_file, Config, PrintConfig};

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

    let mut sources = BTreeMap::new();

    let mut check = vec![(s.path.clone(), s.src.clone())];
    while let Some((path, src)) = check.pop() {
        for n in &parse_partials(&src) {
            match n {
                Node::Partial(_, partial, _) => {
                    check.push(config.get_partial(&path, partial));
                }
                _ => unreachable!(),
            }
        }
        sources.insert(path, src);
    }

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
