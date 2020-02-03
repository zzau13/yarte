use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

use yarte_config::{get_source, Config};
use yarte_parser::{parse_partials, Partial};

pub type Sources<'a> = &'a BTreeMap<PathBuf, String>;

pub fn read(path: PathBuf, src: String, config: &Config) -> BTreeMap<PathBuf, String> {
    fn _read(path: PathBuf, src: String, config: &Config, visited: &mut BTreeMap<PathBuf, String>) {
        let partials = parse_partials(&src)
            .iter()
            .map(|Partial(_, partial, _)| config.resolve_partial(&path, partial.t()))
            .collect::<BTreeSet<_>>();

        visited.insert(path, src);

        for partial in partials {
            if !visited.contains_key(&partial) {
                let src = get_source(partial.as_path());
                _read(partial, src, config, visited);
            }
        }
    }

    let mut visited = BTreeMap::new();

    _read(path, src, config, &mut visited);

    visited
}
