//! Folding ranges for imports, comments, classes, functions, and control blocks.

#![forbid(unsafe_code)]

use crate::diagnostics::byte_offset_to_position;
use crate::workspace::AnalysisSnapshot;
use solix_common::Span;
use solix_compilation::phases::parser::statements::declarations::{ClassMember, Declaration};
use tower_lsp::lsp_types::{FoldingRange, FoldingRangeKind};

/// Resolves `textDocument/foldingRange` for a document.
#[must_use]
pub fn folding_ranges(snapshot: &AnalysisSnapshot) -> Vec<FoldingRange> {
    let mut ranges = Vec::new();

    // 1. Contiguous import block folding
    if let Some(ast) = &snapshot.ast {
        if !ast.imports.is_empty() {
            let first_import = &ast.imports[0];
            let last_import = &ast.imports[ast.imports.len() - 1];
            let start_pos =
                byte_offset_to_position(&snapshot.source_text, first_import.span.start as usize);
            let end_pos =
                byte_offset_to_position(&snapshot.source_text, last_import.span.end as usize);

            if end_pos.line > start_pos.line {
                ranges.push(FoldingRange {
                    start_line: start_pos.line,
                    start_character: Some(start_pos.character),
                    end_line: end_pos.line,
                    end_character: Some(end_pos.character),
                    kind: Some(FoldingRangeKind::Imports),
                    collapsed_text: Some("...".to_string()),
                });
            }
        }

        // 2. Declaration folding (classes, interfaces, enums, methods)
        for decl in &ast.declarations {
            match decl {
                Declaration::Class(class_decl) => {
                    add_folding_range_from_span(
                        &snapshot.source_text,
                        class_decl.span,
                        Some(FoldingRangeKind::Region),
                        &mut ranges,
                    );
                    for member in &class_decl.members {
                        match member {
                            ClassMember::Method(method_decl) => {
                                add_folding_range_from_span(
                                    &snapshot.source_text,
                                    method_decl.span,
                                    Some(FoldingRangeKind::Region),
                                    &mut ranges,
                                );
                            }
                            ClassMember::Constructor(ctor_decl) => {
                                add_folding_range_from_span(
                                    &snapshot.source_text,
                                    ctor_decl.span,
                                    Some(FoldingRangeKind::Region),
                                    &mut ranges,
                                );
                            }
                            ClassMember::Destructor(dtor_decl) => {
                                add_folding_range_from_span(
                                    &snapshot.source_text,
                                    dtor_decl.span,
                                    Some(FoldingRangeKind::Region),
                                    &mut ranges,
                                );
                            }
                            _ => {}
                        }
                    }
                }
                Declaration::Interface(iface_decl) => {
                    add_folding_range_from_span(
                        &snapshot.source_text,
                        iface_decl.span,
                        Some(FoldingRangeKind::Region),
                        &mut ranges,
                    );
                }
                Declaration::Enum(enum_decl) => {
                    add_folding_range_from_span(
                        &snapshot.source_text,
                        enum_decl.span,
                        Some(FoldingRangeKind::Region),
                        &mut ranges,
                    );
                }
                Declaration::Method(method_decl) => {
                    add_folding_range_from_span(
                        &snapshot.source_text,
                        method_decl.span,
                        Some(FoldingRangeKind::Region),
                        &mut ranges,
                    );
                }
                _ => {}
            }
        }
    }

    // 3. Multi-line comment folding
    for token in &snapshot.tokens {
        if matches!(
            token.kind,
            solix_common::tokens::TokenKind::CommentMultiLine
        ) {
            add_folding_range_from_span(
                &snapshot.source_text,
                token.span,
                Some(FoldingRangeKind::Comment),
                &mut ranges,
            );
        }
    }

    ranges
}

fn add_folding_range_from_span(
    source_text: &str,
    span: Span,
    kind: Option<FoldingRangeKind>,
    ranges: &mut Vec<FoldingRange>,
) {
    let start_pos = byte_offset_to_position(source_text, span.start as usize);
    let end_pos = byte_offset_to_position(source_text, span.end as usize);

    if end_pos.line > start_pos.line {
        ranges.push(FoldingRange {
            start_line: start_pos.line,
            start_character: Some(start_pos.character),
            end_line: end_pos.line,
            end_character: Some(end_pos.character),
            kind,
            collapsed_text: Some("...".to_string()),
        });
    }
}
