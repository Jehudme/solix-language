//! Type declarations, class members, methods, constructors, and operator overloads.

#![forbid(unsafe_code)]

use crate::phases::lexing::{Span, TokenKind};
use crate::phases::parser::statements::control_flow::BlockStatement;
use crate::phases::parser::statements::expressions::Expression;
use crate::phases::parser::statements::types::TypeAnnotation;
use serde::{Deserialize, Serialize};

/// Visibility and behavioral declaration modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Modifier {
    /// Public accessibility (`public`).
    Public,
    /// Protected accessibility (`protected`).
    Protected,
    /// Private accessibility (`private`).
    Private,
    /// Internal / package accessibility (`internal`).
    Internal,
    /// Static class member (`static`).
    Static,
    /// Inlined function or method (`inline`).
    Inline,
    /// Native FFI implementation (`native`).
    Native,
    /// Constant immutable declaration (`const`).
    Const,
    /// Polymorphic virtual method (`virtual`).
    Virtual,
    /// Override of virtual superclass method (`override`).
    Override,
    /// Weak reference pointer (`weak`).
    Weak,
    /// Abstract class or method signature (`abstract`).
    Abstract,
    /// Extern declaration for dynamic linking (`extern`).
    Extern,
    /// Non-extensible class or non-overridable method (`final`).
    Final,
}

impl Modifier {
    /// Returns the canonical lowercase keyword representation of this modifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Protected => "protected",
            Self::Private => "private",
            Self::Internal => "internal",
            Self::Static => "static",
            Self::Inline => "inline",
            Self::Native => "native",
            Self::Const => "const",
            Self::Virtual => "virtual",
            Self::Override => "override",
            Self::Weak => "weak",
            Self::Abstract => "abstract",
            Self::Extern => "extern",
            Self::Final => "final",
        }
    }
}

/// Generic type parameter definition with optional type bounds.
///
/// Syntax: `<T>` or `<T : BaseType>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenericParameter {
    /// Overall source coordinate span.
    pub span: Span,
    /// Identifier coordinate span for the type parameter name.
    pub name_span: Span,
    /// Type constraint bounds (if any).
    pub bounds: Vec<TypeAnnotation>,
    /// Whether this is a variadic parameter pack (`<T...>`).
    pub is_variadic: bool,
}

/// Formal parameter declaration within a function, method, or constructor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    /// Overall source coordinate span covering type, name, and optional default.
    pub span: Span,
    /// Identifier coordinate span of the parameter name for LSP navigation.
    pub name_span: Span,
    /// Declared parameter type annotation.
    pub param_type: TypeAnnotation,
    /// Optional default argument expression.
    pub default_value: Option<Expression>,
    /// Whether this is a variadic pack parameter (`...`).
    pub is_variadic: bool,
}

/// Enumeration variant member (`Variant` or `Variant = 42`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumVariant {
    /// Overall source coordinate span of the variant.
    pub span: Span,
    /// Identifier coordinate span for the variant name.
    pub name_span: Span,
    /// Optional explicit integer constant expression.
    pub explicit_value: Option<Expression>,
}

/// Field member declared inside a class or struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldDeclaration {
    /// Overall source coordinate span from modifiers to `;`.
    pub span: Span,
    /// Leading namespace qualification path segments (e.g. `["solix", "math"]` for `solix::math::PI`).
    pub namespace_path: Vec<Span>,
    /// Identifier coordinate span for the field name.
    pub name_span: Span,
    /// Access and storage modifiers.
    pub modifiers: Vec<Modifier>,
    /// Declared field type.
    pub field_type: TypeAnnotation,
    /// Optional initial value assignment.
    pub initializer: Option<Expression>,
    /// Optional resolved symbol identifier populated by the semantic binder.
    pub resolved_symbol_id: Option<u32>,
}

/// Method or function declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MethodDeclaration {
    /// Overall source coordinate span from modifiers to closing `}` or `;`.
    pub span: Span,
    /// Leading namespace qualification path segments (e.g. `["solix", "math"]` for `solix::math::sin`).
    pub namespace_path: Vec<Span>,
    /// Identifier coordinate span of the method name for LSP symbol search.
    pub name_span: Span,
    /// Access and execution modifiers (`public`, `static`, etc.).
    pub modifiers: Vec<Modifier>,
    /// Return type annotation.
    pub return_type: TypeAnnotation,
    /// Generic type parameters declared on the method (`<T, U>`).
    pub generic_params: Vec<GenericParameter>,
    /// Formal parameter declarations.
    pub parameters: Vec<Parameter>,
    /// Method execution body (or None if `abstract` / `native` / interface method).
    pub body: Option<BlockStatement>,
    /// Optional resolved symbol identifier populated by the semantic binder.
    pub resolved_symbol_id: Option<u32>,
}

/// Class constructor declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstructorDeclaration {
    /// Overall source coordinate span.
    pub span: Span,
    /// Identifier coordinate span matching the class name.
    pub name_span: Span,
    /// Modifiers applied to the constructor.
    pub modifiers: Vec<Modifier>,
    /// Formal constructor parameters.
    pub parameters: Vec<Parameter>,
    /// Optional super-constructor invocation arguments (`: super(...)`).
    pub super_call_args: Option<Vec<Expression>>,
    /// Constructor body block.
    pub body: BlockStatement,
}

/// Class destructor / cleanup declaration (`~ClassName() { ... }`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DestructorDeclaration {
    /// Overall source coordinate span from `~` to `}`.
    pub span: Span,
    /// Identifier coordinate span of the destructor name.
    pub name_span: Span,
    /// Destructor body block.
    pub body: BlockStatement,
}

