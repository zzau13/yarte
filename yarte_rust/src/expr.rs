pub enum Expr {
    Array(ExprArray),
    Assign(ExprAssign),
    AssignOp(ExprAssignOp),
    Binary(ExprBinary),
    Block(ExprBlock),
    Break(ExprBreak),
    Cast(ExprCast),
    Closure(ExprClosure),
    Continue(ExprContinue),
    ForLoop(ExprForLoop),
    If(ExprIf),
    Index(ExprIndex),
    Let(ExprLet),
    Lit(ExprLit),
    Loop(ExprLoop),
    Match(ExprMatch),
    Paren(ExprParen),
    Range(ExprRange),
    Reference(ExprReference),
    Repeat(ExprRepeat),
    Return(ExprReturn),
    Struct(ExprStruct),
    Try(ExprTry),
    TryBlock(ExprTryBlock),
    Tuple(ExprTuple),
    Type(ExprType),
    Unary(ExprUnary),
    While(ExprWhile),
    Yield(ExprYield),
}

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
}

pub enum Type {
    Array,
    BareFn,
    Group,
    ImplTrait,
    Infer,
    Macro,
    Never,
    Paren,
    Path,
    Ptr,
    Reference,
    Slice,
    TraitObject,
    Tuple,
    Verbatim,
}

/// A slice literal expression: `[a, b, c, d]`.
pub struct ExprArray {
    pub elems: Vec<Box<Expr>>,
}

