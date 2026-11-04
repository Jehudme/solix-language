//! Two-stage optimization pipeline: AST inlining and bytecode peephole optimization.

#![forbid(unsafe_code)]

pub mod ast_inliner;

pub use ast_inliner::AstInlinerPhase;
