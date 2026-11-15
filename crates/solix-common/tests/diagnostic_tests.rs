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
}
