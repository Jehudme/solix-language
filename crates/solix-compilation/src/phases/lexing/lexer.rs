//! High-performance lexical analyzer implementation for the Solix compiler.

#![forbid(unsafe_code)]

use crate::CompilationContext::CompilationContext;
pub use crate::LexingError;
use crate::phases::lexing::tokens::{Span, Token, TokenKind, lookup_keyword};
use std::time::{Duration, Instant};

/// Metadata and statistics summarizing the completion of the lexical analysis phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MetaData {
    /// Total count of tokens emitted across all source files.
    pub tokens_count: usize,
    /// Number of source files processed.
    pub sources_processed: usize,
    /// Total count of source bytes scanned.
    pub bytes_processed: usize,
    /// Total wall-clock duration of the lexing phase.
    pub duration: Duration,
}

/// Tokenizes all source files registered in the compilation context.
///
/// Unless an [`TokenKind::Unknown`] token is encountered, the lexer will not emit errors.
/// Higher-level literal validations (such as character count or numeric overflow) are deferred to the parser.
///
/// # Errors
/// Returns a list of accumulated [`LexingError`] diagnostics if any unknown tokens were encountered.
pub fn lex_context(context: &mut CompilationContext) -> Result<MetaData, Vec<LexingError>> {
    let start_time = Instant::now();
    let mut total_tokens = 0;
    let mut total_bytes = 0;
    let mut accumulated_errors = Vec::new();

    let sources_count = context.sources.len();
    context.tokens.clear();
    context.tokens.resize(sources_count, Vec::new());

    for source_index in 0..sources_count {
        let source_id = source_index as u16;
        let source_text = &context.sources[source_index];
        total_bytes += source_text.len();

        let (tokens, errors) = tokenize_source(source_id, source_text);
        total_tokens += tokens.len();
        context.tokens[source_index] = tokens;

        accumulated_errors.extend(errors);
    }

    if !accumulated_errors.is_empty() {
        return Err(accumulated_errors);
    }

    Ok(MetaData {
        tokens_count: total_tokens,
        sources_processed: sources_count,
        bytes_processed: total_bytes,
        duration: start_time.elapsed(),
    })
}

/// Convenience alias for [`lex_context`].
#[inline]
pub fn lex(context: &mut CompilationContext) -> Result<MetaData, Vec<LexingError>> {
    lex_context(context)
}

/// Convenience function tokenizing an individual source string into a token stream.
///
/// # Errors
/// Returns a list of accumulated [`LexingError`] diagnostics if unknown tokens were encountered.
pub fn lex_source(source_id: u16, source_text: &str) -> Result<Vec<Token>, Vec<LexingError>> {
    let (tokens, errors) = tokenize_source(source_id, source_text);
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(tokens)
}

/// Internal stateful tokenizer scanning a single source buffer.
struct LexerState<'source> {
    source: &'source str,
    bytes: &'source [u8],
    source_id: u16,
    cursor: usize,
    start_cursor: usize,
    tokens: Vec<Token>,
    errors: Vec<LexingError>,
}

