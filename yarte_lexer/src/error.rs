use std::collections::BTreeMap;
use std::error::Error;
use std::path::PathBuf;

use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};

use yarte_helpers::config::Config;

use crate::{get_bytes_to_chars, source_map::Span, Cursor};

pub trait KiError: Error + PartialEq + Clone {
    const EMPTY: Self;
    const PATH: Self;
    const UNCOMPLETED: Self;
    const WHITESPACE: Self;
    const COMMENTARY: Self;
    const CLOSE_BLOCK: Self;

    fn tag(s: &'static str) -> Self;
    fn tac(c: char) -> Self;
    fn expr(s: String) -> Self;
}

#[derive(Debug, Clone)]
pub enum LexError<K: KiError> {
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

// TODO: chars
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
            let source = sources.get(origin).unwrap();
            let source = &source[lo_line..hi_line];

            let origin = origin.strip_prefix(&prefix).unwrap().to_str().unwrap();

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
            color: true,
            ..Default::default()
        },
    };

    panic!("{}", DisplayList::from(s))
}
