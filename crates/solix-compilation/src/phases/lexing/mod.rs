//! Lexical analysis phase module.

pub mod lexer;
pub mod tokens;

pub use lexer::{LexingError, MetaData, lex, lex_context, lex_source, tokenize_source};
pub use tokens::{KEYWORDS, Span, Token, TokenKind, lookup_keyword};
