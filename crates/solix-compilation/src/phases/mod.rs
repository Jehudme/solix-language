pub mod binder;
pub mod dependency;
pub mod emitter;
pub mod export_api;
pub mod lexing;
pub mod optimizer;
pub mod parser;
pub mod phase;
pub mod standard;

pub use dependency::DependencyResolutionPhase;
pub use export_api::{generate_api_header, generate_api_header_and_ast};
pub use optimizer::{AstInlinerPhase, BytecodeOptimizerPhase};
pub use parser::{parse_context, parse_context_resilient, parse_source, parse_source_resilient};
pub use phase::{CompilationPhase, PhaseDecision};
pub use standard::{BinderPhase, EmitterPhase, LexerPhase, ParserPhase};
