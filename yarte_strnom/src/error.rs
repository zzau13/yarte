use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::path::{Path, PathBuf};

use annotate_snippets::display_list::{DisplayList, FormatOptions};
use annotate_snippets::snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation};

use derive_more::Display;

use crate::{get_bytes_to_chars, source_map::Span, Cursor};

#[allow(clippy::declare_interior_mutable_const)]
pub trait KiError: Error + PartialEq + Clone {
    const EMPTY: Self;
    const UNCOMPLETED: Self;
    const PATH: Self;
    const WHITESPACE: Self;

    fn str(s: &'static str) -> Self;
    fn char(c: char) -> Self;
    fn string(s: String) -> Self;
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Empty;

impl fmt::Display for Empty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl Error for Empty {}
impl KiError for Empty {
    const EMPTY: Self = Empty;
    const UNCOMPLETED: Self = Empty;
    const PATH: Self = Empty;
    const WHITESPACE: Self = Empty;

    #[inline]
    fn str(_: &'static str) -> Self {
        Empty
    }

    #[inline]
    fn char(_: char) -> Self {
        Empty
    }

    #[inline]
    fn string(_: String) -> Self {
        Empty
    }
}

#[derive(Display, Debug, PartialEq, Eq, Clone)]
pub enum Some {
    #[display(fmt = "Empty")]
    Empty,
    #[display(fmt = "Uncompleted")]
    Uncompleted,
    #[display(fmt = "Path")]
    Path,
    #[display(fmt = "Whitespace")]
    Whitespace,
    #[display(fmt = "{_0}")]
    Str(&'static str),
    #[display(fmt = "{_0}")]
    Char(char),
    #[display(fmt = "{_0}")]
    String(String),
}

impl Error for Some {}

impl KiError for Some {
    const EMPTY: Self = Some::Empty;
    const UNCOMPLETED: Self = Some::Uncompleted;
    const PATH: Self = Some::Path;
    const WHITESPACE: Self = Some::Whitespace;

    #[inline]
    fn str(s: &'static str) -> Self {
        Some::Str(s)
    }

    #[inline]
    fn char(s: char) -> Self {
        Some::Char(s)
    }

    #[inline]
    fn string(s: String) -> Self {
        Some::String(s)
    }
}

#[derive(Debug, Clone)]
pub enum LexError<K> {
    Fail(K, Span),
    Next(K, Span),
}

#[macro_export]
macro_rules! next {
    ($ty:ty) => {
        $crate::LexError::Next(<$ty>::EMPTY, $crate::Span { lo: 0, hi: 0 })
    };
}

pub type Result<'a, O, E> = std::result::Result<(Cursor<'a>, O), LexError<E>>;
pub type CResult<'a, E> = std::result::Result<Cursor<'a>, LexError<E>>;

impl<E: Error> From<LexError<E>> for ErrorMessage<E> {
    fn from(e: LexError<E>) -> Self {
        use LexError::*;
        match e {
            Next(m, s) | Fail(m, s) => ErrorMessage {
                message: m,
                span: s,
            },
        }
    }
}

#[derive(Debug)]
pub struct ErrorMessage<E: Error> {
    pub message: E,
    pub span: Span,
}

pub struct EmitterConfig<'a> {
    pub sources: &'a BTreeMap<PathBuf, String>,
    pub config: Config<'a>,
}

impl<'a> std::fmt::Debug for EmitterConfig<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("Emmiter")?;
        Debug::fmt(self.sources, f)
    }
}

pub struct Config<'a> {
    pub color: bool,
    pub prefix: Option<&'a Path>,
}

pub trait Emitter: Debug {
    fn get(&self, path: &Path) -> Option<&str>;
    fn config(&self) -> &Config;
}

impl<'a> Emitter for EmitterConfig<'a> {
    fn get(&self, path: &Path) -> Option<&str> {
        self.sources.get(path).map(|x| x.as_str())
    }

