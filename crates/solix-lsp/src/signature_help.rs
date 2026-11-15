//! Overload-aware signature help for functions, methods, and constructors.

#![forbid(unsafe_code)]

use crate::diagnostics::position_to_byte_offset;
use crate::workspace::AnalysisSnapshot;
use solix_common::Span;
use solix_common::types::Type;
use solix_compilation::phases::parser::statements::declarations::{ClassMember, Declaration};
use solix_compilation::phases::parser::statements::expressions::Expression;
use solix_compilation::phases::parser::statements::{ProgramAst, Statement};
use tower_lsp::lsp_types::{
    Documentation, MarkupContent, MarkupKind, ParameterInformation, ParameterLabel, Position,
    SignatureHelp, SignatureInformation,
};

/// Resolves `textDocument/signatureHelp` at the given cursor position.
#[must_use]
pub fn signature_help(snapshot: &AnalysisSnapshot, position: Position) -> Option<SignatureHelp> {
    let offset = position_to_byte_offset(&snapshot.source_text, position);
    let target_offset = offset as u32;
    let ast = snapshot.ast.as_ref()?;
    let semantic = snapshot.last_good_semantic.as_ref()?;

    // Search for enclosing MethodCall in AST
    let mut best_call: Option<(Span, String, usize)> = None;
    find_enclosing_call_in_ast(ast, target_offset, &snapshot.source_text, &mut best_call);

    let (_, callee_name, active_parameter_index) = best_call?;

    // Find all matching symbols for callee_name (overloads)
    let mut signatures = Vec::new();
    for symbol in &semantic.symbol_table.symbols {
        if symbol.name == callee_name
            && let Type::FunctionPointer {
                return_type,
                parameter_types,
            } = &symbol.symbol_type
        {
            let mut param_infos = Vec::new();
            let mut param_strings = Vec::new();

            for (param_idx, param_type) in parameter_types.iter().enumerate() {
                let label = format!("{param_type} arg{param_idx}");
                param_strings.push(label.clone());
                param_infos.push(ParameterInformation {
                    label: ParameterLabel::Simple(label),
                    documentation: None,
                });
            }

            let full_label = format!("{return_type} {callee_name}({})", param_strings.join(", "));
            let mut doc = None;
            if let Some(doc_text) = snapshot.extract_doc_comment_for_span(symbol.span) {
                doc = Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: doc_text,
                }));
            }

            signatures.push(SignatureInformation {
                label: full_label,
                documentation: doc,
                parameters: Some(param_infos),
                active_parameter: Some(active_parameter_index as u32),
            });
        }
    }

    if signatures.is_empty() {
        None
    } else {
        Some(SignatureHelp {
            signatures,
            active_signature: Some(0),
            active_parameter: Some(active_parameter_index as u32),
        })
    }
}

fn find_enclosing_call_in_ast(
    ast: &ProgramAst,
    target_offset: u32,
    source_text: &str,
    best_call: &mut Option<(Span, String, usize)>,
) {
    for decl in &ast.declarations {
        match decl {
            Declaration::Method(method_decl) => {
                if let Some(body) = &method_decl.body {
                    find_enclosing_call_in_statements(
                        &body.statements,
                        target_offset,
                        source_text,
                        best_call,
                    );
                }
            }
            Declaration::Class(class_decl) => {
                for member in &class_decl.members {
                    if let ClassMember::Method(method_decl) = member
                        && let Some(body) = &method_decl.body
                    {
                        find_enclosing_call_in_statements(
                            &body.statements,
                            target_offset,
                            source_text,
                            best_call,
                        );
                    }
                }
            }
            _ => {}
        }
    }
}

fn find_enclosing_call_in_statements(
    statements: &[Statement],
    target_offset: u32,
    source_text: &str,
    best_call: &mut Option<(Span, String, usize)>,
) {
    for stmt in statements {
        match stmt {
            Statement::Expression(expr_stmt) => {
                find_enclosing_call_in_expression(
                    &expr_stmt.expression,
                    target_offset,
                    source_text,
                    best_call,
                );
            }
            Statement::Block(block) => {
                find_enclosing_call_in_statements(
                    &block.statements,
                    target_offset,
                    source_text,
                    best_call,
                );
            }
            _ => {}
        }
    }
}

fn find_enclosing_call_in_expression(
    expression: &Expression,
    target_offset: u32,
    source_text: &str,
    best_call: &mut Option<(Span, String, usize)>,
) {
    if let Expression::MethodCall {
        span,
        callee,
        arguments,
        ..
    } = expression
        && span.start <= target_offset
        && target_offset <= span.end
    {
        let callee_name = match &**callee {
            Expression::Identifier { span: id_span } => get_span_text(source_text, *id_span),
            Expression::MemberAccess { member_span, .. } => {
                get_span_text(source_text, *member_span)
            }
            _ => "",
        };

        // Calculate active parameter based on comma counts before target_offset
        let mut active_param = 0;
        for (idx, arg) in arguments.iter().enumerate() {
            if target_offset > arg.span().end {
                active_param = idx + 1;
            }
        }

        *best_call = Some((*span, callee_name.to_string(), active_param));
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
