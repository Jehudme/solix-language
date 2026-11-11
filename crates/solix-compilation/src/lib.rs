#[allow(non_snake_case)]
pub mod CompilationContext;
pub mod builtins;
pub mod diagnostics;
pub mod manifest;
pub mod phases;
pub mod pipeline;
pub mod telemetry;

pub use CompilationContext::{BuildProfile, CompilationOptions, LinkMode};
pub use builtins::{BLT_IO_FILE_NAME, BLT_IO_SOURCE};
pub use diagnostics::{
    BindingError, CompilationDiagnostic, EmissionError, LexingError, LocatedDiagnostic,
    ParsingError,
};
pub use manifest::manifest_to_source;
pub use phases::{
    BinderPhase, CompilationPhase, EmitterPhase, LexerPhase, ParserPhase, PhaseDecision,
};
pub use pipeline::CompilationPipeline;
pub use telemetry::{CompilerTelemetry, PhaseMetric};
