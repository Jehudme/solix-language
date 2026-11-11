//! Integration tests for the Solix Language Server Protocol implementation.

#![forbid(unsafe_code)]

use solix_common::{Span, Token, TokenKind};
use solix_compilation::{BindingError, LexingError, ParsingError};
use solix_lsp::{
    binding_error_to_diagnostic, byte_offset_to_position, lexing_error_to_diagnostic,
    parsing_error_to_diagnostic, span_to_range,
};
use tower_lsp::lsp_types::{DiagnosticSeverity, NumberOrString, Position};

#[test]
fn test_lsp_server_scaffolding_ready() {
    let lsp_protocol_version = "3.17";
    assert_eq!(lsp_protocol_version, "3.17");
}

#[test]
fn test_byte_offset_to_position_multiline() {
    let source = "func main() {\n    let val = 42;\n}";
    // Position of 'let' on line 1: offset is 14 (13 chars on line 0 + 1 newline + 4 spaces = 18)
    let position_start = byte_offset_to_position(source, 0);
    assert_eq!(
        position_start,
        Position {
            line: 0,
            character: 0
        }
    );

    let position_line_two = byte_offset_to_position(source, 18);
    assert_eq!(
        position_line_two,
        Position {
            line: 1,
            character: 4
        }
    );
}

#[test]
fn test_span_to_range_conversion() {
    let source = "class Player {\n}\n";
    let span = Span::new(0, 6, 12); // "Player"
    let range = span_to_range(source, span);

    assert_eq!(
        range.start,
        Position {
            line: 0,
            character: 6
        }
    );
    assert_eq!(
        range.end,
        Position {
            line: 0,
            character: 12
        }
    );
}

#[test]
fn test_lexing_error_translation_to_lsp_diagnostic() {
    let source = "let invalid = @;\n";
    let error_token = Token::new(TokenKind::Unknown, Span::new(0, 14, 15));
    let lex_error = LexingError::unknown_token(error_token);

    let diagnostic = lexing_error_to_diagnostic(source, &lex_error);

    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
    assert_eq!(
        diagnostic.code,
        Some(NumberOrString::String("E0000".to_string()))
    );
    assert_eq!(diagnostic.source.as_deref(), Some("solix"));
    assert_eq!(
        diagnostic.range.start,
        Position {
            line: 0,
            character: 14
        }
    );
    assert_eq!(
        diagnostic.range.end,
        Position {
            line: 0,
            character: 15
        }
    );
}

#[test]
fn test_parsing_error_translation_to_lsp_diagnostic() {
    let source = "func test() {\n    return\n}\n";
    let error_span = Span::new(0, 18, 24);
    let parse_error = ParsingError::ExpectedSemicolon {
        span: error_span.to_source_span(),
    };

    let diagnostic = parsing_error_to_diagnostic(source, &parse_error);

    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
    assert_eq!(
        diagnostic.code,
        Some(NumberOrString::String("E0101".to_string()))
    );
    assert_eq!(diagnostic.source.as_deref(), Some("solix"));
    assert_eq!(
        diagnostic.range.start,
        Position {
            line: 1,
            character: 4
        }
    );
    assert_eq!(
        diagnostic.range.end,
        Position {
            line: 1,
            character: 10
        }
    );
}

#[test]
fn test_binding_error_translation_to_lsp_diagnostic() {
    let source = "class Test {\n    int32 count = \"text\";\n}\n";
    let error_span = Span::new(0, 31, 37);
    let bind_error = BindingError::type_mismatch("int32", "string", error_span);

    let diagnostic = binding_error_to_diagnostic(source, &bind_error);

    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
    assert_eq!(
        diagnostic.code,
        Some(NumberOrString::String("E0203".to_string()))
    );
    assert_eq!(diagnostic.source.as_deref(), Some("solix"));
    assert_eq!(
        diagnostic.range.start,
        Position {
            line: 1,
            character: 18
        }
    );
    assert_eq!(
        diagnostic.range.end,
        Position {
            line: 1,
            character: 24
        }
    );
}
