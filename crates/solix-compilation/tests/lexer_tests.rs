//! Comprehensive integration test suite for the Solix lexical analyzer.

#![forbid(unsafe_code)]

mod common;

use common::LanguageFixture;
use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
use solix_compilation::phases::lexing::{TokenKind, lex, lex_source};
use std::path::Path;

#[test]
fn test_positive_lex_all_tokens_fixture() {
    let fixture_path = Path::new("tests/fixtures/syntax/positive/all_tokens.slx");
    let fixture = LanguageFixture::load_from_path(fixture_path)
        .expect("Failed to load all_tokens.slx positive fixture");

    let tokens = lex_source(0, &fixture.source_content)
        .expect("Lexing all_tokens.slx should succeed without errors");

    assert!(!tokens.is_empty(), "Token stream should not be empty");
    assert_eq!(
        tokens.last().map(|token| token.kind),
        Some(TokenKind::Eof),
        "Last token must be End-Of-File"
    );

    // Verify key terminal grammar categories are present in the stream
    let kinds: Vec<TokenKind> = tokens.iter().map(|token| token.kind).collect();

    // Check presence of primitives
    assert!(kinds.contains(&TokenKind::PrimitiveVoid));
    assert!(kinds.contains(&TokenKind::PrimitiveInt32));
    assert!(kinds.contains(&TokenKind::PrimitiveFloat64));
    assert!(kinds.contains(&TokenKind::PrimitiveTuple));

    // Check presence of keywords
    assert!(kinds.contains(&TokenKind::KeywordClass));
    assert!(kinds.contains(&TokenKind::KeywordInterface));
    assert!(kinds.contains(&TokenKind::KeywordReturn));
    assert!(kinds.contains(&TokenKind::KeywordAssert));

    // Check presence of operators
    assert!(kinds.contains(&TokenKind::OperatorEqual));
    assert!(kinds.contains(&TokenKind::OperatorFatArrow));
    assert!(kinds.contains(&TokenKind::OperatorPlus));
    assert!(kinds.contains(&TokenKind::OperatorLogicalAnd));

    // Check presence of punctuation & [] optimization
    assert!(kinds.contains(&TokenKind::PunctuationArrayBrackets));
    assert!(kinds.contains(&TokenKind::PunctuationDoubleColon));
    assert!(kinds.contains(&TokenKind::PunctuationEllipsis));

    // Check presence of literals
    assert!(kinds.contains(&TokenKind::Number));
    assert!(kinds.contains(&TokenKind::String));
    assert!(kinds.contains(&TokenKind::Char));
    assert!(kinds.contains(&TokenKind::Identifier));
}

#[test]
fn test_positive_compilation_context_pipeline() {
    let fixture_path = Path::new("tests/fixtures/syntax/positive/all_tokens.slx");
    let fixture =
        LanguageFixture::load_from_path(fixture_path).expect("Failed to load positive fixture");

    let options = CompilationOptions::new().with_source("all_tokens.slx", &fixture.source_content);
    let mut context = CompilationContext::new(options);

    let metadata = lex(&mut context).expect("Lexing context should succeed");

    assert!(metadata.tokens_count > 0);
    assert_eq!(metadata.sources_processed, 1);
    assert_eq!(context.tokens.len(), 1);
    assert_eq!(context.tokens[0].len(), metadata.tokens_count);

    // Verify lexeme resolution via context
    let first_token = context.tokens[0][0];
    let lexeme = context.get_lexeme(first_token.span);
    assert_eq!(lexeme, "void");
}

#[test]
fn test_negative_inlined_unknown_tokens() {
    // Concise 1-2 line negative snippets are inlined directly in the test file per GL-0009
    let test_cases = [
        ("int32 x = @;", "Unrecognized symbol @", "E0000"),
        ("int32 y = \\;", "Unrecognized symbol \\", "E0000"),
        ("\"unterminated string", "Unterminated string at EOF", "E0011"),
        ("'unterminated char", "Unterminated character literal", "E0012"),
        ("0x;", "Hex prefix missing digits", "E0014"),
        ("0b;", "Binary prefix missing digits", "E0015"),
        ("/* unclosed comment", "Unterminated block comment at EOF", "E0013"),
    ];

    for (source_snippet, description, expected_code) in test_cases {
        let result = lex_source(0, source_snippet);
        assert!(
            result.is_err(),
            "Expected snippet ({}) to trigger error, but it succeeded: {}",
            description,
            source_snippet
        );

        let errors = result.unwrap_err();
        assert!(
            !errors.is_empty(),
            "At least one error must be emitted for {}",
            description
        );
        assert_eq!(
            errors[0].error_code(),
            expected_code,
            "Error code must be {} for {}",
            expected_code,
            description
        );
    }
}

#[test]
fn test_negative_invalid_tokens_fixture_detects_all_errors() {
    let fixture_path = Path::new("tests/fixtures/syntax/negative/invalid_tokens.slx");
    let fixture = LanguageFixture::load_from_path(fixture_path)
        .expect("Failed to load invalid_tokens.slx fixture");

    let result = lex_source(0, &fixture.source_content);
    assert!(
        result.is_err(),
        "invalid_tokens.slx must fail lexing and return accumulated errors"
    );

    let errors = result.expect_err("Expected errors from invalid fixture");

    // invalid_tokens.slx defines exactly 8 distinct lexical errors
    assert_eq!(
        errors.len(),
        8,
        "Expected exactly 8 lexical errors in invalid_tokens.slx, found {}: {:#?}",
        errors.len(),
        errors
    );

    let expected_codes = ["E0000", "E0000", "E0000", "E0000", "E0000", "E0014", "E0015", "E0011"];
    for (index, error) in errors.iter().enumerate() {
        assert_eq!(
            error.error_code(),
            expected_codes[index],
            "Error #{} code should be {}",
            index,
            expected_codes[index]
        );
        let token = error.token();
        assert!(
            token.span.end > token.span.start,
            "Error #{} span must not be empty: {:?}",
            index,
            token.span
        );
    }
}

#[test]
fn test_array_brackets_optimization() {
    let tokens = lex_source(0, "int32[] numbers;").expect("Lexing array brackets should succeed");
    assert_eq!(tokens[0].kind, TokenKind::PrimitiveInt32);
    assert_eq!(tokens[1].kind, TokenKind::PunctuationArrayBrackets);
    assert_eq!(tokens[2].kind, TokenKind::Identifier);
    assert_eq!(tokens[3].kind, TokenKind::PunctuationSemicolon);
    assert_eq!(tokens[4].kind, TokenKind::Eof);
}

#[test]
fn test_multiline_block_comments_stripped() {
    let source = "/* block\ncomment */ let /* another */ x = 42;";
    let tokens = lex_source(0, source).expect("Lexing with block comments should succeed");
    let kinds: Vec<TokenKind> = tokens.iter().map(|token| token.kind).collect();

    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier,     // let
            TokenKind::Identifier,     // x
            TokenKind::OperatorAssign, // =
            TokenKind::Number,         // 42
            TokenKind::PunctuationSemicolon,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn test_numeric_radixes_and_suffixes() {
    let source = "0 255 0xFF 0b1010 100L 0x1FL 3.14 1.5e-3 2.0f";
    let tokens = lex_source(0, source).expect("Lexing numeric radices should succeed");

    assert_eq!(tokens.len(), 10); // 9 numbers + 1 EOF
    for token in &tokens[0..9] {
        assert_eq!(token.kind, TokenKind::Number);
    }
}
