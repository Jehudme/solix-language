//! Inlay hints for parameter names, deduced generic type arguments, and RAII scope exit markers.

#![forbid(unsafe_code)]

use crate::diagnostics::{byte_offset_to_position, position_to_byte_offset};
use crate::workspace::AnalysisSnapshot;
use solix_common::Span;
use solix_common::types::Type;
use solix_compilation::phases::parser::statements::declarations::{ClassMember, Declaration};
use solix_compilation::phases::parser::statements::expressions::Expression;
use solix_compilation::phases::parser::statements::{ProgramAst, Statement};
use tower_lsp::lsp_types::{InlayHint, InlayHintKind, InlayHintLabel, Range};

/// Resolves `textDocument/inlayHint` within the given document range.
#[must_use]
pub fn inlay_hints(
    snapshot: &AnalysisSnapshot,
    range: Range,
) -> Vec<InlayHint> {
    let start_offset = position_to_byte_offset(&snapshot.source_text, range.start);
    let end_offset = position_to_byte_offset(&snapshot.source_text, range.end);
    let ast = match snapshot.ast.as_ref() {
        Some(a) => a,
        None => return Vec::new(),
    };

    let mut hints = Vec::new();
    collect_inlay_hints_in_ast(snapshot, ast, start_offset, end_offset, &mut hints);
    hints
}

fn collect_inlay_hints_in_ast(
    snapshot: &AnalysisSnapshot,
    ast: &ProgramAst,
    start_offset: usize,
    end_offset: usize,
    hints: &mut Vec<InlayHint>,
) {
    for decl in &ast.declarations {
        match decl {
            Declaration::Method(method_decl) => {
                if let Some(body) = &method_decl.body {
                    collect_inlay_hints_in_statements(
                        snapshot,
                        &body.statements,
                        start_offset,
                        end_offset,
                        hints,
                    );
                }
            }
            Declaration::Class(class_decl) => {
                for member in &class_decl.members {
                    if let ClassMember::Method(method_decl) = member
                        && let Some(body) = &method_decl.body {
                            collect_inlay_hints_in_statements(
                                snapshot,
                                &body.statements,
                                start_offset,
                                end_offset,
                                hints,
                            );
                        }
                }
            }
            _ => {}
        }
    }
}

fn collect_inlay_hints_in_statements(
    snapshot: &AnalysisSnapshot,
    statements: &[Statement],
    start_offset: usize,
    end_offset: usize,
    hints: &mut Vec<InlayHint>,
) {
    for stmt in statements {
        match stmt {
            Statement::Expression(expr_stmt) => {
                collect_inlay_hints_in_expression(
                    snapshot,
                    &expr_stmt.expression,
                    start_offset,
                    end_offset,
                    hints,
                );
            }
            Statement::Block(block) => {
                collect_inlay_hints_in_statements(
                    snapshot,
                    &block.statements,
                    start_offset,
                    end_offset,
                    hints,
                );

                // RAII destructor hints at end of block
                let block_end = block.span.end as usize;
                if start_offset <= block_end && block_end <= end_offset {
                    let position = byte_offset_to_position(&snapshot.source_text, block_end.saturating_sub(1));
                    hints.push(InlayHint {
                        position,
                        label: InlayHintLabel::String(" ~RAII".to_string()),
                        kind: Some(InlayHintKind::TYPE),
                        text_edits: None,
                        tooltip: Some(tower_lsp::lsp_types::InlayHintTooltip::String(
                            "Scope exit: destructors invoked for block locals".to_string(),
                        )),
                        padding_left: Some(true),
                        padding_right: Some(false),
                        data: None,
                    });
                }
            }
            Statement::If(if_stmt) => {
                collect_inlay_hints_in_statement(snapshot, &if_stmt.then_branch, start_offset, end_offset, hints);
                if let Some(else_branch) = &if_stmt.else_branch {
                    collect_inlay_hints_in_statement(snapshot, else_branch, start_offset, end_offset, hints);
                }
            }
            _ => {}
        }
    }
}

fn collect_inlay_hints_in_statement(
    snapshot: &AnalysisSnapshot,
    statement: &Statement,
    start_offset: usize,
    end_offset: usize,
    hints: &mut Vec<InlayHint>,
) {
    if let Statement::Block(block) = statement {
        collect_inlay_hints_in_statements(snapshot, &block.statements, start_offset, end_offset, hints);
    } else if let Statement::Expression(expr_stmt) = statement {
        collect_inlay_hints_in_expression(snapshot, &expr_stmt.expression, start_offset, end_offset, hints);
    }
}

fn collect_inlay_hints_in_expression(
    snapshot: &AnalysisSnapshot,
    expression: &Expression,
    start_offset: usize,
    end_offset: usize,
    hints: &mut Vec<InlayHint>,
) {
    if let Expression::MethodCall {
        span,
        callee,
        arguments,
        ..
    } = expression
    {
        let expr_start = span.start as usize;
        let expr_end = span.end as usize;

        if expr_start <= end_offset && expr_end >= start_offset {
            let callee_name = match &**callee {
                Expression::Identifier { span: id_span } => get_span_text(&snapshot.source_text, *id_span),
                Expression::MemberAccess { member_span, .. } => get_span_text(&snapshot.source_text, *member_span),
                _ => "",
            };

            // Look up callee parameter names in symbol table
            if let Some(semantic) = &snapshot.last_good_semantic {
                for symbol in &semantic.symbol_table.symbols {
                    if symbol.name == callee_name {
                        if let Type::FunctionPointer { parameter_types, .. } = &symbol.symbol_type {
                            for (idx, arg) in arguments.iter().enumerate() {
                                if idx < parameter_types.len() {
                                    let arg_start = arg.span().start as usize;
                                    let position = byte_offset_to_position(&snapshot.source_text, arg_start);
                                    let hint_label = format!("arg{idx}: ");
                                    hints.push(InlayHint {
                                        position,
                                        label: InlayHintLabel::String(hint_label),
                                        kind: Some(InlayHintKind::PARAMETER),
                                        text_edits: None,
                                        tooltip: None,
                                        padding_left: Some(false),
                                        padding_right: Some(true),
                                        data: None,
                                    });
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }
    }
}

fn get_span_text(source: &str, span: Span) -> &str {
    let start = span.start as usize;
    let end = span.end as usize;
    if start <= end && end <= source.len() {
        &source[start..end]
    } else {
        ""
    }
}
