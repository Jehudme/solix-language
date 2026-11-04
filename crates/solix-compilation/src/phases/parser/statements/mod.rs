//! Group-partitioned Abstract Syntax Tree (AST) definitions for Solix.

#![forbid(unsafe_code)]

pub mod control_flow;
pub mod declarations;
pub mod expressions;
pub mod modules;
pub mod types;

pub use control_flow::*;
pub use declarations::*;
pub use expressions::*;
pub use modules::*;
pub use types::*;

use crate::phases::lexing::Span;

/// High-level statement variants executable within blocks or compilation units.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// Compound block statement.
    Block(BlockStatement),
    /// Conditional if-else branch statement.
    If(IfStatement),
    /// Loop while statement.
    While(WhileStatement),
    /// Loop do-while statement.
    DoWhile(DoWhileStatement),
    /// Loop for statement.
    For(ForStatement),
    /// Switch-case pattern statement.
    Switch(SwitchStatement),
    /// Exception handling try-catch-finally statement.
    Try(TryStatement),
    /// Exception throw statement.
    Throw(ThrowStatement),
    /// Function/method return statement.
    Return(ReturnStatement),
    /// Loop break statement.
    Break(BreakStatement),
    /// Loop continue statement.
    Continue(ContinueStatement),
    /// Local variable binding declaration.
    VariableDeclaration(VariableDeclarationStatement),
    /// Bare expression evaluation statement.
    Expression(ExpressionStatement),
    /// Assertion statement.
    Assert(AssertStatement),
    /// Process exit statement.
    Exit(ExitStatement),
    /// Nested local declaration statement.
    Declaration(Declaration),
}

impl Statement {
    /// Returns the overall source coordinate span of this statement.
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Self::Block(stmt) => stmt.span,
            Self::If(stmt) => stmt.span,
            Self::While(stmt) => stmt.span,
            Self::DoWhile(stmt) => stmt.span,
            Self::For(stmt) => stmt.span,
            Self::Switch(stmt) => stmt.span,
            Self::Try(stmt) => stmt.span,
            Self::Throw(stmt) => stmt.span,
            Self::Return(stmt) => stmt.span,
            Self::Break(stmt) => stmt.span,
            Self::Continue(stmt) => stmt.span,
            Self::VariableDeclaration(stmt) => stmt.span,
            Self::Expression(stmt) => stmt.span,
            Self::Assert(stmt) => stmt.span,
            Self::Exit(stmt) => stmt.span,
            Self::Declaration(decl) => decl.span(),
        }
    }
}

/// Root Abstract Syntax Tree container representing a complete parsed compilation unit.
#[derive(Debug, Clone, PartialEq)]
pub struct ProgramAst {
    /// Identifier indexing the originating source file in `CompilationContext`.
    pub source_id: u16,
    /// Overall source coordinate span of the compilation unit.
    pub span: Span,
    /// Optional package declaration statement.
    pub package: Option<PackageStatement>,
    /// List of import statements declared at the head of the file.
    pub imports: Vec<ImportStatement>,
    /// List of type aliases declared in this compilation unit.
    pub aliases: Vec<AliasStatement>,
    /// Top-level type and member declarations (classes, interfaces, enums, functions).
    pub declarations: Vec<Declaration>,
    /// Top-level executable statements (for script-style execution or top-level logic).
    pub root_statements: Vec<Statement>,
}

impl ProgramAst {
    /// Creates a new empty ProgramAst for a given source ID.
    #[must_use]
    pub const fn new(source_id: u16, span: Span) -> Self {
        Self {
            source_id,
            span,
            package: None,
            imports: Vec::new(),
            aliases: Vec::new(),
            declarations: Vec::new(),
            root_statements: Vec::new(),
        }
    }
}
