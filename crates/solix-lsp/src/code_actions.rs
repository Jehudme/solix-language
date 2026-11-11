//! Code actions: Quick Fixes (`.` to `::`, typo fixes, auto-imports, missing overrides) and Refactorings.

#![forbid(unsafe_code)]

use crate::diagnostics::span_to_range;
use crate::workspace::{AnalysisSnapshot, WorkspaceManager};
use solix_common::Span;
use solix_compilation::phases::parser::statements::declarations::Declaration;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams, NumberOrString, Position,
    Range, TextEdit, WorkspaceEdit,
};

/// Resolves `textDocument/codeAction` for the given diagnostics and cursor range.
#[must_use]
pub fn code_actions(
    workspace: &WorkspaceManager,
    snapshot: &AnalysisSnapshot,
    params: &CodeActionParams,
) -> Vec<CodeActionOrCommand> {
    let mut actions = Vec::new();

    // 1. Diagnostics-driven Quick Fixes
    for diagnostic in &params.context.diagnostics {
        let code_str = match &diagnostic.code {
            Some(NumberOrString::String(s)) => s.as_str(),
            _ => "",
        };

        // Quick Fix 1: E0226 - Replace '.' with '::' on static member access
        if code_str == "E0226" || diagnostic.message.contains("static") && diagnostic.message.contains("via '.'") {
            let edit_range = diagnostic.range;
            let mut changes = std::collections::HashMap::new();
            changes.insert(
                snapshot.uri.clone(),
                vec![TextEdit {
                    range: edit_range,
                    new_text: "::".to_string(),
                }],
            );

            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Replace '.' with '::'".to_string(),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: Some(vec![diagnostic.clone()]),
                edit: Some(WorkspaceEdit {
                    changes: Some(changes),
                    ..Default::default()
                }),
                is_preferred: Some(true),
                ..Default::default()
            }));
        }

        // Quick Fix 2: E0201 - Unresolved symbol typo fix ("Did you mean ...?") & Auto-import
        if code_str == "E0201" || diagnostic.message.contains("Unresolved symbol") {
            let unresolved_name = extract_symbol_name_from_message(&diagnostic.message);
            if !unresolved_name.is_empty() {
                // Check for auto-import candidates across workspace
                for other_snap in workspace.all_snapshots() {
                    if let Some(ast) = &other_snap.ast {
                        for decl in &ast.declarations {
                            if let Declaration::Class(c) = decl {
                                let class_name = get_span_text(&other_snap.source_text, c.name_span);
                                if class_name == unresolved_name {
                                    let mut changes = std::collections::HashMap::new();
                                    changes.insert(
                                        snapshot.uri.clone(),
                                        vec![TextEdit {
                                            range: Range {
                                                start: Position { line: 0, character: 0 },
                                                end: Position { line: 0, character: 0 },
                                            },
                                            new_text: format!("import {class_name};\n"),
                                        }],
                                    );

                                    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                        title: format!("Import '{class_name}'"),
                                        kind: Some(CodeActionKind::QUICKFIX),
                                        diagnostics: Some(vec![diagnostic.clone()]),
                                        edit: Some(WorkspaceEdit {
                                            changes: Some(changes),
                                            ..Default::default()
                                        }),
                                        is_preferred: Some(true),
                                        ..Default::default()
                                    }));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Quick Fix 3: E0284 - Missing 'override' keyword
        if code_str == "E0284" || diagnostic.message.contains("override") {
            let mut changes = std::collections::HashMap::new();
            changes.insert(
                snapshot.uri.clone(),
                vec![TextEdit {
                    range: Range {
                        start: diagnostic.range.start,
                        end: diagnostic.range.start,
                    },
                    new_text: "override ".to_string(),
                }],
            );

            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Add 'override' modifier".to_string(),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: Some(vec![diagnostic.clone()]),
                edit: Some(WorkspaceEdit {
                    changes: Some(changes),
                    ..Default::default()
                }),
                is_preferred: Some(true),
                ..Default::default()
            }));
        }
    }

    // 2. Refactorings: Organize Imports
    if let Some(ast) = &snapshot.ast
        && ast.imports.len() > 1 {
            let mut import_lines: Vec<String> = ast
                .imports
                .iter()
                .map(|imp| get_span_text(&snapshot.source_text, imp.span).trim().to_string())
                .collect();
            import_lines.sort();
            import_lines.dedup();

            let first_import = &ast.imports[0];
            let last_import = &ast.imports[ast.imports.len() - 1];
            let replace_range = span_to_range(
                &snapshot.source_text,
                first_import.span.combine(last_import.span),
            );

            let mut changes = std::collections::HashMap::new();
            changes.insert(
                snapshot.uri.clone(),
                vec![TextEdit {
                    range: replace_range,
                    new_text: import_lines.join("\n"),
                }],
            );

            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Organize Imports".to_string(),
                kind: Some(CodeActionKind::SOURCE_ORGANIZE_IMPORTS),
                diagnostics: None,
                edit: Some(WorkspaceEdit {
                    changes: Some(changes),
                    ..Default::default()
                }),
                is_preferred: Some(false),
                ..Default::default()
            }));
        }

    actions
}

fn extract_symbol_name_from_message(msg: &str) -> &str {
    if let Some(start) = msg.find('\'')
        && let Some(end) = msg[start + 1..].find('\'') {
            return &msg[start + 1..start + 1 + end];
        }
    ""
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
