use derive_more::Display;

use yarte_parser::{source_map::Span, ErrorMessage};

pub type GResult<T> = Result<T, GError>;

pub(crate) struct MiddleError {
    message: GError,
    in_span: proc_macro2::Span,
    span: Span,
}

impl MiddleError {
    pub(crate) fn new(message: GError, in_span: proc_macro2::Span, span: Span) -> Self {
        Self {
            message,
            in_span,
            span,
        }
    }
}

impl From<MiddleError> for ErrorMessage<GError> {
    fn from(val: MiddleError) -> Self {
        let MiddleError {
            message,
            in_span,
            span,
        } = val;
        ErrorMessage {
            message,
            span: span.join_proc(in_span),
        }
    }
}

// TODO: #39 improve error messages
#[derive(Display, Clone)]
pub enum GError {
    #[display(fmt = "Recursion limit")]
    RecursionLimit,
    #[display(fmt = "Not available Rust expression in a template expression")]
    ValidatorExpression,
    #[display(fmt = "Not available Rust expression in a template `if helper` arguments")]
    ValidatorIfs,
    #[display(fmt = "Not available Rust expression in a template `each helper` argument")]
    ValidatorEach,
    #[display(fmt = "Unary negate operator in `unless helper`, use `if helper` instead")]
    ValidatorUnlessNegate,
    #[display(fmt = "Not available Rust expression in a template `unless helper` expression")]
    ValidatorUnless,
    #[display(fmt = "Not available Rust expression in partial scope argument")]
    ValidatorPartialScope,
    #[display(fmt = "Not available Rust expression in partial assign argument")]
    ValidatorPartialAssign,
    #[display(fmt = "Use inside partial block")]
    PartialBlockNoParent,
    #[display(fmt = "Not available in a template expression")]
    NotAvailable,
    #[display(fmt = "Not available in partial argument")]
    PartialArguments,
    #[display(fmt = "Not available Rust expression in partial scope argument")]
    PartialArgumentsScope,
    #[display(fmt = "place scope argument at first position")]
    PartialArgumentsScopeFirst,
    #[display(fmt = "Use reserved word")]
    ReservedWord,
    #[display(fmt = "Not exist in current scope")]
    NotExist,
    #[display(fmt = "Unimplemented")]
    Unimplemented,
    #[display(fmt = "Compile error: {_0}")]
    UserCompileError(String),
    #[display(fmt = "Internal")]
    Internal,
    #[display(fmt = "use super without any parent")]
    SuperWithoutParent,
}
