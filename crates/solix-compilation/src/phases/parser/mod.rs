//! Syntactic analysis (parser) phase module.

#![forbid(unsafe_code)]
#![allow(clippy::module_inception)]

pub mod errors;
pub mod parser;
pub mod statements;
pub mod visitor;

pub use errors::ParsingError;
pub use parser::{AstMetaData, parse, parse_context, parse_source};
pub use statements::*;
pub use visitor::{AstVisitor, Visitable};
