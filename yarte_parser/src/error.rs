use std::{
    collections::BTreeMap,
    fmt::{self, Display, Write},
    path::PathBuf,
};

use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};
use derive_more::Display;

use yarte_helpers::config::Config;

use crate::{source_map::Span, strnom::LexError};

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

// TODO: #39 improve error messages
#[derive(Debug, Display, Clone, PartialEq)]
pub enum PError {
    #[display(fmt = "problems parsing template source")]
    Uncompleted,
    #[display(fmt = "whitespace")]
    Whitespace,
    #[display(fmt = "tag")]
    Tag,
    #[display(fmt = "comment")]
    Comment,
    #[display(fmt = "expression{}", _0)]
    Expr(DOption),
    #[display(fmt = "safe{}", _0)]
    Safe(DOption),
    #[display(fmt = "local{}", _0)]
    Local(DOption),
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
    #[display(fmt = "argument{}", _0)]
    Argument(DOption),
    #[display(fmt = "Not exist @ helper")]
    AtHelperNotExist,
    #[display(fmt = "@ helper need only {} argument", _0)]
    AtHelperArgsLen(usize),
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

// TODO: T: Priority trait
#[derive(Debug)]
pub struct ErrorMessage<T: Display> {
    pub message: T,
    pub span: Span,
}

// TODO: Accumulate by priority
pub fn emitter<I, T>(sources: &BTreeMap<PathBuf, String>, config: &Config, errors: I) -> !
where
    I: Iterator<Item = ErrorMessage<T>>,
    T: Display,
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
            let source = sources
                .get(origin)
                .unwrap()
                .get(lo_line..hi_line)
                .unwrap()
                .trim_end();
            let origin = origin.strip_prefix(&prefix).unwrap().to_str().unwrap();

            Slice {
                source,
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
