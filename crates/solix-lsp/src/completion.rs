//! Intelligent context-aware code completions for Solix.
//!
//! Enforces strict Solix v2.0 semantics:
//! - `.` proposes only instance fields, instance methods (with generic substitutions), and postfix completions.
//!   If invoked on a type name, provides static completions with an auto-rewrite edit from `.` to `::`.
//! - `::` proposes only static methods, static `const` fields, and `enum` variants.

#![forbid(unsafe_code)]

use crate::diagnostics::position_to_byte_offset;
use crate::workspace::{AnalysisSnapshot, WorkspaceManager};
use solix_common::Span;
use solix_common::types::Type;
use solix_compilation::phases::binder::symbols::SymbolEntry;
use solix_compilation::phases::parser::statements::declarations::Declaration;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionList, CompletionParams, Documentation,
    InsertTextFormat, MarkupContent, MarkupKind, Position, Range, TextEdit,
};

/// Postfix completion templates supported in Solix.
const POSTFIX_TEMPLATES: &[(&str, &str, &str)] = &[
    ("if", "if ($EXPR) {\n    $0\n}", "Wrap in if ($EXPR)"),
    ("notnull", "if ($EXPR != null) {\n    $0\n}", "Check if ($EXPR != null)"),
    ("println", "Console::println($EXPR);", "Wrap in Console::println($EXPR)"),
    ("ret", "return $EXPR;", "Return $EXPR"),
];

/// Solix declaration keywords.
const DECLARATION_KEYWORDS: &[&str] = &[
    "class", "interface", "enum", "operator", "package", "import", "extern import", "alias",
];

/// Solix control flow keywords.
const CONTROL_FLOW_KEYWORDS: &[&str] = &[
    "if", "else", "while", "do", "for", "switch", "case", "default", "break", "continue",
    "return", "try", "catch", "finally", "throw", "assert", "exit",
];

/// Solix modifiers.
const MODIFIER_KEYWORDS: &[&str] = &[
    "public", "private", "protected", "internal", "static", "inline", "native", "const",
    "virtual", "override", "weak", "abstract", "extern",
];

/// Solix primitive types.
const PRIMITIVE_TYPES: &[&str] = &[
    "int8", "int16", "int32", "int64", "uint8", "uint16", "uint32", "uint64", "float32",
    "float64", "bool", "char", "string", "void",
];

/// Resolves code completions at the cursor position.
#[must_use]
pub fn completions(
    workspace: &WorkspaceManager,
    snapshot: &AnalysisSnapshot,
    params: &CompletionParams,
) -> CompletionList {
    let position = params.text_document_position.position;
    let offset = position_to_byte_offset(&snapshot.source_text, position);
    let line_prefix = get_line_prefix(&snapshot.source_text, offset);

    let mut items = Vec::new();

    // Check trigger context
    if line_prefix.ends_with('.') {
        // Dot (.) instance member access or type auto-rewrite
        let receiver = line_prefix.trim_end_matches('.').trim();
        let receiver_token = receiver.split(|c: char| !c.is_alphanumeric() && c != '_').next_back().unwrap_or("");

        handle_dot_completion(workspace, snapshot, receiver_token, position, &mut items);
    } else if line_prefix.ends_with("::") {
        // Scope resolution (::) static / enum access
        let receiver = line_prefix.trim_end_matches(':').trim();
        let receiver_token = receiver.split(|c: char| !c.is_alphanumeric() && c != '_').next_back().unwrap_or("");

        handle_scope_resolution_completion(snapshot, receiver_token, &mut items);
    } else {
        // General identifier / keyword / snippet completion
        handle_general_completion(workspace, snapshot, line_prefix, &mut items);
    }

    CompletionList {
        is_incomplete: false,
        items,
    }
}

/// Resolves additional documentation and text edits for a selected completion item.
#[must_use]
pub fn resolve_completion_item(
    _workspace: &WorkspaceManager,
    snapshot: &AnalysisSnapshot,
    mut item: CompletionItem,
) -> CompletionItem {
    // If documentation is missing, look it up in symbols
    if item.documentation.is_none()
        && let Some(semantic) = &snapshot.last_good_semantic {
            for symbol in &semantic.symbol_table.symbols {
                if symbol.name == item.label
                    && let Some(doc) = snapshot.extract_doc_comment_for_span(symbol.span) {
                        item.documentation = Some(Documentation::MarkupContent(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: doc,
                        }));
                        break;
                    }
            }
        }

    item
}

