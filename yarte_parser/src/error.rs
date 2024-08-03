use std::fmt::{self, Debug, Display, Write};

use annotate_snippets::{Level, Renderer, Snippet};
use derive_more::Display;

use yarte_helpers::config::Config;

use crate::{source_map::Span, strnom::LexError, Parsed};

#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Display, Clone, PartialEq, Eq)]
pub enum PError {
    #[display(fmt = "problems parsing template source")]
    Uncompleted,
    #[display(fmt = "whitespace")]
    Whitespace,
    #[display(fmt = "tag")]
    Tag,
    #[display(fmt = "comment")]
    Comment,
    #[display(fmt = "expression{_0}")]
    Expr(DOption),
    #[display(fmt = "safe{_0}")]
    Safe(DOption),
    #[display(fmt = "local{_0}")]
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
    #[display(fmt = "argument{_0}")]
    Argument(DOption),
    #[display(fmt = "Not exist @ helper")]
    AtHelperNotExist,
    #[display(fmt = "@ helper need only {_0} argument")]
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
pub struct ErrorMessage<T: Display + Debug> {
    pub message: T,
    pub span: Span,
}

pub type MResult<T, E> = Result<T, ErrorMessage<E>>;

// TODO: Accumulate by priority
pub fn emitter<I, T>(sources: Parsed, config: &Config, errors: I) -> !
where
    I: IntoIterator<Item = ErrorMessage<T>>,
    T: Display + Debug,
{
    let mut prefix = config.get_dir().clone();
    prefix.pop();
    let mut errors: Vec<ErrorMessage<T>> = errors.into_iter().collect();

    errors.sort_unstable_by(|a, b| a.span.lo.cmp(&b.span.lo));

    let data = errors
        .into_iter()
        .map(|err| (err.message.to_string(), err.span.file_path(), err.span))
        .collect::<Vec<_>>();
    let snippets = data.iter().map(|(label, origin, span)| {
        let ((lo_line, hi_line), (lo, hi)) = span.range_in_file();
        let start = span.start();
        let source = sources
            .get(origin)
            .expect("exists sources")
            .0
            .as_str()
            .get(lo_line..hi_line)
            .unwrap()
            .trim_end();

        let origin = origin.strip_prefix(&prefix).unwrap().to_str().unwrap();
        Snippet::source(source)
            .line_start(start.line)
            .origin(origin)
            .annotation(Level::Error.span(lo..hi).label(label))
    });
    let message = Level::Error.title("").snippets(snippets);

    panic!("{}", Renderer::styled().render(message))
}
