//! Integration tests for common spans and emission diagnostic infrastructure.

#![forbid(unsafe_code)]

use solix_common::{EmissionError, Span};

#[test]
fn test_common_span_coordinates() {
    let span = Span::new(2, 100, 150);
    assert_eq!(span.id, 2);
    assert_eq!(span.start, 100);
    assert_eq!(span.end, 150);
    assert_eq!(span.length(), 50);
    assert!(!span.is_empty());

    let source_span = span.to_source_span();
    assert_eq!(source_span.offset(), 100);
    assert_eq!(source_span.len(), 50);

    let other_span = Span::new(2, 120, 200);
    let combined = span.combine(other_span);
    assert_eq!(combined.start, 100);
    assert_eq!(combined.end, 200);
}

#[test]
fn test_common_emission_error_codes() {
    let emit_err = EmissionError::ConstantPoolOverflow {
        count: 70000,
        max: 65535,
    };
    assert_eq!(emit_err.error_code(), "E0301");

    let reg_err = EmissionError::RegisterOverflow {
        register_index: 300,
        max: 255,
    };
    assert_eq!(reg_err.error_code(), "E0302");

    let magic_err = EmissionError::InvalidMagic {
        found: *b"PK\x03\x04",
    };
    let magic_text = magic_err.to_string();
    assert!(magic_text.contains("'PK\\x03\\x04'"), "got: {magic_text}");
    assert!(
        !magic_text.contains("[80,"),
        "leaked debug array: {magic_text}"
    );

    let container_err = EmissionError::ContainerSerializationFailed {
        reason: "corrupt manifest".to_string(),
    };
    let help_text = miette::Diagnostic::help(&container_err)
        .expect("help metadata")
        .to_string();
    assert!(help_text.contains("manifest.yaml"));
    assert!(!help_text.contains("manifest.toml"));

    let pool_num = solix_common::ConstantPoolValue::from_f64(2.5);
    let pool_str = solix_common::ConstantPoolValue::String("ok".to_string());
    assert_eq!(pool_num.to_string(), "2.5");
    assert_eq!(pool_str.to_string(), "\"ok\"");
}
