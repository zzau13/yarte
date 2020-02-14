use derive_more::Display;

pub type GResult<T> = Result<T, GError>;

// TODO: improve error messages
#[derive(Display)]
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
    #[display(fmt = "Use inside partial block")]
    PartialBlockNoParent,
}
