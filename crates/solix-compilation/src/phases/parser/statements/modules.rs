//! Module and compilation-unit level statements (packages, imports, and aliases).

#![forbid(unsafe_code)]

use crate::phases::lexing::Span;
use crate::phases::parser::statements::types::TypeAnnotation;

/// Package declaration statement indicating the namespace of the compilation unit.
///
/// Syntax: `package my.organization.pkg;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageStatement {
    /// Overall source coordinate span from `package` keyword to `;`.
    pub span: Span,
    /// Identifier segments composing the package path.
    pub path: Vec<Span>,
}

/// Import statement importing symbols from external modules or packages.
///
/// Syntax:
/// - `import std.collections.List;`
/// - `import std.math.*;`
/// - `import std.io.File as SolixFile;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportStatement {
    /// Overall source coordinate span from `import` keyword to `;`.
    pub span: Span,
    /// Qualified path segments of the import target.
    pub path: Vec<Span>,
    /// Whether this import ends with a wildcard `*`.
    pub is_wildcard: bool,
    /// Optional rename alias specified via `as <Alias>`.
    pub alias: Option<Span>,
    /// Whether this import was declared with `extern import`.
    pub is_extern: bool,
}

/// Type alias statement defining a synonym for a type or generic parameterization.
///
/// Syntax:
/// - `alias Byte = uint8;`
/// - `alias StringMap<V> = Map<string, V>;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliasStatement {
    /// Overall source coordinate span from `alias` keyword to `;`.
    pub span: Span,
    /// Coordinate span of the declared alias identifier name.
    pub name_span: Span,
    /// Optional generic type parameters defined on this alias.
    pub generic_params: Vec<Span>,
    /// The target type annotation to which this alias refers.
    pub target_type: TypeAnnotation,
}
