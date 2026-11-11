//! Diagnostic translation and source coordinate conversions for the Solix Language Server.

#![forbid(unsafe_code)]

use solix_common::Span;
use solix_compilation::{BindingError, LexingError, ParsingError};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range};

/// Converts a 0-based byte offset within a source string into an LSP 0-based [`Position`].
///
/// Note: Characters in LSP [`Position`] are measured in UTF-16 code units according
/// to the Language Server Protocol specification.
#[must_use]
pub fn byte_offset_to_position(source_text: &str, byte_offset: usize) -> Position {
    let clamped_offset = byte_offset.min(source_text.len());
    let mut line_number: u32 = 0;
    let mut character_offset: u32 = 0;

    for (current_byte_index, character) in source_text.char_indices() {
        if current_byte_index >= clamped_offset {
            break;
        }
        if character == '\n' {
            line_number += 1;
            character_offset = 0;
        } else {
            character_offset += character.len_utf16() as u32;
        }
    }

    Position {
        line: line_number,
        character: character_offset,
    }
}

/// Converts a [`Span`] and its associated source text into an LSP [`Range`].
#[must_use]
pub fn span_to_range(source_text: &str, span: Span) -> Range {
    let start_position = byte_offset_to_position(source_text, span.start as usize);
    let end_position = byte_offset_to_position(source_text, span.end as usize);
    Range {
        start: start_position,
        end: end_position,
    }
}

/// Converts a canonical [`LexingError`] into an LSP [`Diagnostic`].
#[must_use]
pub fn lexing_error_to_diagnostic(source_text: &str, error: &LexingError) -> Diagnostic {
    let source_span = error.source_span();
    let start_offset = source_span.offset();
    let end_offset = start_offset + source_span.len();
    let start_position = byte_offset_to_position(source_text, start_offset);
    let end_position = byte_offset_to_position(source_text, end_offset);

    Diagnostic {
        range: Range {
            start: start_position,
            end: end_position,
        },
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::String(error.error_code().to_string())),
        code_description: None,
        source: Some("solix".to_string()),
        message: error.to_string(),
        related_information: None,
        tags: None,
        data: None,
    }
}

/// Converts a canonical [`ParsingError`] into an LSP [`Diagnostic`].
#[must_use]
pub fn parsing_error_to_diagnostic(source_text: &str, error: &ParsingError) -> Diagnostic {
    let source_span = error.source_span();
    let start_offset = source_span.offset();
    let end_offset = start_offset + source_span.len();
    let start_position = byte_offset_to_position(source_text, start_offset);
    let end_position = byte_offset_to_position(source_text, end_offset);

    Diagnostic {
        range: Range {
            start: start_position,
            end: end_position,
        },
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::String(error.error_code().to_string())),
        code_description: None,
        source: Some("solix".to_string()),
        message: error.to_string(),
        related_information: None,
        tags: None,
        data: None,
    }
}

/// Converts a canonical [`BindingError`] into an LSP [`Diagnostic`].
#[must_use]
pub fn binding_error_to_diagnostic(source_text: &str, error: &BindingError) -> Diagnostic {
    let source_span = error.source_span();
    let start_offset = source_span.offset();
    let end_offset = start_offset + source_span.len();
    let start_position = byte_offset_to_position(source_text, start_offset);
    let end_position = byte_offset_to_position(source_text, end_offset);

    Diagnostic {
        range: Range {
            start: start_position,
            end: end_position,
        },
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::String(error.error_code().to_string())),
        code_description: None,
        source: Some("solix".to_string()),
        message: error.to_string(),
        related_information: None,
        tags: None,
        data: None,
    }
}