    fn config(&self) -> &Config {
        &self.config
    }
}

// TODO: Warnings and another types
// TODO: Check annotate snippets
pub fn emitter<Who, E, M, I>(who: Who, errors: I) -> String
where
    Who: Emitter,
    E: Into<ErrorMessage<M>>,
    M: Error,
    I: Iterator<Item = E>,
{
    let Config { prefix, color } = who.config();
    let prefix = prefix.unwrap_or_else(|| Path::new(""));
    let mut errors: Vec<ErrorMessage<M>> = errors.map(Into::into).collect();

    errors.sort_by(|a, b| a.span.lo.cmp(&b.span.lo));
    let slices: Vec<(String, PathBuf, Span)> = errors
        .into_iter()
        .map(|err| (err.message.to_string(), err.span.file_path(), err.span))
        .collect();
    let slices = slices
        .iter()
        .map(|(label, origin, span)| {
            let ((lo_line, hi_line), (lo, hi)) = span.range_in_file();
            let start = span.start();
            let source = who.get(origin).expect("Who get source");
            let source = &source[lo_line..hi_line];

            let origin = origin
                .strip_prefix(prefix)
                .expect("template prefix")
                .to_str()
                .unwrap();

            Slice {
                source,
                line_start: start.line,
                origin: Some(origin),
                annotations: vec![SourceAnnotation {
                    label,
                    range: get_bytes_to_chars(source, lo, hi),
                    annotation_type: AnnotationType::Error,
                }],
                fold: false,
            }
        })
        .collect();

    // TODO: Margin annotate-snippets
    let s = Snippet {
        title: Some(Annotation {
            id: None,
            label: None,
            annotation_type: AnnotationType::Error,
        }),
        footer: vec![],
        slices,
        opt: FormatOptions {
            color: *color,
            ..Default::default()
        },
    };

    format!("{}", DisplayList::from(s))
}

#[cfg(test)]
mod test {
    use super::*;

    use std::fmt::Display;
    use std::iter::once;

    use crate::source_map::get_cursor;

    #[derive(Debug)]
    struct Errr(&'static str);
    impl Error for Errr {}
    impl fmt::Display for Errr {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            Display::fmt(self.0, f)
        }
    }

    // TODO: check annotate-snipped
    #[test]
    fn test_chars() {
        let path = PathBuf::from("foo.hbs");

        let src = "foó bañ tuú foú";
        let l = src.len() as u32;
        let mut sources = BTreeMap::new();
        let _ = get_cursor(&path, src);
        sources.insert(path, src.to_owned());

        let expected =
            "error\n --> foo.hbs:1:9\n  |\n1 | foó bañ tuú foú\n  |         ^^^ bar\n  |";
        let who = EmitterConfig {
            sources: &sources,
            config: Config {
                color: false,
                prefix: None,
            },
        };

        let result = emitter(
            who,
            once(ErrorMessage {
                message: Errr("bar"),
                span: Span { lo: 10, hi: 14 },
            }),
        );

        assert_eq!(result, expected);

        let path = PathBuf::from("bars.hbs");

        let src = "foó bañ \ntuú\n foú";
        let mut sources = BTreeMap::new();
        let _ = get_cursor(&path, src);
        sources.insert(path, src.to_owned());
        // TODO: check annotated-snipped
        let expected = "error\n --> bars.hbs:1:5\n  |\n1 |   foó bañ \n  |  _____^\n2 | | tuú\n3 | |  foú\n  | |___^ bar\n  |";
        let who = EmitterConfig {
            sources: &sources,
            config: Config {
                color: false,
                prefix: None,
            },
        };

        let result = emitter(
            who,
            once(ErrorMessage {
                message: Errr("bar"),
                span: Span {
                    lo: l + 5 + 1,
                    hi: l + 19 + 1,
                },
            }),
        );
        assert_eq!(result, expected);
    }
}
