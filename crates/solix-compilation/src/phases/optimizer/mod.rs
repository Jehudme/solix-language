//! Two-stage optimization pipeline: AST inlining and bytecode peephole optimization.

#![forbid(unsafe_code)]

pub mod ast_inliner;
pub mod bytecode_optimizer;

pub use ast_inliner::AstInlinerPhase;
pub use bytecode_optimizer::BytecodeOptimizerPhase;
