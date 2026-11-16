#[allow(non_snake_case)]
pub mod CompilationContext;
pub mod diagnostics;
pub mod manifest;
pub mod phases;
pub mod pipeline;
pub mod telemetry;

pub use CompilationContext::{BuildProfile, CompilationOptions, LinkMode};
pub use diagnostics::{
    BindingError, CompilationDiagnostic, EmissionError, LexingError, LocatedDiagnostic,
    ParsingError,
};
pub use manifest::manifest_to_source;
pub use phases::{
    BinderPhase, CompilationPhase, EmitterPhase, LexerPhase, ParserPhase, PhaseDecision,
    parse_context_resilient, parse_source_resilient,
};
pub use pipeline::CompilationPipeline;
pub use telemetry::{CompilerTelemetry, PhaseMetric};
