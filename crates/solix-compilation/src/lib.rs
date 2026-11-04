#[allow(non_snake_case)]
pub mod CompilationContext;
pub mod builtins;
pub mod manifest;
pub mod phases;
pub mod pipeline;

pub use builtins::{BLT_IO_FILE_NAME, BLT_IO_SOURCE};
pub use manifest::manifest_to_source;
pub use phases::{
    BinderPhase, CompilationPhase, EmitterPhase, LexerPhase, ParserPhase, PhaseDecision,
};
pub use pipeline::CompilationPipeline;