/// Operator overload method declaration (`operator +(...) -> RetType { ... }`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperatorDeclaration {
    /// Overall source coordinate span.
    pub span: Span,
    /// Coordinate span of the operator token being overloaded.
    pub operator_span: Span,
    /// Overloaded operator terminal token kind.
    pub operator_token: TokenKind,
    /// Modifiers applied to the operator.
    pub modifiers: Vec<Modifier>,
    /// Formal parameters (typically 1 for binary rhs, 0 for unary).
    pub parameters: Vec<Parameter>,
    /// Overloaded operator return type annotation.
    pub return_type: TypeAnnotation,
    /// Operator body block.
    pub body: BlockStatement,
}

/// Discrete member declared within a class body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClassMember {
    /// Field declaration.
    Field(FieldDeclaration),
    /// Method declaration.
    Method(MethodDeclaration),
    /// Constructor declaration.
    Constructor(ConstructorDeclaration),
    /// Destructor declaration.
    Destructor(DestructorDeclaration),
    /// Operator overload declaration.
    Operator(OperatorDeclaration),
    /// Nested enum declaration.
    Enum(EnumDeclaration),
    /// Nested class declaration.
    Class(ClassDeclaration),
    /// Nested interface declaration.
    Interface(InterfaceDeclaration),
}

/// Class type declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassDeclaration {
    /// Overall source coordinate span from modifiers to closing `}`.
    pub span: Span,
    /// Leading namespace qualification path segments (e.g. `["solix", "io"]` for `solix::io::Console`).
    pub namespace_path: Vec<Span>,
    /// Identifier coordinate span for the class name.
    pub name_span: Span,
    /// Class modifiers (`public`, `abstract`, etc.).
    pub modifiers: Vec<Modifier>,
    /// Generic type parameters (`<T, U>`).
    pub generic_params: Vec<GenericParameter>,
    /// Optional base class inherited via `extends <Super>`.
    pub super_class: Option<TypeAnnotation>,
    /// List of interfaces implemented via `implements <Interface>`.
    pub interfaces: Vec<TypeAnnotation>,
    /// Members defined inside the class body.
    pub members: Vec<ClassMember>,
    /// Optional resolved symbol identifier populated by the semantic binder.
    pub resolved_symbol_id: Option<u32>,
}

/// Interface type declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterfaceDeclaration {
    /// Overall source coordinate span.
    pub span: Span,
    /// Leading namespace qualification path segments (e.g. `["solix", "io"]` for `solix::io::IStream`).
    pub namespace_path: Vec<Span>,
    /// Identifier coordinate span for the interface name.
    pub name_span: Span,
    /// Interface modifiers (`public`, etc.).
    pub modifiers: Vec<Modifier>,
    /// Generic type parameters.
    pub generic_params: Vec<GenericParameter>,
    /// Base interfaces inherited via `extends <SuperInterface>`.
    pub super_interfaces: Vec<TypeAnnotation>,
    /// Method signatures defined in the interface.
    pub methods: Vec<MethodDeclaration>,
    /// Optional resolved symbol identifier populated by the semantic binder.
    pub resolved_symbol_id: Option<u32>,
}

/// Enumeration type declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumDeclaration {
    /// Overall source coordinate span.
    pub span: Span,
    /// Leading namespace qualification path segments (e.g. `["solix", "io"]` for `solix::io::FileMode`).
    pub namespace_path: Vec<Span>,
    /// Identifier coordinate span for the enum name.
    pub name_span: Span,
    /// Enum modifiers.
    pub modifiers: Vec<Modifier>,
    /// Enumeration variants.
    pub variants: Vec<EnumVariant>,
    /// Optional resolved symbol identifier populated by the semantic binder.
    pub resolved_symbol_id: Option<u32>,
}

/// High-level declaration variants permitted in Solix programs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Declaration {
    /// Class type declaration.
    Class(ClassDeclaration),
    /// Interface type declaration.
    Interface(InterfaceDeclaration),
    /// Enum type declaration.
    Enum(EnumDeclaration),
    /// Global or top-level field declaration.
    Field(FieldDeclaration),
    /// Global or top-level function/method declaration.
    Method(MethodDeclaration),
    /// Constructor declaration.
    Constructor(ConstructorDeclaration),
    /// Destructor declaration.
    Destructor(DestructorDeclaration),
    /// Operator overload declaration.
    Operator(OperatorDeclaration),
}

impl Declaration {
    /// Returns the overall source coordinate span of this declaration.
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Self::Class(decl) => decl.span,
            Self::Interface(decl) => decl.span,
            Self::Enum(decl) => decl.span,
            Self::Field(decl) => decl.span,
            Self::Method(decl) => decl.span,
            Self::Constructor(decl) => decl.span,
            Self::Destructor(decl) => decl.span,
            Self::Operator(decl) => decl.span,
        }
    }

    /// Returns the identifier coordinate span of this declaration's name.
    #[must_use]
    pub const fn name_span(&self) -> Span {
        match self {
            Self::Class(decl) => decl.name_span,
            Self::Interface(decl) => decl.name_span,
            Self::Enum(decl) => decl.name_span,
            Self::Field(decl) => decl.name_span,
            Self::Method(decl) => decl.name_span,
            Self::Constructor(decl) => decl.name_span,
            Self::Destructor(decl) => decl.name_span,
            Self::Operator(decl) => decl.operator_span,
        }
    }
}
