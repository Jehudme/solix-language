//! Lexical token definitions, source coordinates, and keyword mappings re-exported from `solix_common`.

#![forbid(unsafe_code)]

pub use solix_common::span::Span;
pub use solix_common::tokens::{KEYWORDS, Token, TokenKind, lookup_keyword};
