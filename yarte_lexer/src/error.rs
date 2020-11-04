use std::iter::once;
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
    #[display(fmt = "report please")]
    Empty,
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

fn get_chars(text: &str, left: usize, right: usize) -> &str {
    let mut ended = false;
    let mut taken = 0;
    let range = text
        .char_indices()
        .skip(left)
        // Complete char iterator with final character
        .chain(once((text.len(), '\0')))
        // Take until the next one to the final condition
        .take_while(|(_, ch)| {
            // Fast return to iterate over final byte position
            if ended {
                return false;
            }
            // Make sure that the trimming on the right will fall within the terminal width.
            // FIXME: `unicode_width` sometimes disagrees with terminals on how wide a `char` is.
            // For now, just accept that sometimes the code line will be longer than desired.
            taken += unicode_width::UnicodeWidthChar::width(*ch).unwrap_or(1);
            if taken > right - left {
                ended = true;
            }
            true
        })
        // Reduce to start and end byte position
        .fold((None, 0), |acc, (i, _)| {
            if acc.0.is_some() {
                (acc.0, i)
            } else {
                (Some(i), i)
            }
        });

    if let Some(left) = range.0 {
        &text[left..range.1]
    } else {
        ""
    }
}
