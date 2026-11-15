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
    let semantic = snapshot.last_good_semantic.as_ref()?;

    // 1. Search for enclosing MethodCall in AST if AST is present
    let mut best_call: Option<(Span, String, usize)> = None;
    if let Some(ast) = snapshot.ast.as_ref() {
        find_enclosing_call_in_ast(ast, target_offset, &snapshot.source_text, &mut best_call);
    }

    // 2. Fall back to resilient lexical scan if AST does not contain an enclosing call (e.g. while actively typing)
    let (callee_name, qualifier_name, active_parameter_index) = match best_call {
        Some((_, call_callee, param_index)) => (call_callee, None, param_index),
        None => find_enclosing_call_lexical(&snapshot.source_text, offset)?,
    };

    // Find all matching symbols for callee_name (overloads, methods, and constructors)
    let mut signatures = Vec::new();

    // Try finding matching symbols under qualified class first if qualifier is present
    if let Some(ref qualifier) = qualifier_name {
        for class_symbol in &semantic.symbol_table.symbols {
            if class_symbol.name == *qualifier
                && matches!(class_symbol.symbol_type, Type::Class { .. })
            {
                for member_id in &class_symbol.members {
                    if let Some(member) = semantic.symbol_table.get_symbol(*member_id)
                        && member.name == callee_name
                        && let Type::FunctionPointer {
                            return_type,
                            parameter_types,
                        } = &member.symbol_type
                    {
                        signatures.push(build_signature_info(
                            member,
                            &callee_name,
                            return_type,
                            parameter_types,
                            active_parameter_index,
                            snapshot,
                        ));
                    }
                }
            }
        }
    }

    // Search general symbol table if no qualified symbols were found
    if signatures.is_empty() {
        for symbol in &semantic.symbol_table.symbols {
            if symbol.name == callee_name
                && let Type::FunctionPointer {
                    return_type,
                    parameter_types,
                } = &symbol.symbol_type
            {
                signatures.push(build_signature_info(
                    symbol,
                    &callee_name,
                    return_type,
                    parameter_types,
                    active_parameter_index,
                    snapshot,
                ));
            } else if symbol.name == "constructor"
                && let Some(parent_id) = symbol.parent_symbol
                && let Some(parent_class) = semantic.symbol_table.get_symbol(parent_id)
                && parent_class.name == callee_name
                && let Type::FunctionPointer {
                    return_type,
                    parameter_types,
                } = &symbol.symbol_type
            {
                signatures.push(build_signature_info(
                    symbol,
                    &callee_name,
                    return_type,
                    parameter_types,
                    active_parameter_index,
                    snapshot,
                ));
            }
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

fn build_signature_info(
    symbol: &solix_compilation::phases::binder::symbols::SymbolEntry,
    display_name: &str,
    return_type: &Type,
    parameter_types: &[Type],
    active_parameter_index: usize,
    snapshot: &AnalysisSnapshot,
) -> SignatureInformation {
    let mut parameter_infos = Vec::new();
    let mut parameter_strings = Vec::new();

    for (parameter_index, parameter_type) in parameter_types.iter().enumerate() {
        let parameter_name = symbol
            .parameter_names
            .get(parameter_index)
            .map(String::as_str)
            .filter(|name| !name.is_empty())
            .unwrap_or("arg");
        let label = format!("{parameter_type} {parameter_name}");
        parameter_strings.push(label.clone());
        parameter_infos.push(ParameterInformation {
            label: ParameterLabel::Simple(label),
            documentation: None,
        });
    }

    let full_label = if display_name == "constructor" || symbol.name == "constructor" {
        format!("{display_name}({})", parameter_strings.join(", "))
    } else {
        format!(
            "{return_type} {display_name}({})",
            parameter_strings.join(", ")
        )
    };

    let mut documentation = None;
    if let Some(doc_text) = snapshot.extract_doc_comment_for_span(symbol.span) {
        documentation = Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: doc_text,
        }));
    }

    SignatureInformation {
        label: full_label,
        documentation,
        parameters: Some(parameter_infos),
        active_parameter: Some(active_parameter_index as u32),
    }
}

fn find_enclosing_call_lexical(
    source_text: &str,
    target_offset: usize,
) -> Option<(String, Option<String>, usize)> {
    let clamped_offset = target_offset.min(source_text.len());
    let prefix_text = &source_text[..clamped_offset];

    let mut paren_depth = 0i32;
    let mut comma_count = 0usize;
    let mut open_paren_offset: Option<usize> = None;

    for (byte_index, character) in prefix_text.char_indices().rev() {
        if character == ')' {
            paren_depth += 1;
        } else if character == '(' {
            if paren_depth > 0 {
                paren_depth -= 1;
            } else {
                open_paren_offset = Some(byte_index);
                break;
            }
        } else if character == ',' && paren_depth == 0 {
            comma_count += 1;
        } else if (character == ';' || character == '{' || character == '}') && paren_depth == 0 {
            break;
        }
    }

    let paren_byte_offset = open_paren_offset?;
    let before_paren = source_text[..paren_byte_offset].trim_end();

    let mut identifier_start = before_paren.len();
    for (byte_index, character) in before_paren.char_indices().rev() {
        if character.is_alphanumeric() || character == '_' {
            identifier_start = byte_index;
        } else {
            break;
        }
    }

    if identifier_start >= before_paren.len() {
        return None;
    }

    let callee_name = before_paren[identifier_start..].to_string();
    let before_callee = before_paren[..identifier_start].trim_end();

    let qualifier_name = if let Some(stripped) = before_callee.strip_suffix("::") {
        let trimmed_qualifier = stripped.trim_end();
        let qualifier_start = trimmed_qualifier
            .rfind(|character: char| !character.is_alphanumeric() && character != '_')
            .map_or(0, |index| index + 1);
        Some(trimmed_qualifier[qualifier_start..].to_string())
    } else if let Some(stripped) = before_callee.strip_suffix('.') {
        let trimmed_qualifier = stripped.trim_end();
        let qualifier_start = trimmed_qualifier
            .rfind(|character: char| !character.is_alphanumeric() && character != '_')
            .map_or(0, |index| index + 1);
        Some(trimmed_qualifier[qualifier_start..].to_string())
    } else {
        None
    };

    Some((callee_name, qualifier_name, comma_count))
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
