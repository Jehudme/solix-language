//! Integration tests for compiler diagnostics and unified error reporting.

#![forbid(unsafe_code)]

use solix_common::{Span, Token, TokenKind};
use solix_compilation::{
    BindingError, CompilationDiagnostic, EmissionError, LexingError, ParsingError,
};

#[test]
fn test_compiler_lexing_error() {
    let span = Span::new(0, 10, 11);
    let token = Token::new(TokenKind::Unknown, span);
    let error = LexingError::unknown_token(token);

    assert_eq!(error.error_code(), "E0000");
    assert_eq!(error.source_span().offset(), 10);
    assert_eq!(error.source_span().len(), 1);
    assert_eq!(error.token().kind, TokenKind::Unknown);
}

#[test]
fn test_compiler_parsing_error() {
    let span = Span::new(1, 40, 41);
    let error = ParsingError::ExpectedSemicolon {
        span: span.to_source_span(),
    };

    assert_eq!(error.error_code(), "E0101");
    assert_eq!(error.source_span().offset(), 40);
    assert_eq!(error.source_span().len(), 1);
}

#[test]
fn test_compiler_binding_error() {
    let span = Span::new(1, 80, 95);
    let error = BindingError::undefined_symbol("MyMissingClass", span);

    assert_eq!(error.error_code(), "E0201");
    assert_eq!(error.source_span().offset(), 80);
    assert_eq!(error.source_span().len(), 15);
}

#[test]
fn test_compiler_binding_diagnostics_dependency_codes() {
    let dummy_span = Span::new(1, 10, 20);

    let version_conflict_error = BindingError::IncompatibleDependencyVersion {
        name: "raylib".to_string(),
        reason: "Major version mismatch between 1.0.0 and 2.0.0".to_string(),
        span: dummy_span.to_source_span(),
    };
    assert_eq!(version_conflict_error.error_code(), "E0289");

    let package_not_found_error = BindingError::ExternalPackageNotFound {
        name: "unknown_lib".to_string(),
        reason: "Package not installed".to_string(),
        span: dummy_span.to_source_span(),
    };
    assert_eq!(package_not_found_error.error_code(), "E0290");
}

#[test]
fn test_compilation_diagnostic_wrapping_and_codes() {
    let span = Span::new(1, 10, 20);

    // Lex
    let lex_err = LexingError::unknown_token(Token::new(TokenKind::Unknown, span));
    let diag_lex: CompilationDiagnostic = lex_err.into();
    assert_eq!(diag_lex.error_code(), "E0000");
    assert!(diag_lex.source_span().is_some());

    // Parse
    let parse_err = ParsingError::ExpectedSemicolon {
        span: span.to_source_span(),
    };
    let diag_parse: CompilationDiagnostic = parse_err.into();
    assert_eq!(diag_parse.error_code(), "E0101");
    assert_eq!(diag_parse.source_span().unwrap().offset(), 10);

    // Bind
    let bind_err = BindingError::undefined_symbol("Test", span);
    let diag_bind: CompilationDiagnostic = bind_err.into();
    assert_eq!(diag_bind.error_code(), "E0201");

    // Emit
    let emit_err = EmissionError::ConstantPoolOverflow {
        count: 70000,
        max: 65535,
    };
    let diag_emit: CompilationDiagnostic = emit_err.into();
    assert_eq!(diag_emit.error_code(), "E0301");
    assert!(diag_emit.source_span().is_none());

    // Custom
    let custom_diag = CompilationDiagnostic::custom("P9999", "Custom lint warning", Some(span));
    assert_eq!(custom_diag.error_code(), "P9999");
    assert_eq!(custom_diag.source_span().unwrap().offset(), 10);
}
