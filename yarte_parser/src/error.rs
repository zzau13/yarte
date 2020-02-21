use std::fmt::Display;

use annotate_snippets::{
    display_list::DisplayList,
    formatter::DisplayListFormatter,
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};
use derive_more::Display;

use yarte_helpers::config::Config;

use crate::{source_map::Span, strnom::LexError};
use std::{collections::BTreeMap, path::PathBuf};

// TODO: #39 improve error messages
#[derive(Debug, Display, Copy, Clone)]
pub enum PError {
    #[display(fmt = "problems parsing template source")]
    Uncompleted,
    #[display(fmt = "whitespace")]
    Whitespace,
    #[display(fmt = "tag")]
    Tag,
    #[display(fmt = "comment")]
    Comment,
    #[display(fmt = "expression")]
    Expr,
    #[display(fmt = "safe")]
    Safe,
    #[display(fmt = "local")]
    Local,
    #[display(fmt = "if else")]
    IfElse,
    #[display(fmt = "raw")]
    Raw,
    #[display(fmt = "helpers")]
    Helpers,
    #[display(fmt = "partial block")]
    PartialBlock,
    #[display(fmt = "partial path")]
    PartialPath,
    #[display(fmt = "identifier")]
    Ident,
    #[display(fmt = "end expression")]
    EndExpression,
    #[display(fmt = "argument")]
    Argument,
}

impl From<LexError> for ErrorMessage<PError> {
    fn from(e: LexError) -> Self {
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
pub struct ErrorMessage<T: Display> {
    pub message: T,
    pub span: Span,
}

// TODO: #39 improve
// TODO: label
// - print all line with len limits
pub fn emitter<I, T>(sources: &BTreeMap<PathBuf, String>, config: &Config, errors: I) -> !
where
    I: Iterator<Item = ErrorMessage<T>>,
    T: Display,
{
    let mut prefix = config.get_dir().clone();
    prefix.pop();
    let mut errors: Vec<ErrorMessage<T>> = errors.collect();

    errors.sort_unstable_by(|a, b| a.span.lo.cmp(&b.span.lo));
    let slices = errors
        .into_iter()
        .map(|err| {
            let origin = err.span.file_path();
            let (lo, hi) = err.span.range_in_line();
            let start = err.span.start();
            let source = sources
                .get(&origin)
                .unwrap()
                .get(lo..hi)
                .unwrap()
                .to_string();
            let origin = origin
                .strip_prefix(&prefix)
                .unwrap()
                .to_string_lossy()
                .to_string();

            Slice {
                source,
                line_start: start.line,
                origin: Some(origin),
                annotations: vec![SourceAnnotation {
                    range: (start.column, hi),
                    label: err.message.to_string(),
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
    };

    let dl = DisplayList::from(s);
    let dlf = DisplayListFormatter::new(true, false);

    panic!("{}", dlf.format(&dl))
}