impl<'source> LexerState<'source> {
    fn new(source_id: u16, source: &'source str) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            source_id,
            cursor: 0,
            start_cursor: 0,
            tokens: Vec::with_capacity(source.len() / 8 + 16),
            errors: Vec::new(),
        }
    }

    fn is_at_end(&self) -> bool {
        self.cursor >= self.bytes.len()
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.cursor).copied()
    }

    fn peek_next(&self) -> Option<u8> {
        self.bytes.get(self.cursor + 1).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let byte = self.bytes.get(self.cursor).copied()?;
        self.cursor += 1;
        Some(byte)
    }

    fn match_byte(&mut self, expected: u8) -> bool {
        if self.is_at_end() || self.bytes[self.cursor] != expected {
            return false;
        }
        self.cursor += 1;
        true
    }

    fn make_span(&self) -> Span {
        Span::new(self.source_id, self.start_cursor as u32, self.cursor as u32)
    }

    fn add_token(&mut self, kind: TokenKind) -> Token {
        let token = Token::new(kind, self.make_span());
        self.tokens.push(token);
        token
    }

    fn add_unknown_token(&mut self) {
        let token = self.add_token(TokenKind::Unknown);
        self.errors.push(LexingError::unknown_token(token));
    }

    fn handle_identifier(&mut self) {
        while let Some(byte) = self.peek() {
            if byte.is_ascii_alphanumeric() || byte == b'_' {
                self.advance();
            } else {
                break;
            }
        }

        let identifier_lexeme = &self.source[self.start_cursor..self.cursor];
        if let Some(keyword_kind) = lookup_keyword(identifier_lexeme) {
            self.add_token(keyword_kind);
        } else {
            self.add_token(TokenKind::Identifier);
        }
    }

    fn handle_number(&mut self, first_byte: u8) {
        // Hexadecimal literal: 0x... or 0X...
        if first_byte == b'0' && matches!(self.peek(), Some(b'x' | b'X')) {
            self.advance(); // consume 'x' or 'X'
            let hex_start = self.cursor;
            while let Some(byte) = self.peek() {
                if byte.is_ascii_hexdigit() || byte == b'_' {
                    self.advance();
                } else {
                    break;
                }
            }

            if self.cursor == hex_start {
                let token = self.add_token(TokenKind::Number);
                self.errors.push(LexingError::malformed_hex(token));
                return;
            }

            // Optional integer suffix L / l
            if matches!(self.peek(), Some(b'L' | b'l')) {
                self.advance();
            }

            self.add_token(TokenKind::Number);
            return;
        }

        // Binary literal: 0b... or 0B...
        if first_byte == b'0' && matches!(self.peek(), Some(b'b' | b'B')) {
            self.advance(); // consume 'b' or 'B'
            let binary_start = self.cursor;
            while let Some(byte) = self.peek() {
                if matches!(byte, b'0' | b'1' | b'_') {
                    self.advance();
                } else {
                    break;
                }
            }

            if self.cursor == binary_start {
                let token = self.add_token(TokenKind::Number);
                self.errors.push(LexingError::malformed_binary(token));
                return;
            }

            // Optional integer suffix L / l
            if matches!(self.peek(), Some(b'L' | b'l')) {
                self.advance();
            }

            self.add_token(TokenKind::Number);
            return;
        }

        let has_leading_zero = first_byte == b'0'
            && self
                .peek()
                .is_some_and(|byte| byte.is_ascii_digit() || byte == b'_');

        // Decimal digits
        while let Some(byte) = self.peek() {
            if byte.is_ascii_digit() || byte == b'_' {
                self.advance();
            } else {
                break;
            }
        }

        // Fractional part
        let is_fractional = if self.peek() == Some(b'.') {
            match self.peek_next() {
                Some(b'.') => false,
                Some(next_byte) if next_byte.is_ascii_alphabetic() || next_byte == b'_' => {
                    let is_suffix_f = (next_byte == b'f' || next_byte == b'F')
                        && !self.bytes.get(self.cursor + 2).is_some_and(|after_byte| {
                            after_byte.is_ascii_alphanumeric() || *after_byte == b'_'
                        });
                    let is_exponent = (next_byte == b'e' || next_byte == b'E')
                        && self.bytes.get(self.cursor + 2).is_some_and(|after_byte| {
                            after_byte.is_ascii_digit()
                                || *after_byte == b'+'
                                || *after_byte == b'-'
                        });
                    is_suffix_f || is_exponent
                }
                _ => true,
            }
        } else {
            false
        };

        if is_fractional {
            self.advance(); // consume '.'
            while let Some(byte) = self.peek() {
                if byte.is_ascii_digit() || byte == b'_' {
                    self.advance();
                } else {
                    break;
                }
            }

            // Optional exponent part
            if matches!(self.peek(), Some(b'e' | b'E')) {
                self.advance();
                if matches!(self.peek(), Some(b'+' | b'-')) {
                    self.advance();
                }
                while let Some(byte) = self.peek() {
                    if byte.is_ascii_digit() || byte == b'_' {
                        self.advance();
                    } else {
                        break;
                    }
                }
            }

            // Optional float suffix f / F
            if matches!(self.peek(), Some(b'f' | b'F')) {
                self.advance();
            }

            let token = self.add_token(TokenKind::Number);
            if has_leading_zero {
                self.errors.push(LexingError::leading_zero_integer(token));
            }
            return;
        }

        // Standalone exponent without decimal point (e.g. 1e10)
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.advance();
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.advance();
            }
            while let Some(byte) = self.peek() {
                if byte.is_ascii_digit() || byte == b'_' {
                    self.advance();
                } else {
                    break;
                }
            }

            if matches!(self.peek(), Some(b'f' | b'F')) {
                self.advance();
            }

            let token = self.add_token(TokenKind::Number);
            if has_leading_zero {
                self.errors.push(LexingError::leading_zero_integer(token));
            }
            return;
        }

        // Optional integer suffix L / l
        if matches!(self.peek(), Some(b'L' | b'l')) {
            self.advance();
        }

        let token = self.add_token(TokenKind::Number);
        if has_leading_zero {
            self.errors.push(LexingError::leading_zero_integer(token));
        }
    }

    fn handle_string(&mut self) {
        while let Some(byte) = self.peek() {
            if byte == b'"' {
                self.advance(); // Consume closing quote
                self.add_token(TokenKind::String);
                return;
            }

            if byte == b'\n' || byte == b'\r' {
                // Regular strings cannot span multiple lines
                break;
            }

            if byte == b'\\' {
                self.advance(); // Consume '\'
                if self.is_at_end() {
                    break;
                }
                self.advance(); // Consume escaped byte
            } else {
                self.advance();
            }
        }

        // If loop finished without closing quote, mark as unterminated string
        let token = self.add_token(TokenKind::String);
        self.errors.push(LexingError::unterminated_string(token));
    }

    fn handle_character(&mut self) {
        if self.is_at_end() {
            let token = self.add_token(TokenKind::Char);
            self.errors.push(LexingError::unterminated_char(token));
            return;
        }

        while let Some(byte) = self.peek() {
            if byte == b'\'' {
                self.advance(); // Consume closing quote
                self.add_token(TokenKind::Char);
                return;
            }

            if byte == b'\n' {
                // Character literals cannot span lines
                break;
            }

            if byte == b'\\' {
                self.advance(); // Consume '\'
                if self.is_at_end() {
                    break;
                }
                self.advance(); // Consume escaped byte
            } else {
                self.advance();
            }
        }

        let token = self.add_token(TokenKind::Char);
        self.errors.push(LexingError::unterminated_char(token));
    }
}

