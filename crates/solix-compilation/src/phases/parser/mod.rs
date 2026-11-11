//! Syntactic analysis (parser) phase module.

#![forbid(unsafe_code)]
#![allow(clippy::module_inception)]

pub mod errors;
pub mod parser;
pub mod statements;
pub mod visitor;

pub use errors::ParsingError;
pub use parser::{
    parse, parse_context, parse_context_resilient, parse_source, parse_source_resilient,
    AstMetaData,
};
pub use statements::*;
pub use visitor::{AstVisitor, Visitable};