fn handle_dot_completion(
    _workspace: &WorkspaceManager,
    snapshot: &AnalysisSnapshot,
    receiver: &str,
    position: Position,
    items: &mut Vec<CompletionItem>,
) {
    let semantic = snapshot.last_good_semantic.as_ref();

    // 1. Check if receiver is a known Class Name or Type Name
    let mut is_type_name = false;
    if let Some(sem) = semantic {
        for symbol in &sem.symbol_table.symbols {
            if symbol.name == receiver
                && let Type::Class { symbol_name, .. } = &symbol.symbol_type
                    && (symbol.name == *symbol_name || symbol.fully_qualified_name == *symbol_name) {
                        is_type_name = true;
                        // Propose static members with auto-rewrite from . to ::
                        for member_id in &symbol.members {
                            if let Some(member) = sem.symbol_table.get_symbol(*member_id)
                                && member.is_static {
                                    let mut item = make_symbol_completion_item(member);
                                    let dot_position = Position {
                                        line: position.line,
                                        character: position.character.saturating_sub(1),
                                    };
                                    let replace_range = Range {
                                        start: dot_position,
                                        end: position,
                                    };
                                    item.additional_text_edits = Some(vec![TextEdit {
                                        range: replace_range,
                                        new_text: "::".to_string(),
                                    }]);
                                    item.detail = Some(format!("(Static) {}", member.to_signature_string()));
                                    items.push(item);
                                }
                        }
                        break;
                    }
        }
    }

    if is_type_name {
        return;
    }

    // 2. Receiver is an instance: suggest instance methods and fields
    if let Some(sem) = semantic {
        // Attempt to find receiver's evaluated type
        let mut receiver_type = None;

        if receiver == "this" {
            let target_offset = crate::diagnostics::position_to_byte_offset(&snapshot.source_text, position) as u32;
            for symbol in &sem.symbol_table.symbols {
                if symbol.source_id == snapshot.source_id
                    && matches!(symbol.symbol_type, Type::Class { .. })
                    && symbol.span.start <= target_offset
                    && target_offset <= symbol.span.end
                {
                    for member_id in &symbol.members {
                        if let Some(member) = sem.symbol_table.get_symbol(*member_id)
                            && !member.is_static {
                                items.push(make_symbol_completion_item(member));
                            }
                    }
                    return;
                }
            }
        }

        // Check local symbols recorded in symbol table
        for local_symbol in &sem.symbol_table.local_symbols {
            if local_symbol.source_id == snapshot.source_id
                && local_symbol.name == receiver
            {
                receiver_type = Some(local_symbol.symbol_type.clone());
                break;
            }
        }

        // Check declared instance symbols
        if receiver_type.is_none() {
            for symbol in &sem.symbol_table.symbols {
                if symbol.name == receiver && !symbol.is_static {
                    receiver_type = Some(symbol.symbol_type.clone());
                    break;
                }
            }
        }

        // Fallback: search AST for local variable declaration if semantic pass was incomplete
        if receiver_type.is_none()
            && let Some(ref program_ast) = snapshot.ast
        {
            for declaration in &program_ast.declarations {
                if let Declaration::Method(method_declaration) = declaration
                    && let Some(ref method_body) = method_declaration.body
                {
                    for statement in &method_body.statements {
                        if let solix_compilation::phases::parser::statements::Statement::VariableDeclaration(var_decl) = statement {
                            let declared_var_name = if (var_decl.name_span.end as usize) <= snapshot.source_text.len() {
                                &snapshot.source_text[var_decl.name_span.start as usize..var_decl.name_span.end as usize]
                            } else {
                                ""
                            };
                            if declared_var_name == receiver {
                                let resolved_type = solix_compilation::phases::binder::type_resolver::resolve_type_annotation(
                                    snapshot.source_id,
                                    &snapshot.source_text,
                                    &var_decl.var_type,
                                    &sem.symbol_table,
                                );
                                receiver_type = Some(resolved_type);
                                break;
                            }
                        }
                    }
                }
            }
        }

        if let Some(Type::Class { symbol_name, .. }) = receiver_type {
            for symbol in &sem.symbol_table.symbols {
                if (symbol.name == symbol_name || symbol.fully_qualified_name == symbol_name)
                    && matches!(symbol.symbol_type, Type::Class { .. })
                {
                    for member_id in &symbol.members {
                        if let Some(member) = sem.symbol_table.get_symbol(*member_id)
                            && !member.is_static {
                                items.push(make_symbol_completion_item(member));
                            }
                    }
                    break;
                }
            }
        }
    }

    // 3. Postfix completions
    if !receiver.is_empty() {
        for (name, template, description) in POSTFIX_TEMPLATES {
            let replaced_template = template.replace("$EXPR", receiver);
            let start_col = position.character.saturating_sub((receiver.len() + 1) as u32);
            let replace_range = Range {
                start: Position {
                    line: position.line,
                    character: start_col,
                },
                end: position,
            };

            items.push(CompletionItem {
                label: format!(".{name}"),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: Some(description.to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("```solix\n{replaced_template}\n```"),
                })),
                insert_text: Some(replaced_template),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                text_edit: Some(tower_lsp::lsp_types::CompletionTextEdit::Edit(TextEdit {
                    range: replace_range,
                    new_text: template.replace("$EXPR", receiver),
                })),
                ..Default::default()
            });
        }
    }
}

