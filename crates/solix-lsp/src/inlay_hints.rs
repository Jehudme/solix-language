//! Inlay hints for parameter names, deduced generic type arguments, and RAII scope exit markers.

#![forbid(unsafe_code)]

use crate::diagnostics::{byte_offset_to_position, position_to_byte_offset};
use crate::workspace::AnalysisSnapshot;
use solix_common::Span;
use solix_common::types::Type;
use solix_compilation::phases::parser::statements::declarations::{ClassMember, Declaration};
use solix_compilation::phases::parser::statements::expressions::Expression;
use solix_compilation::phases::parser::statements::{ProgramAst, Statement, TypeAnnotation};
use tower_lsp::lsp_types::{InlayHint, InlayHintKind, InlayHintLabel, Range};

/// Resolves `textDocument/inlayHint` within the given document range.
#[must_use]
pub fn inlay_hints(snapshot: &AnalysisSnapshot, range: Range) -> Vec<InlayHint> {
    let start_offset = position_to_byte_offset(&snapshot.source_text, range.start);
    let end_offset = position_to_byte_offset(&snapshot.source_text, range.end);
    let ast = match snapshot.ast.as_ref() {
        Some(parsed_ast) => parsed_ast,
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
                        && let Some(body) = &method_decl.body
                    {
                        collect_inlay_hints_in_statements(
                            snapshot,
                            &body.statements,
                            start_offset,
                            end_offset,
                            hints,
                        );
                    } else if let ClassMember::Constructor(constructor_decl) = member {
                        collect_inlay_hints_in_statements(
                            snapshot,
                            &constructor_decl.body.statements,
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
    for statement in statements {
        collect_inlay_hints_in_statement(snapshot, statement, start_offset, end_offset, hints);

        if let Statement::Block(block) = statement {
            let block_end = block.span.end as usize;
            if start_offset <= block_end && block_end <= end_offset {
                let position =
                    byte_offset_to_position(&snapshot.source_text, block_end.saturating_sub(1));
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
    }
}

fn collect_inlay_hints_in_statement(
    snapshot: &AnalysisSnapshot,
    statement: &Statement,
    start_offset: usize,
    end_offset: usize,
    hints: &mut Vec<InlayHint>,
) {
    match statement {
        Statement::Block(block) => {
            collect_inlay_hints_in_statements(
                snapshot,
                &block.statements,
                start_offset,
                end_offset,
                hints,
            );
        }
        Statement::Expression(expr_stmt) => {
            collect_inlay_hints_in_expression(
                snapshot,
                &expr_stmt.expression,
                start_offset,
                end_offset,
                hints,
            );
        }
        Statement::VariableDeclaration(var_decl) => {
            if let Some(initializer) = &var_decl.initializer {
                collect_inlay_hints_in_expression(
                    snapshot,
                    initializer,
                    start_offset,
                    end_offset,
                    hints,
                );
            }
        }
        Statement::If(if_stmt) => {
            collect_inlay_hints_in_expression(
                snapshot,
                &if_stmt.condition,
                start_offset,
                end_offset,
                hints,
            );
            collect_inlay_hints_in_statement(
                snapshot,
                &if_stmt.then_branch,
                start_offset,
                end_offset,
                hints,
            );
            if let Some(else_branch) = &if_stmt.else_branch {
                collect_inlay_hints_in_statement(
                    snapshot,
                    else_branch,
                    start_offset,
                    end_offset,
                    hints,
                );
            }
        }
        Statement::While(while_stmt) => {
            collect_inlay_hints_in_expression(
                snapshot,
                &while_stmt.condition,
                start_offset,
                end_offset,
                hints,
            );
            collect_inlay_hints_in_statement(
                snapshot,
                &while_stmt.body,
                start_offset,
                end_offset,
                hints,
            );
        }
        Statement::DoWhile(do_while_stmt) => {
            collect_inlay_hints_in_statement(
                snapshot,
                &do_while_stmt.body,
                start_offset,
                end_offset,
                hints,
            );
            collect_inlay_hints_in_expression(
                snapshot,
                &do_while_stmt.condition,
                start_offset,
                end_offset,
                hints,
            );
        }
        Statement::For(for_stmt) => {
            if let Some(initializer) = &for_stmt.initializer {
                collect_inlay_hints_in_statement(
                    snapshot,
                    initializer,
                    start_offset,
                    end_offset,
                    hints,
                );
            }
            if let Some(condition) = &for_stmt.condition {
                collect_inlay_hints_in_expression(
                    snapshot,
                    condition,
                    start_offset,
                    end_offset,
                    hints,
                );
            }
            if let Some(update) = &for_stmt.update {
                collect_inlay_hints_in_expression(
                    snapshot,
                    update,
                    start_offset,
                    end_offset,
                    hints,
                );
            }
            collect_inlay_hints_in_statement(
                snapshot,
                &for_stmt.body,
                start_offset,
                end_offset,
                hints,
            );
        }
        Statement::Return(return_stmt) => {
            if let Some(return_value) = &return_stmt.value {
                collect_inlay_hints_in_expression(
                    snapshot,
                    return_value,
                    start_offset,
                    end_offset,
                    hints,
                );
            }
        }
        _ => {}
    }
}

fn collect_inlay_hints_in_expression(
    snapshot: &AnalysisSnapshot,
    expression: &Expression,
    start_offset: usize,
    end_offset: usize,
    hints: &mut Vec<InlayHint>,
) {
    match expression {
        Expression::MethodCall {
            span,
            callee,
            arguments,
            ..
        } => {
            let expr_start = span.start as usize;
            let expr_end = span.end as usize;

            if expr_start <= end_offset && expr_end >= start_offset {
                let callee_name = match &**callee {
                    Expression::Identifier { span: id_span } => {
                        get_span_text(&snapshot.source_text, *id_span)
                    }
                    Expression::MemberAccess { member_span, .. } => {
                        get_span_text(&snapshot.source_text, *member_span)
                    }
                    _ => "",
                };

                // Look up callee parameter names in symbol table
                if let Some(semantic) = &snapshot.last_good_semantic {
                    let matching_symbol = semantic
                        .symbol_table
                        .symbols
                        .iter()
                        .filter(|symbol| symbol.name == callee_name)
                        .find(|symbol| {
                            if let Type::FunctionPointer {
                                parameter_types, ..
                            } = &symbol.symbol_type
                            {
                                parameter_types.len() == arguments.len()
                            } else {
                                false
                            }
                        })
                        .or_else(|| {
                            semantic
                                .symbol_table
                                .symbols
                                .iter()
                                .find(|symbol| symbol.name == callee_name)
                        });

                    if let Some(symbol) = matching_symbol
                        && let Type::FunctionPointer {
                            parameter_types, ..
                        } = &symbol.symbol_type
                    {
                        for (index, argument) in arguments.iter().enumerate() {
                            if index < parameter_types.len() {
                                let arg_start = argument.span().start as usize;
                                let position =
                                    byte_offset_to_position(&snapshot.source_text, arg_start);
                                let param_name = symbol
                                    .parameter_names
                                    .get(index)
                                    .map(String::as_str)
                                    .filter(|name| !name.is_empty())
                                    .unwrap_or("arg");
                                let hint_label = format!("{param_name}: ");
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
                }
            }

            // Recurse into callee and arguments
            collect_inlay_hints_in_expression(snapshot, callee, start_offset, end_offset, hints);
            for argument in arguments {
                collect_inlay_hints_in_expression(
                    snapshot,
                    argument,
                    start_offset,
                    end_offset,
                    hints,
                );
            }
        }
        Expression::NewInstance {
            span,
            instance_type,
            arguments,
        } => {
            let expr_start = span.start as usize;
            let expr_end = span.end as usize;

            if expr_start <= end_offset && expr_end >= start_offset {
                let type_name = match instance_type {
                    TypeAnnotation::Named { path, .. } => path
                        .last()
                        .map(|span| get_span_text(&snapshot.source_text, *span))
                        .unwrap_or(""),
                    TypeAnnotation::Generic { base, .. } => {
                        if let TypeAnnotation::Named { path, .. } = &**base {
                            path.last()
                                .map(|span| get_span_text(&snapshot.source_text, *span))
                                .unwrap_or("")
                        } else {
                            ""
                        }
                    }
                    _ => "",
                };

                if let Some(semantic) = &snapshot.last_good_semantic
                    && !type_name.is_empty()
                {
                    let class_symbol = semantic
                        .symbol_table
                        .symbols
                        .iter()
                        .find(|symbol| symbol.name == type_name);

                    if let Some(class_sym) = class_symbol {
                        let constructor_symbol = semantic
                            .symbol_table
                            .symbols
                            .iter()
                            .filter(|symbol| {
                                symbol.parent_symbol == Some(class_sym.id)
                                    && symbol.name == "constructor"
                            })
                            .find(|symbol| {
                                if let Type::FunctionPointer {
                                    parameter_types, ..
                                } = &symbol.symbol_type
                                {
                                    parameter_types.len() == arguments.len()
                                } else {
                                    false
                                }
                            })
                            .or_else(|| {
                                semantic.symbol_table.symbols.iter().find(|symbol| {
                                    symbol.parent_symbol == Some(class_sym.id)
                                        && symbol.name == "constructor"
                                })
                            });

                        if let Some(constructor) = constructor_symbol
                            && let Type::FunctionPointer {
                                parameter_types, ..
                            } = &constructor.symbol_type
                        {
                            for (index, argument) in arguments.iter().enumerate() {
                                if index < parameter_types.len() {
                                    let arg_start = argument.span().start as usize;
                                    let position =
                                        byte_offset_to_position(&snapshot.source_text, arg_start);
                                    let param_name = constructor
                                        .parameter_names
                                        .get(index)
                                        .map(String::as_str)
                                        .filter(|name| !name.is_empty())
                                        .unwrap_or("arg");
                                    let hint_label = format!("{param_name}: ");
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
                    }
                }
            }

            for argument in arguments {
                collect_inlay_hints_in_expression(
                    snapshot,
                    argument,
                    start_offset,
                    end_offset,
                    hints,
                );
            }
        }
        Expression::Assignment { target, value, .. } => {
            collect_inlay_hints_in_expression(snapshot, target, start_offset, end_offset, hints);
            collect_inlay_hints_in_expression(snapshot, value, start_offset, end_offset, hints);
        }
        Expression::Binary { left, right, .. } => {
            collect_inlay_hints_in_expression(snapshot, left, start_offset, end_offset, hints);
            collect_inlay_hints_in_expression(snapshot, right, start_offset, end_offset, hints);
        }
        Expression::Unary { operand, .. } => {
            collect_inlay_hints_in_expression(snapshot, operand, start_offset, end_offset, hints);
        }
        Expression::MemberAccess { object, .. } => {
            collect_inlay_hints_in_expression(snapshot, object, start_offset, end_offset, hints);
        }
        Expression::ArrayAccess { array, index, .. } => {
            collect_inlay_hints_in_expression(snapshot, array, start_offset, end_offset, hints);
            collect_inlay_hints_in_expression(snapshot, index, start_offset, end_offset, hints);
        }
        Expression::ArrayLiteral { elements, .. } => {
            for element in elements {
                collect_inlay_hints_in_expression(
                    snapshot,
                    element,
                    start_offset,
                    end_offset,
                    hints,
                );
            }
        }
        _ => {}
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
