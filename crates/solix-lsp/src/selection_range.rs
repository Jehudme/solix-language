//! Smart structural AST selection range expansion.

#![forbid(unsafe_code)]

use crate::diagnostics::{position_to_byte_offset, span_to_range};
use crate::workspace::AnalysisSnapshot;
use solix_common::Span;
use solix_compilation::phases::parser::statements::ProgramAst;
use solix_compilation::phases::parser::statements::declarations::{ClassMember, Declaration};
use tower_lsp::lsp_types::{Position, Range, SelectionRange};

/// Resolves `textDocument/selectionRange` for the given cursor positions.
#[must_use]
pub fn selection_ranges(
    snapshot: &AnalysisSnapshot,
    positions: &[Position],
) -> Vec<SelectionRange> {
    let mut results = Vec::new();

    for &pos in positions {
        let offset = position_to_byte_offset(&snapshot.source_text, pos);
        let target_offset = offset as u32;

        let mut enclosing_spans = Vec::new();

        // 1. Full document range
        enclosing_spans.push(Span::new(0, 0, snapshot.source_text.len() as u32));

        if let Some(ast) = &snapshot.ast {
            collect_selection_spans_in_ast(ast, target_offset, &mut enclosing_spans);
        }

        // 2. Token at offset
        for token in &snapshot.tokens {
            if token.span.start <= target_offset && target_offset <= token.span.end {
                enclosing_spans.push(token.span);
                break;
            }
        }

        // Deduplicate and sort from innermost (smallest) to outermost (largest)
        enclosing_spans.sort_by_key(|span| span.length());
        enclosing_spans.dedup();

        // Construct nested Box<SelectionRange>
        let mut current_range: Option<Box<SelectionRange>> = None;
        for &span in enclosing_spans.iter().rev() {
            let range = span_to_range(&snapshot.source_text, span);
            current_range = Some(Box::new(SelectionRange {
                range,
                parent: current_range,
            }));
        }

        if let Some(sel) = current_range {
            results.push(*sel);
        } else {
            let default_range = Range {
                start: pos,
                end: pos,
            };
            results.push(SelectionRange {
                range: default_range,
                parent: None,
            });
        }
    }

    results
}

fn collect_selection_spans_in_ast(ast: &ProgramAst, target_offset: u32, spans: &mut Vec<Span>) {
    if ast.span.start <= target_offset && target_offset <= ast.span.end {
        spans.push(ast.span);
    }

    for decl in &ast.declarations {
        match decl {
            Declaration::Class(class_decl) => {
                if class_decl.span.start <= target_offset && target_offset <= class_decl.span.end {
                    spans.push(class_decl.span);
                    if class_decl.name_span.start <= target_offset
                        && target_offset <= class_decl.name_span.end
                    {
                        spans.push(class_decl.name_span);
                    }
                    for member in &class_decl.members {
                        if let ClassMember::Method(m) = member
                            && m.span.start <= target_offset
                            && target_offset <= m.span.end
                        {
                            spans.push(m.span);
                            if m.name_span.start <= target_offset
                                && target_offset <= m.name_span.end
                            {
                                spans.push(m.name_span);
                            }
                        }
                    }
                }
            }
            Declaration::Method(m)
                if m.span.start <= target_offset && target_offset <= m.span.end =>
            {
                spans.push(m.span);
                if m.name_span.start <= target_offset && target_offset <= m.name_span.end {
                    spans.push(m.name_span);
                }
            }
            _ => {}
        }
    }
}
