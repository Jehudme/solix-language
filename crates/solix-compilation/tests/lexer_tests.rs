//! Comprehensive integration test suite for the Solix lexical analyzer.

#![forbid(unsafe_code)]

mod common;

use common::LanguageFixture;
use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
use solix_compilation::phases::lexing::{LexingError, TokenKind, lex, lex_source};
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
        (
            "\"unterminated string",
            "Unterminated string at EOF",
            "E0011",
        ),
        (
            "'unterminated char",
            "Unterminated character literal",
            "E0012",
        ),
        ("0x;", "Hex prefix missing digits", "E0014"),
        ("0b;", "Binary prefix missing digits", "E0015"),
        (
            "/* unclosed comment",
            "Unterminated block comment at EOF",
            "E0013",
        ),
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

    let expected_codes = [
        "E0000", "E0000", "E0000", "E0000", "E0000", "E0014", "E0015", "E0011",
    ];
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

#[test]
fn test_trailing_decimal_point_float_literal() {
    let source = "1. 0. 42. 10.f 3.14";
    let tokens = lex_source(0, source).expect("Trailing decimal point floats must lex cleanly");
    assert_eq!(tokens.len(), 6); // 5 numbers + EOF
    for token in &tokens[0..5] {
        assert_eq!(token.kind, TokenKind::Number);
    }

    // Verify '..' range is not absorbed as float
    let range_source = "1..10";
    let range_tokens =
        lex_source(0, range_source).expect("Range dots must not be consumed into float");
    let range_kinds: Vec<TokenKind> = range_tokens.iter().map(|token| token.kind).collect();
    assert_eq!(
        range_kinds,
        vec![
            TokenKind::Number,
            TokenKind::PunctuationDot,
            TokenKind::PunctuationDot,
            TokenKind::Number,
            TokenKind::Eof,
        ]
    );

    // Verify member call '1.toString()' leaves dot as punctuation
    let member_source = "1.toString()";
    let member_tokens = lex_source(0, member_source)
        .expect("Member call on number must not consume dot into float");
    let member_kinds: Vec<TokenKind> = member_tokens.iter().map(|token| token.kind).collect();
    assert_eq!(
        member_kinds,
        vec![
            TokenKind::Number,
            TokenKind::PunctuationDot,
            TokenKind::Identifier,
            TokenKind::PunctuationOpenParen,
            TokenKind::PunctuationCloseParen,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn test_raw_newline_in_string_literal_rejected() {
    let unescaped_newline_source = "\"hello\nworld\"";
    let lex_result = lex_source(0, unescaped_newline_source);
    assert!(
        lex_result.is_err(),
        "Raw newline in string literal must produce lexing error"
    );
    let errors = lex_result.expect_err("Lexing errors");
    assert!(
        errors
            .iter()
            .any(|error| matches!(error, LexingError::UnterminatedStringLiteral { .. })),
        "Expected UnterminatedStringLiteral diagnostic for raw newline in string literal"
    );

    // Escaped newline must continue to succeed
    let escaped_source = "\"hello\\nworld\"";
    let escaped_result = lex_source(0, escaped_source);
    assert!(
        escaped_result.is_ok(),
        "Escaped newline in string literal must lex successfully"
    );
}

#[test]
fn test_decimal_integer_with_leading_zeros_rejected() {
    let invalid_inputs = ["0123", "007", "00", "0_123"];
    for invalid_input in invalid_inputs {
        let lex_result = lex_source(0, invalid_input);
        assert!(
            lex_result.is_err(),
            "Leading zero integer literal '{invalid_input}' must be rejected by lexer"
        );
        let errors = lex_result.expect_err("Lexing errors");
        assert!(
            errors
                .iter()
                .any(|error| matches!(error, LexingError::LeadingZeroIntegerLiteral { .. })),
            "Expected LeadingZeroIntegerLiteral diagnostic for '{invalid_input}', got {errors:?}"
        );
    }

    // Single 0, hex, binary, and floats starting with 0 must succeed
    let valid_inputs = ["0", "0L", "0x12", "0XFF", "0b101", "0B0", "0.5", "0."];
    for valid_input in valid_inputs {
        let lex_result = lex_source(0, valid_input);
        assert!(
            lex_result.is_ok(),
            "Valid literal '{valid_input}' must lex successfully, got {lex_result:?}"
        );
    }
}

#[test]
fn test_numeric_underscore_and_exponent_validation() {
    let invalid_cases = [
        ("0x_", "E0014"),
        ("0x___", "E0014"),
        ("0x_FF", "E0014"),
        ("0xFF_", "E0014"),
        ("0xF__F", "E0014"),
        ("0xFF_L", "E0014"),
        ("0b_", "E0015"),
        ("0b___", "E0015"),
        ("0b_10", "E0015"),
        ("0b10_", "E0015"),
        ("0b1__0", "E0015"),
        ("0b10_L", "E0015"),
        ("1_", "E0017"),
        ("1__0", "E0017"),
        ("1_L", "E0017"),
        ("1_.0", "E0017"),
        ("1.0_", "E0017"),
        ("1.0__5", "E0017"),
        ("1.0_f", "E0017"),
        ("1_e2", "E0017"),
        ("1e", "E0017"),
        ("1e+", "E0017"),
        ("1e-", "E0017"),
        ("1.0e", "E0017"),
        ("1.0e+", "E0017"),
        ("1.0e-", "E0017"),
        ("1e_2", "E0017"),
        ("1e2_", "E0017"),
        ("1e2__3", "E0017"),
        ("1.0e_5", "E0017"),
    ];

    for (snippet, expected_code) in invalid_cases {
        let result = lex_source(0, snippet);
        assert!(
            result.is_err(),
            "Expected malformed numeric literal '{snippet}' to fail lexing"
        );
        let errors = result.expect_err("Expected lexing error");
        assert!(
            errors
                .iter()
                .any(|error| error.error_code() == expected_code),
            "Expected {expected_code} for '{snippet}', got {errors:?}"
        );
    }

    let valid_cases = [
        "0xDEAD_BEEF",
        "0b1111_0000",
        "1_000_000",
        "1_000L",
        "1.",
        "1.f",
        "1.F",
        "1.e3",
        "1_0.2_5e+1_0",
        "1_0.2_5e-1_0f",
    ];
    for snippet in valid_cases {
        let result = lex_source(0, snippet);
        assert!(
            result.is_ok(),
            "Expected valid numeric literal '{snippet}' to lex cleanly, got {result:?}"
        );
    }
}

#[test]
fn test_source_index_u16_capacity_check() {
    use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
    let mut context = CompilationContext::new(CompilationOptions::new());
    for index in 0..usize::from(u16::MAX) + 1 {
        context
            .add_source(format!("src_{index}.slx"), "")
            .expect("First 65,536 sources fit within u16");
    }
    let overflow = context.add_source("overflow.slx", "");
    assert!(
        overflow.is_err(),
        "65,537th source must be rejected with diagnostic"
    );
    let diag = overflow.expect_err("Expected E0018");
    assert!(
        format!("{diag}").contains("source file limit exceeded"),
        "Unexpected diagnostic message: {diag}"
    );
}
