use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

use yarte_config::{get_source, Config};
use yarte_parser::{parse_partials, Partial};

use crate::helpers::calculate_hash;

pub type Sources<'a> = &'a BTreeMap<PathBuf, String>;

pub fn read(path: PathBuf, src: String, config: &Config) -> BTreeMap<PathBuf, String> {
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
