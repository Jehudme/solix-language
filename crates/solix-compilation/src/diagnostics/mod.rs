//! Standard compiler diagnostic hierarchies and error classifications.

pub mod binding;
pub mod compilation;
pub mod lexing;
pub mod parsing;

pub use binding::BindingError;
pub use compilation::{CompilationDiagnostic, LocatedDiagnostic};
pub use lexing::LexingError;
pub use parsing::ParsingError;
pub use solix_common::diagnostics::EmissionError;
