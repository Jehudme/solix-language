//! Rich hover tooltips displaying signatures, doc comments, instance byte sizes, field offsets, and ARC representation.

#![forbid(unsafe_code)]

use crate::diagnostics::{position_to_byte_offset, span_to_range};
use crate::workspace::AnalysisSnapshot;
use solix_common::types::Type;
use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position};

/// Resolves `textDocument/hover` for the given position in a document.
#[must_use]
pub fn hover(snapshot: &AnalysisSnapshot, position: Position) -> Option<Hover> {
    let offset = position_to_byte_offset(&snapshot.source_text, position);
    let semantic = snapshot.last_good_semantic.as_ref()?;

    if let Some(symbol) = snapshot.symbol_at_offset(offset) {
        let mut markdown_sections = Vec::new();

        // 1. Code block with symbol signature
        let signature_line = match &symbol.symbol_type {
            Type::Class { symbol_name, .. } => format!("class {symbol_name}"),
            Type::FunctionPointer { .. } => {
                if symbol.is_static {
                    format!("static {}", symbol.to_signature_string())
                } else {
                    symbol.to_signature_string()
                }
            }
            _ => {
                let modifier_str = if symbol.is_const {
                    "const "
                } else if symbol.is_static {
                    "static "
                } else {
                    ""
                };
                format!("{modifier_str}{} {}", symbol.symbol_type, symbol.name)
            }
        };
        markdown_sections.push(format!("```solix\n{signature_line}\n```"));

        // 2. Memory layout and ARC representation metadata
        let mut layout_meta = Vec::new();

        // Check if symbol is a class
        if matches!(symbol.symbol_type, Type::Class { .. })
            && let Some(layout) = semantic.layouts.get(&symbol.id) {
                layout_meta.push(format!("**`sizeof`**: `{}B`", layout.instance_size_bytes));
                if layout.reference_field_offsets.is_empty() {
                    layout_meta.push("**ARC Status**: `ObjectPod` (Value/Pod payload, zero ref-fields)".to_string());
                } else {
                    layout_meta.push(format!(
                        "**ARC Status**: `Object` ({} ref-fields, {} weak)",
                        layout.reference_field_offsets.len(),
                        layout.weak_field_offsets.len()
                    ));
                }
            }

        // Check if symbol is a field within an enclosing class
        if let Some(parent_id) = symbol.parent_symbol
            && let Some(parent_layout) = semantic.layouts.get(&parent_id)
                && let Some(byte_offset) = parent_layout.field_offsets.get(&symbol.name) {
                    layout_meta.push(format!("**Field Offset**: `+{}B`", byte_offset));
                }

        if !layout_meta.is_empty() {
            markdown_sections.push(layout_meta.join(" • "));
        }

        // 3. Attached doc comments (/// and /** ... */)
        if let Some(doc_comment) = snapshot.extract_doc_comment_for_span(symbol.span) {
            markdown_sections.push(doc_comment);
        }

        let combined_markdown = markdown_sections.join("\n\n---\n\n");
        let range = span_to_range(&snapshot.source_text, symbol.span);

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: combined_markdown,
            }),
            range: Some(range),
        })
    } else if let Some(evaluated_type) = snapshot.type_at_offset(offset) {
        let markdown = format!("```solix\n{evaluated_type}\n```");
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: markdown,
            }),
            range: None,
        })
    } else {
        None
    }
}
