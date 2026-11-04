//! Standard compiler diagnostic hierarchies and error classifications.

pub mod binding;
pub mod compilation;
pub mod emission;
pub mod lexing;
pub mod parsing;

pub use binding::BindingError;
pub use compilation::CompilationDiagnostic;
pub use emission::EmissionError;
pub use lexing::LexingError;
pub use parsing::ParsingError;
