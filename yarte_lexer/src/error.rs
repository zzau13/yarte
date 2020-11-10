use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{self, Display, Write};
use std::path::PathBuf;

use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};

use yarte_helpers::config::Config;

use crate::{source_map::Span, strnom::get_chars, Cursor};

pub trait KiError: Error + PartialEq + Clone + Copy {
    const EMPTY: Self;
    const PATH: Self;
    const UNCOMPLETED: Self;
    const WHITESPACE: Self;

    fn tag(s: &'static str) -> Self;
    fn tac(c: char) -> Self;
}

#[derive(Debug, Clone)]
pub enum LexError<K: KiError> {
    Fail(K, Span),
    Next(K, Span),
}

#[macro_export]
macro_rules! next {
    ($ty:ty) => {
        $crate::error::LexError::Next(<$ty>::EMPTY, $crate::source_map::Span { lo: 0, hi: 0 })
    };
}

pub type PResult<'a, O, E> = Result<(Cursor<'a>, O), LexError<E>>;

impl<E: KiError> From<LexError<E>> for ErrorMessage<E> {
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

// TODO: T: Priority trait
#[derive(Debug)]
pub struct ErrorMessage<T: Error> {
    pub message: T,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DOption {
    Some(String),
    None,
}

impl Display for DOption {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use DOption::*;
        match self {
            Some(s) => {
                f.write_char(' ')?;
                f.write_str(s)
            }
            None => Ok(()),
        }
    }
}

// TODO: Accumulate by priority
pub fn emitter<I, T>(sources: &BTreeMap<PathBuf, String>, config: &Config, errors: I) -> !
where
    I: Iterator<Item = ErrorMessage<T>>,
    T: Error,
{
    let mut prefix = config.get_dir().clone();
    prefix.pop();
    let mut errors: Vec<ErrorMessage<T>> = errors.collect();

    errors.sort_unstable_by(|a, b| a.span.lo.cmp(&b.span.lo));
    let slices: Vec<(String, PathBuf, Span)> = errors
        .into_iter()
        .map(|err| (err.message.to_string(), err.span.file_path(), err.span))
        .collect();
    let slices = slices
        .iter()
        .map(|(label, origin, span)| {
            let ((lo_line, hi_line), (lo, hi)) = span.range_in_file();
            let start = span.start();
            // TODO: without reallocate
            let source = sources.get(origin).unwrap();

            let origin = origin.strip_prefix(&prefix).unwrap().to_str().unwrap();

            Slice {
                source: get_chars(source, lo_line, hi_line),
                line_start: start.line,
                origin: Some(origin),
                annotations: vec![SourceAnnotation {
                    label,
                    range: (lo, hi),
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
            color: true,
            ..Default::default()
        },
    };

    panic!("{}", DisplayList::from(s))
}
