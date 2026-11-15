//! Imperative control-flow statements, loops, conditionals, and bindings.

#![forbid(unsafe_code)]

use crate::phases::lexing::Span;
use crate::phases::parser::statements::Statement;
use crate::phases::parser::statements::expressions::Expression;
use crate::phases::parser::statements::types::TypeAnnotation;
use serde::{Deserialize, Serialize};

/// A compound block statement enclosed by curly braces (`{ stmt1; stmt2; }`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockStatement {
    /// Overall source coordinate span from `{` to `}`.
    pub span: Span,
    /// List of nested statements within the block.
    pub statements: Vec<Statement>,
}

/// Conditional branch statement (`if (cond) stmt [else stmt]`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IfStatement {
    /// Overall source coordinate span of the if-statement.
    pub span: Span,
    /// The evaluated boolean condition expression.
    pub condition: Expression,
    /// Statement executed if condition evaluates to true.
    pub then_branch: Box<Statement>,
    /// Optional statement executed if condition evaluates to false.
    pub else_branch: Option<Box<Statement>>,
}

/// Loop statement repeating while a condition evaluates to true (`while (cond) stmt`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhileStatement {
    /// Overall source coordinate span.
    pub span: Span,
    /// Evaluated boolean predicate.
    pub condition: Expression,
    /// Statement body repeatedly executed.
    pub body: Box<Statement>,
}

/// Loop statement evaluating its condition after each iteration (`do stmt while (cond);`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DoWhileStatement {
    /// Overall source coordinate span from `do` to `;`.
    pub span: Span,
    /// Statement body executed at least once.
    pub body: Box<Statement>,
    /// Evaluated boolean predicate.
    pub condition: Expression,
}

/// Standard 3-clause for-loop (`for (init; cond; step) body`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForStatement {
    /// Overall source coordinate span.
    pub span: Span,
    /// Optional loop initialization statement.
    pub initializer: Option<Box<Statement>>,
    /// Optional continuation condition.
    pub condition: Option<Expression>,
    /// Optional per-iteration update expression.
    pub update: Option<Expression>,
    /// Repeated loop body statement.
    pub body: Box<Statement>,
}

/// Single case clause within a switch statement (`case <expr>: <statements>`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseClause {
    /// Overall source coordinate span.
    pub span: Span,
    /// Matched constant expression.
    pub case_value: Expression,
    /// Statements executed upon match.
    pub statements: Vec<Statement>,
}

/// Multi-branch pattern switch statement (`switch (expr) { case ...: default: ... }`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwitchStatement {
    /// Overall source coordinate span.
    pub span: Span,
    /// Expression whose value is matched against cases.
    pub discriminant: Expression,
    /// Ordered list of explicit case clauses.
    pub cases: Vec<CaseClause>,
    /// Optional fallback default statements.
    pub default_case: Option<Vec<Statement>>,
}

/// Individual catch clause catching a typed exception in a try block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatchClause {
    /// Overall source coordinate span.
    pub span: Span,
    /// Captured exception type annotation.
    pub exception_type: TypeAnnotation,
    /// Coordinate span of the exception variable name.
    pub variable_name: Span,
    /// Block executed when the exception is caught.
    pub block: BlockStatement,
}

/// Exception-handling block statement (`try { ... } catch (...) { ... } finally { ... }`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TryStatement {
    /// Overall source coordinate span.
    pub span: Span,
    /// Protected try block.
    pub try_block: BlockStatement,
    /// Sequence of catch clauses.
    pub catch_clauses: Vec<CatchClause>,
    /// Optional cleanup finally block.
    pub finally_block: Option<BlockStatement>,
}

/// Throw statement raising an exception (`throw expr;`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThrowStatement {
    /// Overall source coordinate span from `throw` to `;`.
    pub span: Span,
    /// Raised exception expression.
    pub expression: Expression,
}

/// Function or method return statement (`return [expr];`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReturnStatement {
    /// Overall source coordinate span from `return` to `;`.
    pub span: Span,
    /// Optional returned expression value.
    pub value: Option<Expression>,
}

/// Break statement transferring control outside the innermost loop or switch (`break;`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BreakStatement {
    /// Overall source coordinate span from `break` to `;`.
    pub span: Span,
}

/// Continue statement jumping to the next iteration of the innermost loop (`continue;`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContinueStatement {
    /// Overall source coordinate span from `continue` to `;`.
    pub span: Span,
}

/// Variable declaration statement (`[const] Type name [= initializer];`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariableDeclarationStatement {
    /// Overall source coordinate span.
    pub span: Span,
    /// Coordinate span of the variable name identifier for LSP/IntelliSense.
    pub name_span: Span,
    /// Whether this variable was declared with the `const` modifier.
    pub is_const: bool,
    /// Declared type annotation.
    pub var_type: TypeAnnotation,
    /// Optional initialization expression.
    pub initializer: Option<Expression>,
}

/// Statement wrapping a single evaluated expression (`expr;`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionStatement {
    /// Overall source coordinate span including the terminating semicolon.
    pub span: Span,
    /// Wrapped expression.
    pub expression: Expression,
}

/// Assert intrinsic statement verifying a condition (`assert(cond [, message]);`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssertStatement {
    /// Overall source coordinate span from `assert` to `;`.
    pub span: Span,
    /// Evaluated assertion condition.
    pub condition: Expression,
    /// Optional custom diagnostic failure message expression.
    pub message: Option<Expression>,
}

/// Exit statement terminating the execution runtime (`exit(code);`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExitStatement {
    /// Overall source coordinate span from `exit` to `;`.
    pub span: Span,
    /// Exit status code expression.
    pub code: Expression,
}
