//! Strongly-typed AST representations for Solix type annotations and signatures.

#![forbid(unsafe_code)]

use crate::phases::lexing::{Span, TokenKind};

/// Represents a parsed type annotation in Solix source code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeAnnotation {
    /// Primitive type keyword (`void`, `bool`, `int8`..`int64`, `uint8`..`uint64`, `float32`, `float64`, `char`).
    Primitive {
        /// Source coordinate span of the primitive keyword.
        span: Span,
        /// The terminal primitive token kind.
        kind: TokenKind,
    },
    /// Named reference type or user-defined class/interface/enum with an optional qualified path.
    Named {
        /// Overall coordinate span covering the entire path.
        span: Span,
        /// Qualified namespace segments (e.g., `["std", "collections", "List"]`).
        path: Vec<Span>,
    },
    /// Generic parameterized type (e.g., `List<int32>`, `Map<string, User>`).
    Generic {
        /// Overall coordinate span including type arguments.
        span: Span,
        /// The base parameterized type.
        base: Box<TypeAnnotation>,
        /// Type arguments provided between angle brackets `<...>`.
        arguments: Vec<TypeAnnotation>,
    },
    /// Multidimensional or single array type (e.g., `int32[]`, `string[][]`).
    Array {
        /// Overall coordinate span including array brackets.
        span: Span,
        /// The base element type.
        element_type: Box<TypeAnnotation>,
        /// Depth of array nesting (e.g., 1 for `T[]`, 2 for `T[][]`).
        depth: usize,
    },
    /// Strongly-typed tuple type (e.g., `tuple<int32, string, bool>`).
    Tuple {
        /// Overall coordinate span of the tuple annotation.
        span: Span,
        /// Types of each element in the tuple.
        elements: Vec<TypeAnnotation>,
    },
    /// First-class function pointer signature: `RetType(*)(ParamType1, ParamType2)`.
    FunctionPointer {
        /// Overall coordinate span of the function pointer type signature.
        span: Span,
        /// Return type of the referenced function.
        return_type: Box<TypeAnnotation>,
        /// Parameter types accepted by the referenced function.
        parameters: Vec<TypeAnnotation>,
    },
}

impl TypeAnnotation {
    /// Returns the source coordinate span of this type annotation.
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Self::Primitive { span, .. }
            | Self::Named { span, .. }
            | Self::Generic { span, .. }
            | Self::Array { span, .. }
            | Self::Tuple { span, .. }
            | Self::FunctionPointer { span, .. } => *span,
        }
    }
}