/// An assignment expression: `a = compute()`.
pub struct ExprAssign {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

/// A compound assignment expression: `counter += 1`.
///
/// *This type is available only if Syn is built with the `"full"` feature.*
pub struct ExprAssignOp {
    pub left: Box<Expr>,
    pub op: BinOp,
    pub right: Box<Expr>,
}

/// A binary operation: `a + b`, `a * b`.
pub struct ExprBinary {
    pub left: Box<Expr>,
    pub op: BinOp,
    pub right: Box<Expr>,
}

/// A blocked scope: `{ ... }`.
pub struct ExprBlock {
    pub block: Vec<Box<Expr>>,
}

/// A `break`, with an optional label to break and an optional
/// expression.
pub struct ExprBreak {
    // pub label: Option<Lifetime>,
    pub expr: Option<Box<Expr>>,
}

/// A cast expression: `foo as f64`.
pub struct ExprCast {
    pub expr: Box<Expr>,
    pub ty: Box<Type>,
}

/// A closure expression: `|a, b| a + b`.
pub struct ExprClosure {
    pub movability: bool,
    pub inputs: Vec<Box<Expr>>,
    pub output: Type,
    pub body: Box<Expr>,
}

/// A `continue`, with an optional label.
pub struct ExprContinue; //{
                         // pub continue_token: Token![continue],
                         // pub label: Option<Lifetime>,
                         // }

/// A for loop: `for pat in expr { ... }`.
pub struct ExprForLoop {
    pub pat: Box<Expr>,
    pub expr: Box<Expr>,
    pub body: Vec<Box<Expr>>,
}

/// An expression contained within invisible delimiters.
///
/// This variant is important for faithfully representing the precedence
/// of expressions and is related to `None`-delimited spans in a
/// `TokenStream`.
// pub struct ExprGroup  {
//     pub group_token: token::Group,
//     pub expr: Box<Expr>,
// }

/// An `if` expression with an optional `else` block: `if expr { ... }
/// else { ... }`.
///
/// The `else` branch expression may only be an `If` or `Block`
/// expression, not any of the other types of expression.
pub struct ExprIf {
    pub cond: Box<Expr>,
    pub then_branch: Vec<Box<Expr>>,
    pub else_branch: Vec<Box<Expr>>,
}

/// A square bracketed indexing expression: `vector[2]`.
pub struct ExprIndex {
    pub expr: Box<Expr>,
    pub index: Box<Expr>,
}

/// A `let` guard: `let Some(x) = opt`.
pub struct ExprLet {
    pub pat: Box<Expr>,
    pub expr: Box<Expr>,
}

/// A literal in place of an expression: `1`, `"foo"`.
pub struct ExprLit {
    pub lit: String,
}

/// Conditionless loop: `loop { ... }`.
///
/// *This type is available only if Syn is built with the `"full"` feature.*
pub struct ExprLoop {
    pub body: Vec<Box<Expr>>,
}

/// A `match` expression: `match n { Some(n) => {}, None => {} }`.
pub struct ExprMatch {
    pub expr: Box<Expr>,
    pub arms: Vec<Arm>,
}

/// A parenthesized expression: `(a + b)`.
pub struct ExprParen {
    pub expr: Box<Expr>,
}

/// A range expression: `1..2`, `1..`, `..2`, `1..=2`, `..=2`.
pub struct ExprRange {
    pub from: Option<Box<Expr>>,
    pub limits: RangeLimits,
    pub to: Option<Box<Expr>>,
}

/// A referencing operation: `&a` or `&mut a`.
pub struct ExprReference {
    pub mutability: bool,
    pub expr: Box<Expr>,
}

/// An array literal constructed from one repeated element: `[0u8; N]`.
pub struct ExprRepeat {
    pub expr: Box<Expr>,
    pub len: Box<Expr>,
}

/// A `return`, with an optional value to be returned.
pub struct ExprReturn {
    pub expr: Option<Box<Expr>>,
}

/// A struct literal expression: `Point { x: 1, y: 1 }`.
///
/// The `rest` provides the value of the remaining fields as in `S { a:
/// 1, b: 1, ..rest }`.
pub struct ExprStruct {
    pub path: String,
    pub fields: Vec<FieldValue>,
    pub dot2_token: bool,
    pub rest: Option<Box<Expr>>,
}

/// A try-expression: `expr?`.
pub struct ExprTry {
    pub expr: Box<Expr>,
}

/// A try block: `try { ... }`.
pub struct ExprTryBlock {
    pub block: Vec<Box<Expr>>,
}

/// A tuple expression: `(a, b, c, d)`.
pub struct ExprTuple {
    pub elems: Vec<Box<Expr>>,
}

/// A type ascription expression: `foo: f64`.
pub struct ExprType {
    pub expr: Box<Expr>,
    pub ty: Box<Type>,
}

pub enum UnOp {
    Deref,
    Not,
    Neg,
}

/// A unary operation: `!x`, `*x`.
pub struct ExprUnary {
    pub op: UnOp,
    pub expr: Box<Expr>,
}

/// A while loop: `while expr { ... }`.
pub struct ExprWhile {
    pub cond: Box<Expr>,
    pub body: Vec<Box<Expr>>,
}

/// A yield expression: `yield expr`.
pub struct ExprYield {
    pub expr: Option<Box<Expr>>,
}

// impl Expr {
//     #[cfg(all(feature = "parsing", not(syn_no_const_vec_new)))]
//     const DUMMY: Self = Expr::Path(ExprPath {
//         attrs: Vec::new(),
//         qself: None,
//         path: Path {
//             leading_colon: None,
//             segments: Punctuated::new(),
//         },
//     });

//     #[cfg(all(feature = "parsing", feature = "full"))]
//     pub(crate) fn replace_attrs(&mut self, new: Vec<Attribute>) -> Vec<Attribute> {
//         match self {
//             Expr::Box(ExprBox { attrs, .. })
//             | Expr::Array(ExprArray { attrs, .. })
//             | Expr::Call(ExprCall { attrs, .. })
//             | Expr::MethodCall(ExprMethodCall { attrs, .. })
//             | Expr::Tuple(ExprTuple { attrs, .. })
//             | Expr::Binary(ExprBinary { attrs, .. })
//             | Expr::Unary(ExprUnary { attrs, .. })
//             | Expr::Lit(ExprLit { attrs, .. })
//             | Expr::Cast(ExprCast { attrs, .. })
//             | Expr::Type(ExprType { attrs, .. })
//             | Expr::Let(ExprLet { attrs, .. })
//             | Expr::If(ExprIf { attrs, .. })
//             | Expr::While(ExprWhile { attrs, .. })
//             | Expr::ForLoop(ExprForLoop { attrs, .. })
//             | Expr::Loop(ExprLoop { attrs, .. })
//             | Expr::Match(ExprMatch { attrs, .. })
//             | Expr::Closure(ExprClosure { attrs, .. })
//             | Expr::Unsafe(ExprUnsafe { attrs, .. })
//             | Expr::Block(ExprBlock { attrs, .. })
//             | Expr::Assign(ExprAssign { attrs, .. })
//             | Expr::AssignOp(ExprAssignOp { attrs, .. })
//             | Expr::Field(ExprField { attrs, .. })
//             | Expr::Index(ExprIndex { attrs, .. })
//             | Expr::Range(ExprRange { attrs, .. })
//             | Expr::Path(ExprPath { attrs, .. })
//             | Expr::Reference(ExprReference { attrs, .. })
//             | Expr::Break(ExprBreak { attrs, .. })
//             | Expr::Continue(ExprContinue { attrs, .. })
//             | Expr::Return(ExprReturn { attrs, .. })
//             | Expr::Macro(ExprMacro { attrs, .. })
//             | Expr::Struct(ExprStruct { attrs, .. })
//             | Expr::Repeat(ExprRepeat { attrs, .. })
//             | Expr::Paren(ExprParen { attrs, .. })
//             | Expr::Group(ExprGroup { attrs, .. })
//             | Expr::Try(ExprTry { attrs, .. })
//             | Expr::Async(ExprAsync { attrs, .. })
//             | Expr::Await(ExprAwait { attrs, .. })
//             | Expr::TryBlock(ExprTryBlock { attrs, .. })
//             | Expr::Yield(ExprYield { attrs, .. }) => mem::replace(attrs, new),
//             Expr::Verbatim(_) => Vec::new(),

//             #[cfg(syn_no_non_exhaustive)]
//             _ => unreachable!(),
//         }
//     }
// }

/// An individual generic argument to a method, like `T`.
pub enum GenericMethodArgument {
    /// A type argument.
    Type(Type),
    /// A const expression. Must be inside of a block.
    ///
    /// NOTE: Identity expressions are represented as Type arguments, as
    /// they are indistinguishable syntactically.
    Const(Box<Expr>),
}

/// A field-value pair in a struct literal.
pub struct FieldValue {
    /// Attributes tagged on the field.

    /// Name or index of the field.
    pub member: String,

    /// Value of the field.
    pub expr: Box<Expr>,
}

/// A lifetime labeling a `for`, `while`, or `loop`.
// pub struct Label {
//     pub name: Lifetime,
// }

/// One arm of a `match` expression: `0...10 => { return true; }`.
///
/// As in:
///
/// ```
/// # fn f() -> bool {
/// #     let n = 0;
/// match n {
///     0..=10 => {
///         return true;
///     }
///     // ...
///     # _ => {}
/// }
/// #   false
/// # }
/// ```
pub struct Arm {
    pub pat: Box<Expr>,
    pub body: Box<Expr>,
}

/// Limit types of a range, inclusive or exclusive.
///
/// *This type is available only if Syn is built with the `"full"` feature.*
pub enum RangeLimits {
    /// Inclusive at the beginning, exclusive at the end.
    HalfOpen, //..
    /// Inclusive at the beginning and end.
    Closed, // ..=
}
