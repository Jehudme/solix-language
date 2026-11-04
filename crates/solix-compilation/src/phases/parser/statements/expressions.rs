//! Strongly-typed AST representations for Solix expressions and operators.

#![forbid(unsafe_code)]

use crate::phases::lexing::{Span, TokenKind};
use crate::phases::parser::statements::control_flow::BlockStatement;
use crate::phases::parser::statements::declarations::Parameter;
use crate::phases::parser::statements::types::TypeAnnotation;

/// Evaluated literal constant value parsed from source code.
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    /// 64-bit signed integer value.
    Int(i64),
    /// 64-bit unsigned integer value.
    UInt(u64),
    /// 64-bit IEEE 754 floating-point value.
    Float(f64),
    /// Decoded UTF-8 string literal.
    String(String),
    /// Decoded Unicode character literal.
    Char(char),
    /// Boolean literal (`true` or `false`).
    Bool(bool),
    /// Literal null pointer / object reference (`null`).
    Null,
}

/// Body representation for a lambda closure expression.
#[derive(Debug, Clone, PartialEq)]
pub enum LambdaBody {
    /// Single expression body (e.g. `(x) => x + 1`).
    Expression(Box<Expression>),
    /// Block body with multiple statements (e.g. `(x) => { return x + 1; }`).
    Block(BlockStatement),
}

/// Abstract Syntax Tree node representing a Solix expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Literal constant value (numbers, strings, booleans, null).
    Literal {
        /// Overall source coordinate span.
        span: Span,
        /// Decoded constant literal value.
        value: LiteralValue,
    },
    /// Variable or symbol identifier reference.
    Identifier {
        /// Overall source coordinate span of the identifier.
        span: Span,
    },
    /// Binary arithmetic, comparison, logical, or bitwise operation.
    Binary {
        /// Overall source coordinate span from left operand to right operand.
        span: Span,
        /// Left-hand side operand.
        left: Box<Expression>,
        /// Binary operator token kind.
        operator: TokenKind,
        /// Right-hand side operand.
        right: Box<Expression>,
    },
    /// Unary prefix or postfix operation.
    Unary {
        /// Overall source coordinate span of the unary expression.
        span: Span,
        /// Unary operator token kind (`!`, `-`, `~`, `++`, `--`).
        operator: TokenKind,
        /// Operand target.
        operand: Box<Expression>,
        /// Whether this is a prefix operator (true) or postfix operator (false).
        is_prefix: bool,
    },
    /// Assignment expression (`=`, `+=`, `-=`, `*=`, `/=`, `%=`).
    Assignment {
        /// Overall source coordinate span.
        span: Span,
        /// Target assignment l-value.
        target: Box<Expression>,
        /// Assignment operator token kind.
        operator: TokenKind,
        /// Assigned value expression.
        value: Box<Expression>,
    },
    /// Object member access or static member lookup (`object.member` or `Type::member`).
    MemberAccess {
        /// Overall source coordinate span.
        span: Span,
        /// Target object or namespace expression.
        object: Box<Expression>,
        /// Identifier span of the accessed member name.
        member_span: Span,
        /// Whether this was accessed via scope resolution operator `::` (true) or dot operator `.` (false).
        is_scope_resolution: bool,
    },
    /// Invocation / method call (`callee(arguments)`).
    MethodCall {
        /// Overall source coordinate span from callee to closing `)`.
        span: Span,
        /// The callable target expression.
        callee: Box<Expression>,
        /// Evaluated arguments passed to the call.
        arguments: Vec<Expression>,
    },
    /// Indexed array or collection lookup (`array[index]`).
    ArrayAccess {
        /// Overall source coordinate span from array to closing `]`.
        span: Span,
        /// Target indexed array expression.
        array: Box<Expression>,
        /// Index expression between brackets.
        index: Box<Expression>,
    },
    /// Dynamic array allocation expression (`new Type[size]`).
    ArrayCreation {
        /// Overall source coordinate span from `new` to closing `]`.
        span: Span,
        /// Element type to allocate.
        element_type: TypeAnnotation,
        /// Capacity / length expression.
        size: Box<Expression>,
    },
    /// Inline array literal definition (`[expr1, expr2, ...]`).
    ArrayLiteral {
        /// Overall source coordinate span from `[` to `]`.
        span: Span,
        /// List of element expressions in the array.
        elements: Vec<Expression>,
    },
    /// Class or struct instantiation expression (`new Class(args)`).
    NewInstance {
        /// Overall source coordinate span from `new` to closing `)`.
        span: Span,
        /// Target instantiated type annotation.
        instance_type: TypeAnnotation,
        /// Constructor invocation arguments.
        arguments: Vec<Expression>,
    },
    /// Explicit type conversion expression (`cast(expr, TargetType)`).
    Cast {
        /// Overall source coordinate span from `cast` to closing `)`.
        span: Span,
        /// Expression to convert.
        expression: Box<Expression>,
        /// Destination target type.
        target_type: TypeAnnotation,
    },
    /// Type query check expression (`expr instanceof TargetType`).
    Instanceof {
        /// Overall source coordinate span.
        span: Span,
        /// Target tested expression.
        expression: Box<Expression>,
        /// Tested type annotation.
        target_type: TypeAnnotation,
    },
    /// Ternary conditional expression (`cond ? true_expr : false_expr`).
    Ternary {
        /// Overall source coordinate span.
        span: Span,
        /// Conditional boolean predicate.
        condition: Box<Expression>,
        /// Expression evaluated if condition is true.
        then_branch: Box<Expression>,
        /// Expression evaluated if condition is false.
        else_branch: Box<Expression>,
    },
    /// Lambda closure definition (`(params) => expr` or `[captures](params) => { block }`).
    Lambda {
        /// Overall source coordinate span.
        span: Span,
        /// Explicit capture list variables (e.g. `[a, b, this]`).
        captures: Vec<Span>,
        /// Closure parameter list.
        parameters: Vec<Parameter>,
        /// Closure execution body.
        body: LambdaBody,
    },
    /// Size query expression (`sizeof(Type)`).
    Sizeof {
        /// Overall source coordinate span.
        span: Span,
        /// Type queried for byte size.
        target_type: TypeAnnotation,
    },
    /// Default value query expression (`default(Type)`).
    Default {
        /// Overall source coordinate span.
        span: Span,
        /// Type queried for default initialization value.
        target_type: TypeAnnotation,
    },
    /// Inline tuple instantiation (`tuple(a, b, c)` or `(a, b)`).
    TupleCreation {
        /// Overall source coordinate span.
        span: Span,
        /// Element expressions composing the tuple.
        elements: Vec<Expression>,
    },
}

impl Expression {
    /// Returns the overall source coordinate span of this expression.
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Self::Literal { span, .. }
            | Self::Identifier { span, .. }
            | Self::Binary { span, .. }
            | Self::Unary { span, .. }
            | Self::Assignment { span, .. }
            | Self::MemberAccess { span, .. }
            | Self::MethodCall { span, .. }
            | Self::ArrayAccess { span, .. }
            | Self::ArrayCreation { span, .. }
            | Self::ArrayLiteral { span, .. }
            | Self::NewInstance { span, .. }
            | Self::Cast { span, .. }
            | Self::Instanceof { span, .. }
            | Self::Ternary { span, .. }
            | Self::Lambda { span, .. }
            | Self::Sizeof { span, .. }
            | Self::Default { span, .. }
            | Self::TupleCreation { span, .. } => *span,
        }
    }
}
