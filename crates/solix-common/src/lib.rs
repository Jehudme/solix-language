//! Solix common types, source coordinate spans, and shared diagnostic error models.

#![forbid(unsafe_code)]

pub mod bytecode;
pub mod diagnostics;
pub mod project;
pub mod solver;
pub mod span;
pub mod tokens;
pub mod types;

pub use bytecode::{
    BinaryProgram, CompiledClass, CompiledFunction, ConstantPool, ConstantPoolValue,
    DeclarationModifier, ExceptionHandlerEntry, ExportManifest, ExportedClass, ExportedField,
    ExportedFunction, ExportedGenericTemplate, ExportedParameter, ExportedStatic, Instruction,
    OpCode, SourceMapEntry, extract_api_manifest,
};
pub use diagnostics::EmissionError;
pub use project::{
    LoadedSlxarc, ProjectEntryPoints, ProjectInfo, ProjectManifest, SlxarcArchive, SlxarcFileEntry,
    SlxarcInspection,
};
pub use solver::{
    DependencyRequirement, InMemoryModuleSolver, ModuleSolver, PackageVersion, SolvedPackage,
    SolverError,
};
pub use span::Span;
pub use tokens::{KEYWORDS, Token, TokenKind, lookup_keyword};
pub use types::{PrimitiveKind, Type};