fn handle_scope_resolution_completion(
    snapshot: &AnalysisSnapshot,
    receiver: &str,
    items: &mut Vec<CompletionItem>,
) {
    let semantic = match snapshot.last_good_semantic.as_ref() {
        Some(sem) => sem,
        None => return,
    };

    // Look up the receiver symbol
    for symbol in &semantic.symbol_table.symbols {
        if symbol.name == receiver || symbol.fully_qualified_name == receiver {
            // Case 1: Class with static methods and const fields
            for member_id in &symbol.members {
                if let Some(member) = semantic.symbol_table.get_symbol(*member_id)
                    && (member.is_static || member.is_const) {
                        items.push(make_symbol_completion_item(member));
                    }
            }
            return;
        }
    }

    // Case 2: Package namespace prefix (e.g. `solix::io::`)
    for symbol in &semantic.symbol_table.symbols {
        if symbol.fully_qualified_name.starts_with(receiver) && symbol.name != receiver {
            items.push(make_symbol_completion_item(symbol));
        }
    }
}

fn handle_general_completion(
    workspace: &WorkspaceManager,
    snapshot: &AnalysisSnapshot,
    line_prefix: &str,
    items: &mut Vec<CompletionItem>,
) {
    let trimmed = line_prefix.trim_start();

    // Contextual keywords
    if trimmed.starts_with("package ") {
        // Suggest project package name
        items.push(CompletionItem {
            label: "main;".to_string(),
            kind: Some(CompletionItemKind::MODULE),
            detail: Some("Main package declaration".to_string()),
            ..Default::default()
        });
        return;
    }

    if trimmed.starts_with("import ") || trimmed.starts_with("extern import ") {
        // Suggest known packages and types
        items.push(CompletionItem {
            label: "solix.io.Console;".to_string(),
            kind: Some(CompletionItemKind::MODULE),
            detail: Some("Standard console I/O".to_string()),
            ..Default::default()
        });
        for other_snap in workspace.all_snapshots() {
            if let Some(ast) = &other_snap.ast {
                for decl in &ast.declarations {
                    if let Declaration::Class(c) = decl {
                        let name = get_span_text(&other_snap.source_text, c.name_span);
                        items.push(CompletionItem {
                            label: format!("{name};"),
                            kind: Some(CompletionItemKind::CLASS),
                            detail: Some(format!("Import {name}")),
                            ..Default::default()
                        });
                    }
                }
            }
        }
        return;
    }

    // Add local variables, parameters, and declared symbols in scope
    if let Some(semantic) = &snapshot.last_good_semantic {
        for symbol in &semantic.symbol_table.symbols {
            if symbol.source_id == snapshot.source_id || symbol.is_public() {
                items.push(make_symbol_completion_item(symbol));
            }
        }
    }

    // Add Solix keywords
    for keyword in DECLARATION_KEYWORDS {
        items.push(CompletionItem {
            label: (*keyword).to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..Default::default()
        });
    }

    for keyword in CONTROL_FLOW_KEYWORDS {
        items.push(CompletionItem {
            label: (*keyword).to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..Default::default()
        });
    }

    for modifier in MODIFIER_KEYWORDS {
        items.push(CompletionItem {
            label: (*modifier).to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..Default::default()
        });
    }

    for primitive in PRIMITIVE_TYPES {
        items.push(CompletionItem {
            label: (*primitive).to_string(),
            kind: Some(CompletionItemKind::TYPE_PARAMETER),
            ..Default::default()
        });
    }
}

fn make_symbol_completion_item(symbol: &SymbolEntry) -> CompletionItem {
    let (kind, detail) = match &symbol.symbol_type {
        Type::FunctionPointer { .. } => (
            CompletionItemKind::METHOD,
            Some(symbol.to_signature_string()),
        ),
        Type::Class { .. } => (
            CompletionItemKind::CLASS,
            Some(format!("class {}", symbol.name)),
        ),
        _ => (
            CompletionItemKind::FIELD,
            Some(format!("{} {}", symbol.symbol_type, symbol.name)),
        ),
    };

    CompletionItem {
        label: symbol.name.clone(),
        kind: Some(kind),
        detail,
        ..Default::default()
    }
}

fn get_line_prefix(source_text: &str, offset: usize) -> &str {
    let clamped = offset.min(source_text.len());
    let prefix = &source_text[..clamped];
    let last_newline = prefix.rfind('\n').map_or(0, |idx| idx + 1);
    &prefix[last_newline..]
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
