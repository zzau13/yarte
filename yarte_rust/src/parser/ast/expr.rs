use serde::Deserialize;

#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Expr {
    Array(ExprArray),
    Binary(ExprBinary),
    Index(ExprIndex),
    Lit(ExprLit),
    Paren(ExprParen),
    Range(ExprRange),
    Reference(ExprReference),
    Repeat(ExprRepeat),
    Tuple(ExprTuple),
    /// A function call expression: `invoke(a, b)`.
    Call(ExprCall),
    // TODO: check before template logic for completeness ?
    //  i.e, no use ?
    Try(ExprTry),
    Unary(ExprUnary),
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    BitXor,
    BitAnd,
    BitOr,
    Shl,
    Shr,
    Eq,
    Lt,
    Le,
    Ne,
    Ge,
    Gt,
    AddEq,
    SubEq,
    MulEq,
    DivEq,
    RemEq,
    BitXorEq,
    BitAndEq,
    BitOrEq,
    ShlEq,
    ShrEq,
    Pipe,
}

/// A slice literal expression: `[a, b, c, d]`.
#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct ExprArray {
    pub elems: Vec<Expr>,
}

/// An assignment expression: `a = compute()`.
#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct ExprAssign {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

/// A compound assignment expression: `counter += 1`.
///
/// *This type is available only if Syn is built with the `"full"` feature.*
#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct ExprAssignOp {
    pub left: Box<Expr>,
    pub op: BinOp,
    pub right: Box<Expr>,
}

/// A binary operation: `a + b`, `a * b`.
#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct ExprBinary {
    pub left: Box<Expr>,
    pub op: BinOp,
    pub right: Box<Expr>,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct ExprCall {
    pub func: Box<Expr>,
    pub args: Vec<Expr>,
}

/// A square bracketed indexing expression: `vector[2]`.
#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct ExprIndex {
    pub expr: Box<Expr>,
    pub index: Box<Expr>,
}

/// A literal in place of an expression: `1`, `"foo"`.
#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct ExprLit {
    pub lit: String,
}

/// A parenthesized expression: `(a + b)`.
#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct ExprParen {
    pub expr: Box<Expr>,
}

/// A range expression: `1..2`, `1..`, `..2`, `1..=2`, `..=2`.
#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct ExprRange {
    pub from: Option<Box<Expr>>,
    pub limits: RangeLimits,
    pub to: Option<Box<Expr>>,
}

/// A referencing operation: `&a` or `&mut a`.
#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct ExprReference {
    pub mutability: bool,
    pub expr: Box<Expr>,
}

/// An array literal constructed from one repeated element: `[0u8; N]`.
#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct ExprRepeat {
    pub expr: Box<Expr>,
    pub len: Box<Expr>,
}

/// A try-expression: `expr?`.
#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct ExprTry {
    pub expr: Box<Expr>,
}

/// A tuple expression: `(a, b, c, d)`.
#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct ExprTuple {
    pub elems: Vec<Expr>,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum UnOp {
    Deref,
    Not,
    Neg,
}

/// A unary operation: `!x`, `*x`.
#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct ExprUnary {
    pub op: UnOp,
    pub expr: Box<Expr>,
}

/// Limit types of a range, inclusive or exclusive.
#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum RangeLimits {
    /// Inclusive at the beginning, exclusive at the end.
    HalfOpen, //..
    /// Inclusive at the beginning and end.
    Closed, // ..=
}
