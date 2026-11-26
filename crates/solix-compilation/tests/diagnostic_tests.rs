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

#[test]
fn test_diagnostics_have_no_debug_leaks_and_unique_codes() {
    use solix_common::diagnostics::ManifestError;
    use std::collections::HashMap;

    let dummy = Span::new(0, 1, 5);
    let dummy_span = dummy.to_source_span();
    let dummy_tok = Token::new(TokenKind::Unknown, dummy);

    let missing_ret = BindingError::MissingReturn {
        method_name: "compute".to_string(),
        expected_type: "int32".to_string(),
        span: dummy_span,
    };
    let missing_ret_msg = missing_ret.to_string();
    assert_eq!(
        missing_ret_msg,
        "Non-void function 'compute' must return a value of type 'int32'"
    );
    assert!(!missing_ret_msg.contains("Primitive("));

    let ambig = BindingError::AmbiguousImport {
        symbol_name: "Widget".to_string(),
        candidates: vec!["ui::Widget".to_string(), "net::Widget".to_string()],
        span: dummy_span,
    };
    let ambig_msg = ambig.to_string();
    assert_eq!(
        ambig_msg,
        "Ambiguous imported symbol 'Widget'; matches candidates: ui::Widget, net::Widget"
    );
    assert!(!ambig_msg.contains("[\""));

    assert_eq!(
        ParsingError::DotNotAllowedInTypePath { span: dummy_span }.error_code(),
        "E0157"
    );
    assert_eq!(
        ParsingError::UnknownControlFlowKeyword {
            identifier: "whil".to_string(),
            suggestion: "while".to_string(),
            span: dummy_span,
        }
        .error_code(),
        "E0158"
    );
    assert_eq!(
        BindingError::AssertMessageNotString {
            found: "int32".to_string(),
            span: dummy_span,
        }
        .error_code(),
        "E0317"
    );
    assert_eq!(
        BindingError::CyclicInterfaceInheritance {
            interface_name: "Proto".to_string(),
            span: dummy_span,
        }
        .error_code(),
        "E0323"
    );

    let sample_variants: Vec<(&'static str, &'static str)> = vec![
        (
            "LexingError::UnknownToken",
            LexingError::unknown_token(dummy_tok).error_code(),
        ),
        (
            "ParsingError::ExpectedSemicolon",
            ParsingError::ExpectedSemicolon { span: dummy_span }.error_code(),
        ),
        (
            "ParsingError::DotNotAllowedInTypePath",
            ParsingError::DotNotAllowedInTypePath { span: dummy_span }.error_code(),
        ),
        (
            "ParsingError::UnknownControlFlowKeyword",
            ParsingError::UnknownControlFlowKeyword {
                identifier: "x".to_string(),
                suggestion: "y".to_string(),
                span: dummy_span,
            }
            .error_code(),
        ),
        (
            "BindingError::UndefinedSymbol",
            BindingError::undefined_symbol("missing", dummy).error_code(),
        ),
        (
            "BindingError::DuplicateSymbol",
            BindingError::DuplicateSymbol {
                symbol_name: "dup".to_string(),
                span: dummy_span,
            }
            .error_code(),
        ),
        (
            "BindingError::CyclicInheritance",
            BindingError::CyclicInheritance {
                class_name: "A".to_string(),
                span: dummy_span,
            }
            .error_code(),
        ),
        (
            "BindingError::CyclicInterfaceInheritance",
            BindingError::CyclicInterfaceInheritance {
                interface_name: "I".to_string(),
                span: dummy_span,
            }
            .error_code(),
        ),
        (
            "BindingError::AssertMessageNotString",
            BindingError::AssertMessageNotString {
                found: "bool".to_string(),
                span: dummy_span,
            }
            .error_code(),
        ),
        (
            "BindingError::GenericInstantiationDepthExceeded",
            BindingError::GenericInstantiationDepthExceeded {
                class_name: "G".to_string(),
                maximum_depth: 64,
                span: dummy_span,
            }
            .error_code(),
        ),
        (
            "BindingError::CyclicStaticFieldDependency",
            BindingError::CyclicStaticFieldDependency {
                cycle_path: "A.x -> B.y -> A.x".to_string(),
                span: dummy_span,
            }
            .error_code(),
        ),
        (
            "BindingError::VirtualCallInDestructorForbidden",
            BindingError::VirtualCallInDestructorForbidden {
                method_name: "m".to_string(),
                span: dummy_span,
            }
            .error_code(),
        ),
        (
            "BindingError::SymbolCollidesWithPackageName",
            BindingError::SymbolCollidesWithPackageName {
                symbol_name: "p".to_string(),
                package_name: "p".to_string(),
                span: dummy_span,
            }
            .error_code(),
        ),
        (
            "BindingError::InexhaustiveEnumSwitch",
            BindingError::InexhaustiveEnumSwitch {
                enum_name: "E".to_string(),
                span: dummy_span,
            }
            .error_code(),
        ),
        (
            "EmissionError::ConstantPoolOverflow",
            EmissionError::ConstantPoolOverflow { count: 1, max: 1 }.error_code(),
        ),
        (
            "EmissionError::RegisterOverflow",
            EmissionError::RegisterOverflow {
                register_index: 1,
                max: 1,
            }
            .error_code(),
        ),
        (
            "EmissionError::ArgumentCountOverflow",
            EmissionError::ArgumentCountOverflow {
                argument_count: 1,
                max: 1,
            }
            .error_code(),
        ),
        (
            "EmissionError::JumpOffsetOverflow",
            EmissionError::JumpOffsetOverflow { offset: 1 }.error_code(),
        ),
        (
            "EmissionError::InvalidMagic",
            EmissionError::InvalidMagic { found: *b"XXXX" }.error_code(),
        ),
        (
            "EmissionError::UnsupportedVersion",
            EmissionError::UnsupportedVersion {
                expected_major: 1,
                expected_minor: 0,
                found_major: 2,
                found_minor: 0,
            }
            .error_code(),
        ),
        (
            "ManifestError::ParseFailed",
            ManifestError::ParseFailed {
                filename: "manifest.yaml".to_string(),
                line: None,
                column: None,
                field_path: None,
                message: "err".to_string(),
                snippet: String::new(),
            }
            .error_code(),
        ),
    ];

    let mut seen: HashMap<&'static str, &'static str> = HashMap::new();
    for (variant_name, code) in sample_variants {
        if let Some(previous) = seen.insert(code, variant_name) {
            panic!("Duplicate error code '{code}' shared by '{previous}' and '{variant_name}'");
        }
    }
}

#[test]
fn test_real_missing_return_diagnostic_formats_canonical_solix_type() {
    use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
    use solix_compilation::phases::{binder::bind, lexing::lex, parser::parse};

    let source = "int32 compute_total() { int32 local_val = 42; }";
    let mut context =
        CompilationContext::new(CompilationOptions::new().with_source("test.slx", source));
    lex(&mut context).expect("lex succeeds");
    parse(&mut context).expect("parse succeeds");
    let bind_err = bind(&mut context).expect_err("missing return must fail binding");
    let formatted = bind_err
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        formatted.contains("Non-void function 'compute_total'"),
        "got: {formatted}"
    );
    assert!(formatted.contains("'int32'"), "got: {formatted}");
    assert!(
        !formatted.contains("Primitive("),
        "leaked Primitive Debug syntax: {formatted}"
    );
}