/// Tokenizes a single source string, returning the token stream alongside accumulated unknown token errors.
pub fn tokenize_source(source_id: u16, source: &str) -> (Vec<Token>, Vec<LexingError>) {
    let mut state = LexerState::new(source_id, source);

    // Consume optional UTF-8 Byte Order Mark (BOM: 0xEF, 0xBB, 0xBF)
    if state.bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        state.cursor = 3;
    }

    while !state.is_at_end() {
        // Skip whitespace
        if matches!(state.peek(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
            state.advance();
            continue;
        }

        state.start_cursor = state.cursor;
        let current_byte = match state.advance() {
            Some(byte) => byte,
            None => break,
        };

        // Identifiers and Keywords
        if current_byte.is_ascii_alphabetic() || current_byte == b'_' {
            state.handle_identifier();
            continue;
        }

        // Numeric Literals
        if current_byte.is_ascii_digit() {
            state.handle_number(current_byte);
            continue;
        }

        // String Literals
        if current_byte == b'"' {
            state.handle_string();
            continue;
        }

        // Character Literals
        if current_byte == b'\'' {
            state.handle_character();
            continue;
        }

        // Comments, Operators, and Punctuation
        match current_byte {
            // Delimiters & Grouping
            b'(' => {
                state.add_token(TokenKind::PunctuationOpenParen);
            }
            b')' => {
                state.add_token(TokenKind::PunctuationCloseParen);
            }
            b'{' => {
                state.add_token(TokenKind::PunctuationOpenBrace);
            }
            b'}' => {
                state.add_token(TokenKind::PunctuationCloseBrace);
            }
            b'[' => {
                // The [] array brackets optimization
                if state.match_byte(b']') {
                    state.add_token(TokenKind::PunctuationArrayBrackets);
                } else {
                    state.add_token(TokenKind::PunctuationOpenBracket);
                }
            }
            b']' => {
                state.add_token(TokenKind::PunctuationCloseBracket);
            }
            b';' => {
                state.add_token(TokenKind::PunctuationSemicolon);
            }
            b',' => {
                state.add_token(TokenKind::PunctuationComma);
            }
            b'?' => {
                state.add_token(TokenKind::OperatorQuestion);
            }
            b'~' => {
                state.add_token(TokenKind::OperatorBitwiseNot);
            }

            // Dot and Ellipsis
            b'.' => {
                if state.peek() == Some(b'.') && state.peek_next() == Some(b'.') {
                    state.advance();
                    state.advance();
                    state.add_token(TokenKind::PunctuationEllipsis);
                } else {
                    state.add_token(TokenKind::PunctuationDot);
                }
            }

            // Colon and Double Colon
            b':' => {
                if state.match_byte(b':') {
                    state.add_token(TokenKind::PunctuationDoubleColon);
                } else {
                    state.add_token(TokenKind::PunctuationColon);
                }
            }

            // Assignment, Equality, and Fat Arrow
            b'=' => {
                if state.match_byte(b'=') {
                    state.add_token(TokenKind::OperatorEqual);
                } else if state.match_byte(b'>') {
                    state.add_token(TokenKind::OperatorFatArrow);
                } else {
                    state.add_token(TokenKind::OperatorAssign);
                }
            }

            // Logical NOT and Not-Equal
            b'!' => {
                if state.match_byte(b'=') {
                    state.add_token(TokenKind::OperatorNotEqual);
                } else {
                    state.add_token(TokenKind::OperatorLogicalNot);
                }
            }

            // Relational and Shift Operators
            b'<' => {
                if state.match_byte(b'=') {
                    state.add_token(TokenKind::OperatorLessEqual);
                } else if state.match_byte(b'<') {
                    state.add_token(TokenKind::OperatorShiftLeft);
                } else {
                    state.add_token(TokenKind::OperatorLessThan);
                }
            }
            b'>' => {
                if state.match_byte(b'=') {
                    state.add_token(TokenKind::OperatorGreaterEqual);
                } else if state.match_byte(b'>') {
                    state.add_token(TokenKind::OperatorShiftRight);
                } else {
                    state.add_token(TokenKind::OperatorGreaterThan);
                }
            }

            // Ampersand & Logical AND
            b'&' => {
                if state.match_byte(b'&') {
                    state.add_token(TokenKind::OperatorLogicalAnd);
                } else {
                    state.add_token(TokenKind::PunctuationAmpersand);
                }
            }

            // Bitwise XOR
            b'^' => {
                state.add_token(TokenKind::OperatorBitwiseXor);
            }

            // Logical OR and Bitwise OR
            b'|' => {
                if state.match_byte(b'|') {
                    state.add_token(TokenKind::OperatorLogicalOr);
                } else {
                    state.add_token(TokenKind::OperatorBitwiseOr);
                }
            }

            // Addition and Increment
            b'+' => {
                if state.match_byte(b'+') {
                    state.add_token(TokenKind::OperatorIncrement);
                } else if state.match_byte(b'=') {
                    state.add_token(TokenKind::OperatorPlusAssign);
                } else {
                    state.add_token(TokenKind::OperatorPlus);
                }
            }

            // Subtraction and Decrement
            b'-' => {
                if state.match_byte(b'-') {
                    state.add_token(TokenKind::OperatorDecrement);
                } else if state.match_byte(b'=') {
                    state.add_token(TokenKind::OperatorMinusAssign);
                } else {
                    state.add_token(TokenKind::OperatorMinus);
                }
            }

            // Multiplication and Modulo
            b'*' => {
                if state.match_byte(b'=') {
                    state.add_token(TokenKind::OperatorMultiplyAssign);
                } else {
                    state.add_token(TokenKind::OperatorMultiply);
                }
            }
            b'%' => {
                if state.match_byte(b'=') {
                    state.add_token(TokenKind::OperatorModuloAssign);
                } else {
                    state.add_token(TokenKind::OperatorModulo);
                }
            }

            // Division and Comments
            b'/' => {
                if state.match_byte(b'=') {
                    state.add_token(TokenKind::OperatorDivideAssign);
                } else if state.match_byte(b'/') {
                    // Single-line comment: consume until newline or EOF
                    while let Some(byte) = state.peek() {
                        if byte == b'\n' {
                            break;
                        }
                        state.advance();
                    }
                } else if state.match_byte(b'*') {
                    // Multi-line block comment: consume until */
                    let mut terminated = false;
                    while !state.is_at_end() {
                        if state.peek() == Some(b'*') && state.peek_next() == Some(b'/') {
                            state.advance();
                            state.advance();
                            terminated = true;
                            break;
                        }
                        state.advance();
                    }

                    if !terminated {
                        let token = Token::new(TokenKind::CommentMultiLine, state.make_span());
                        state.errors.push(LexingError::unterminated_comment(token));
                    }
                } else {
                    state.add_token(TokenKind::OperatorDivide);
                }
            }

            // Unrecognized character
            _ => {
                state.add_unknown_token();
            }
        }
    }

    // Terminate stream with End-Of-File token
    let eof_span = Span::new(source_id, state.cursor as u32, state.cursor as u32);
    state.tokens.push(Token::new(TokenKind::Eof, eof_span));

    (state.tokens, state.errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_bitwise_and_shift_tokens() {
        let source_code = "a | b ^ c << 2 >> 1";
        let tokens = lex_source(0, source_code).expect("Lexing should succeed");
        let kinds: Vec<TokenKind> = tokens.iter().map(|token| token.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Identifier,
                TokenKind::OperatorBitwiseOr,
                TokenKind::Identifier,
                TokenKind::OperatorBitwiseXor,
                TokenKind::Identifier,
                TokenKind::OperatorShiftLeft,
                TokenKind::Number,
                TokenKind::OperatorShiftRight,
                TokenKind::Number,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_negative_unterminated_string_diagnostic() {
        let source_code = "\"unterminated string";
        let result = lex_source(0, source_code);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_code(), "E0011");
    }

    #[test]
    fn test_negative_unterminated_character_diagnostic() {
        let source_code = "'a";
        let result = lex_source(0, source_code);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_code(), "E0012");
    }

    #[test]
    fn test_negative_unterminated_comment_diagnostic() {
        let source_code = "/* unterminated block comment";
        let result = lex_source(0, source_code);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_code(), "E0013");
    }

    #[test]
    fn test_negative_malformed_hex_diagnostic() {
        let source_code = "0x;";
        let result = lex_source(0, source_code);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_code(), "E0014");
    }

    #[test]
    fn test_negative_malformed_binary_diagnostic() {
        let source_code = "0b;";
        let result = lex_source(0, source_code);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_code(), "E0015");
    }
}
