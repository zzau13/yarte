//! Right now a Yarte configuration file can have the following:
//!
//! - **`main`** (general configuration - optional): with attribute
//!   - **`dir`**: name of template directory. If no value is given, a default directory
//! **`templates`** will be used. If the defined directory is not found, an error
//! will prompt.
//!   - **`debug`**: type of output of debug mode. The code and/or  ast generated by  Yarte
//! can be visualize, to do so, at most one of three possible values has to be given:
//! `code`, `ast`, or `all`.
//!
//! - **`partials`** (partials aliasing - optional): each entry must be of the type
//! `name_alias = "./alias/path/"`, where `./` makes reference to `dir` value. Path
//! must exist, or error will be prompt. If the tag `partials` doesn't exist no aliasing
//! will be possible.
//!
//! - **`debug`** (debugging configuration - optional): in order to visualize clearly generated code
//! in a debugging environment Yarte gives it a tabulated format, and the possibility
//! to see the number line use a color theme. Options are the following:
//!
//! > Deprecated
//!   - **`number_line`** (default:  `false`): Boolean, if set to `true` number lines will appear
//! in debug-mode.
//! > Deprecated
//!   - **`theme`** (default: `zenburn`): String, color theme used in debugging environment.
//! Possible values are:
//!     - `DarkNeon`,
//!     - `GitHub`,
//!     - `Monokai Extended`,
//!     - `Monokai Extended Bright`,
//!     - `Monokai Extended Light`,
//!     - `Monokai Extended Origin`,
//!     - `OneHalfDark`,
//!     - `OneHalfLight`,
//!     - `Sublime Snazzy`,
//!     - `TwoDark`,
//!     - `zenburn`
//! > Deprecated
//!   - **`grid`** (default:  `false`): Boolean
//! > Deprecated
//!   - **`header`** (default:  `false`): Boolean
//! > Deprecated
//!   - **`paging`** (default:  `false`): Boolean
//! > Deprecated
//!   - **`short`** (default:  `true`): Boolean, if set to `false` to verbose
//!
//! ### Example of a config file
//! ```toml
//! [main]
//! dir = "templates"
//! debug = "all"
//!
//! [partials]
//! alias = "./deep/more/deep"
//! ```
//!
//! With this configuration, the user can call `alias` in a partial instance with
//! `{{> alias context}}` or `{{> alias}}` if the current context is well defined.
//!
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

#[derive(Debug)]
pub struct Dir(PathBuf);

impl Dir {
    pub fn get_template(&self, path: &Path) -> PathBuf {
        let template = self.0.join(path);

        if template.exists() {
            template
        } else {
            panic!("template not found in directory {template:?}")
        }
    }
}

impl From<Option<&str>> for Dir {
    fn from(p: Option<&str>) -> Self {
        let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        Dir(p.map_or_else(|| root.join(DEFAULT_DIR), |v| root.join(v)))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PrintConfig {
    All,
    Ast,
    Code,
    None,
}

impl From<Option<&str>> for PrintConfig {
    fn from(s: Option<&str>) -> Self {
        match s {
            Some("all") => PrintConfig::All,
            Some("ast") => PrintConfig::Ast,
            Some("code") => PrintConfig::Code,
            _ => PrintConfig::None,
        }
    }
}

#[derive(Debug)]
pub struct Config<'a> {
    dir: Dir,
    alias: BTreeMap<&'a str, &'a str>,
    pub print_override: PrintConfig,
    pub debug: PrintOption<'a>,
}

impl<'a> Config<'a> {
    pub fn new(s: &str) -> Config {
        let raw: RawConfig =
            toml::from_str(s).unwrap_or_else(|_| panic!("invalid TOML in {CONFIG_FILE_NAME}"));
        let (dir, print) = raw.main.map(|x| (x.dir, x.debug)).unwrap_or((None, None));

        Config {
            dir: Dir::from(dir),
            print_override: PrintConfig::from(print),
            debug: raw.debug.unwrap_or_default(),
            alias: raw.partials.unwrap_or_default(),
        }
    }

    pub fn get_dir(&self) -> &PathBuf {
        &self.dir.0
    }

    pub fn get_template(&self, path: &Path) -> (PathBuf, String) {
        let path = self.dir.get_template(path);
        let src = get_source(path.as_path());
        (path, src)
    }

    pub fn resolve_partial(&self, parent: &Path, ident: &str) -> PathBuf {
        let (mut buf, is_alias) = self
            .alias
            .iter()
            .find_map(|(k, v)| {
                if let Some(stripped) = ident.strip_prefix(k) {
                    let mut path = (*v).to_string();
                    path.push_str(stripped);
                    Some(PathBuf::from(path))
                } else {
                    None
                }
            })
            .map_or((PathBuf::from(ident), false), |s| (s, true));

        if buf.extension().is_none() {
            if let Some(ext) = parent.extension() {
                buf = buf.with_extension(ext);
            }
        };

        if is_alias {
            normalize(self.dir.get_template(&buf))
        } else {
            let mut parent = parent.to_owned();
            parent.pop();
            parent.push(buf);
            normalize(parent)
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn normalize(p: PathBuf) -> PathBuf {
    p.canonicalize().expect("Correct template path")
}

#[cfg(target_os = "windows")]
fn normalize(p: PathBuf) -> PathBuf {
    p
}

#[derive(Deserialize)]
struct RawConfig<'a> {
    #[serde(borrow)]
    main: Option<Main<'a>>,
    #[serde(borrow)]
    debug: Option<PrintOption<'a>>,
    #[serde(borrow)]
    partials: Option<BTreeMap<&'a str, &'a str>>,
}

#[derive(Deserialize)]
struct Main<'a> {
    #[serde(borrow)]
    dir: Option<&'a str>,
    #[serde(borrow)]
    debug: Option<&'a str>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PrintOption<'a> {
    #[serde(borrow)]
    pub theme: Option<&'a str>,
    pub number_line: Option<bool>,
    pub grid: Option<bool>,
    pub paging: Option<bool>,
    pub header: Option<bool>,
    #[deprecated]
    pub short: Option<bool>,
}

#[allow(deprecated)]

pub fn read_config_file() -> String {
    let filename = config_file_path();
    if filename.exists() {
        fs::read_to_string(&filename)
            .unwrap_or_else(|_| panic!("unable to read {}", filename.to_str().unwrap()))
    } else {
        String::new()
    }
}

#[inline]
pub fn config_file_path() -> PathBuf {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join(CONFIG_FILE_NAME)
}

pub fn get_source(path: &Path) -> String {
    match fs::read_to_string(path) {
        Ok(mut source) => match source
            .as_bytes()
            .iter()
            .rposition(|x| !x.is_ascii_whitespace())
        {
            Some(j) => {
                source.drain(j + 1..);
                source
            }
            None => source,
        },
        _ => panic!("unable to open template file '{path:?}'"),
    }
}

static CONFIG_FILE_NAME: &str = "yarte.toml";
static DEFAULT_DIR: &str = "templates";
