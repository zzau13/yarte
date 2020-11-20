use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};

use yarte_helpers::config::Config;

use crate::source_map::clean;
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

#[derive(Debug, PartialEq, Clone)]
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

pub type PResult<'a, O, E> = Result<(Cursor<'a>, O), LexError<E>>;

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
pub struct ErrorMessage<T: Error> {
    pub message: T,
    pub span: Span,
}

// TODO:
pub struct EmitterConfig<'a> {
    pub config: &'a Config<'a>,
    pub color: bool,
}

pub trait Emitter {
    fn callback() {
        clean();
    }
}

impl<'a> Emitter for EmitterConfig<'a> {}

pub fn emitter<I, T>(
    sources: &BTreeMap<PathBuf, String>,
    EmitterConfig { config, color }: EmitterConfig,
    errors: I,
) -> !
where
    I: Iterator<Item = ErrorMessage<T>>,
    T: Error,
{
    let mut prefix = config.get_dir().clone();
    prefix.pop();
    let mut errors: Vec<ErrorMessage<T>> = errors.collect();

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
            let source = sources.get(origin).unwrap();
            let source = &source[lo_line..hi_line];

            let origin = origin
                .strip_prefix(&prefix)
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

    let s = Snippet {
        title: Some(Annotation {
            id: None,
            label: None,
            annotation_type: AnnotationType::Error,
        }),
        footer: vec![],
        slices,
        opt: FormatOptions {
            color,
            ..Default::default()
        },
    };

    EmitterConfig::callback();
    panic!("{}", DisplayList::from(s))
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;
    use std::error::Error;
    use std::fmt::{self, Display};
    use std::iter::once;

    use yarte_helpers::config::Config;

    use crate::error::EmitterConfig;
    use crate::source_map::get_cursor;
    use crate::{emitter, ErrorMessage, Span};

    #[derive(Debug)]
    struct Errr(&'static str);
    impl Error for Errr {}
    impl Display for Errr {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            self.0.fmt(f)
        }
    }

    // TODO: check annotate-snipped
    #[test]
    #[should_panic(
        expected = "error\n --> foo.hbs:1:9\n  |\n1 | foó bañ tuú foú\n  |         ^^^ bar\n  |"
    )]
    fn test_chars() {
        let config = Config::new("");
        let mut path = config.get_dir().clone();
        path.pop();
        path.push("foo.hbs");

        let src = "foó bañ tuú foú";
        let mut sources = BTreeMap::new();
        let _ = get_cursor(&path, src);
        sources.insert(path, src.to_owned());

        emitter(
            &sources,
            EmitterConfig {
                config: &config,
                color: false,
            },
            // TODO: Check this numbers
            once(ErrorMessage {
                message: Errr("bar"),
                span: Span { lo: 10, hi: 14 },
            }),
        )
    }
}
