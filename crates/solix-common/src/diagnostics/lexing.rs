//! Lexical analysis errors emitted during tokenization.

#![forbid(unsafe_code)]

use crate::tokens::Token;
use miette::{Diagnostic, SourceSpan};

/// Structured error emitted during lexical tokenization when an unknown or illegal token is encountered.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Diagnostic)]
pub enum LexingError {
    #[error("Unknown or unrecognized token")]
    #[diagnostic(
        code(solix::lexer::unknown_token),
        help("Remove or replace this unrecognized character with valid Solix syntax.")
    )]
    UnknownToken {
        token: Token,
        #[label("unrecognized token")]
        span: SourceSpan,
    },

    #[error("Unterminated string literal")]
    #[diagnostic(
        code(solix::lexer::unterminated_string),
        help("Add a closing quote '\"' to terminate the string literal before end of line or file.")
    )]
    UnterminatedStringLiteral {
        token: Token,
        #[label("unterminated string literal")]
        span: SourceSpan,
    },

    #[error("Unterminated character literal")]
    #[diagnostic(
        code(solix::lexer::unterminated_char),
        help("Add a closing quote '\\'' to terminate the character literal before end of line.")
    )]
    UnterminatedCharacterLiteral {
        token: Token,
        #[label("unterminated character literal")]
        span: SourceSpan,
    },

    #[error("Unterminated block comment")]
    #[diagnostic(
        code(solix::lexer::unterminated_comment),
        help("Add a closing '*/' before end of file.")
    )]
    UnterminatedBlockComment {
        token: Token,
        #[label("unterminated block comment starts here")]
        span: SourceSpan,
    },

    #[error("Malformed hexadecimal integer literal")]
    #[diagnostic(
        code(solix::lexer::malformed_hex),
        help("Hexadecimal literals must have one or more hex digits (0-9, a-f) after '0x'.")
    )]
    MalformedHexLiteral {
        token: Token,
        #[label("missing hex digits")]
        span: SourceSpan,
    },

    #[error("Malformed binary integer literal")]
    #[diagnostic(
        code(solix::lexer::malformed_binary),
        help("Binary literals must have one or more binary digits (0 or 1) after '0b'.")
    )]
    MalformedBinaryLiteral {
        token: Token,
        #[label("missing binary digits")]
        span: SourceSpan,
    },
}

impl LexingError {
    /// Constructs a new [`LexingError::UnknownToken`] from a token.
    #[must_use]
    pub fn unknown_token(token: Token) -> Self {
        Self::UnknownToken {
            token,
            span: token.span.to_source_span(),
        }
    }

    /// Constructs a new [`LexingError::UnterminatedStringLiteral`] from a token.
    #[must_use]
    pub fn unterminated_string(token: Token) -> Self {
        Self::UnterminatedStringLiteral {
            token,
            span: token.span.to_source_span(),
        }
    }

    /// Constructs a new [`LexingError::UnterminatedCharacterLiteral`] from a token.
    #[must_use]
    pub fn unterminated_char(token: Token) -> Self {
        Self::UnterminatedCharacterLiteral {
            token,
            span: token.span.to_source_span(),
        }
    }

    /// Constructs a new [`LexingError::UnterminatedBlockComment`] from a token.
    #[must_use]
    pub fn unterminated_comment(token: Token) -> Self {
        Self::UnterminatedBlockComment {
            token,
            span: token.span.to_source_span(),
        }
    }

    /// Constructs a new [`LexingError::MalformedHexLiteral`] from a token.
    #[must_use]
    pub fn malformed_hex(token: Token) -> Self {
        Self::MalformedHexLiteral {
            token,
            span: token.span.to_source_span(),
        }
    }

    /// Constructs a new [`LexingError::MalformedBinaryLiteral`] from a token.
    #[must_use]
    pub fn malformed_binary(token: Token) -> Self {
        Self::MalformedBinaryLiteral {
            token,
            span: token.span.to_source_span(),
        }
    }

    /// Returns the underlying token associated with this error.
    #[must_use]
    pub const fn token(&self) -> Token {
        match self {
            Self::UnknownToken { token, .. }
            | Self::UnterminatedStringLiteral { token, .. }
            | Self::UnterminatedCharacterLiteral { token, .. }
            | Self::UnterminatedBlockComment { token, .. }
            | Self::MalformedHexLiteral { token, .. }
            | Self::MalformedBinaryLiteral { token, .. } => *token,
        }
    }

    /// Returns the standardized diagnostic error code string.
    #[must_use]
    pub const fn error_code(&self) -> &'static str {
        match self {
            Self::UnknownToken { .. } => "E0000",
            Self::UnterminatedStringLiteral { .. } => "E0011",
            Self::UnterminatedCharacterLiteral { .. } => "E0012",
            Self::UnterminatedBlockComment { .. } => "E0013",
            Self::MalformedHexLiteral { .. } => "E0014",
            Self::MalformedBinaryLiteral { .. } => "E0015",
        }
    }

    /// Returns the source span for this diagnostic error.
    #[must_use]
    pub const fn source_span(&self) -> SourceSpan {
        match self {
            Self::UnknownToken { span, .. }
            | Self::UnterminatedStringLiteral { span, .. }
            | Self::UnterminatedCharacterLiteral { span, .. }
            | Self::UnterminatedBlockComment { span, .. }
            | Self::MalformedHexLiteral { span, .. }
            | Self::MalformedBinaryLiteral { span, .. } => *span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;
    use crate::tokens::TokenKind;

    #[test]
    fn test_positive_lexing_error_constructors() {
        let dummy_span = Span::new(0, 5, 10);
        let dummy_token = Token::new(TokenKind::Unknown, dummy_span);

        let error_unknown = LexingError::unknown_token(dummy_token);
        assert_eq!(error_unknown.error_code(), "E0000");
        assert_eq!(error_unknown.token(), dummy_token);

        let error_str = LexingError::unterminated_string(dummy_token);
        assert_eq!(error_str.error_code(), "E0011");

        let error_char = LexingError::unterminated_char(dummy_token);
        assert_eq!(error_char.error_code(), "E0012");

        let error_comment = LexingError::unterminated_comment(dummy_token);
        assert_eq!(error_comment.error_code(), "E0013");

        let error_hex = LexingError::malformed_hex(dummy_token);
        assert_eq!(error_hex.error_code(), "E0014");

        let error_binary = LexingError::malformed_binary(dummy_token);
        assert_eq!(error_binary.error_code(), "E0015");
    }

    #[test]
    fn test_negative_error_message_no_sourcespan_debug() {
        let dummy_span = Span::new(0, 10, 20);
        let dummy_token = Token::new(TokenKind::Unknown, dummy_span);
        let error = LexingError::unknown_token(dummy_token);
        let message = error.to_string();
        assert!(!message.contains("SourceSpan"));
        assert!(!message.contains("offset"));
    }
}
