//! CodeLens actions for running main/test functions and showing reference counts.

#![forbid(unsafe_code)]

use crate::diagnostics::span_to_range;
use crate::workspace::{AnalysisSnapshot, WorkspaceManager};
use solix_common::Span;
use solix_compilation::phases::parser::statements::declarations::{ClassMember, Declaration, Modifier};
use tower_lsp::lsp_types::{CodeLens, Command};

/// Resolves `textDocument/codeLens` for entry points and references.
#[must_use]
pub fn code_lenses(
    workspace: &WorkspaceManager,
    snapshot: &AnalysisSnapshot,
) -> Vec<CodeLens> {
    let ast = match snapshot.ast.as_ref() {
        Some(a) => a,
        None => return Vec::new(),
    };

    let mut lenses = Vec::new();

    for decl in &ast.declarations {
        match decl {
            Declaration::Method(method_decl) => {
                let name = get_span_text(&snapshot.source_text, method_decl.name_span);
                let range = span_to_range(&snapshot.source_text, method_decl.span);

                if name == "main" {
                    lenses.push(CodeLens {
                        range,
                        command: Some(Command {
                            title: "▶ Run Main".to_string(),
                            command: "solix.runMain".to_string(),
                            arguments: Some(vec![serde_json::json!(snapshot.uri.to_string())]),
                        }),
                        data: None,
                    });
                    lenses.push(CodeLens {
                        range,
                        command: Some(Command {
                            title: "🐞 Debug Main".to_string(),
                            command: "solix.debugMain".to_string(),
                            arguments: Some(vec![serde_json::json!(snapshot.uri.to_string())]),
                        }),
                        data: None,
                    });
                } else if name.starts_with("test") {
                    lenses.push(CodeLens {
                        range,
                        command: Some(Command {
                            title: "▶ Run Test".to_string(),
                            command: "solix.runTest".to_string(),
                            arguments: Some(vec![
                                serde_json::json!(snapshot.uri.to_string()),
                                serde_json::json!(name),
                            ]),
                        }),
                        data: None,
                    });
                    lenses.push(CodeLens {
                        range,
                        command: Some(Command {
                            title: "🐞 Debug Test".to_string(),
                            command: "solix.debugTest".to_string(),
                            arguments: Some(vec![
                                serde_json::json!(snapshot.uri.to_string()),
                                serde_json::json!(name),
                            ]),
                        }),
                        data: None,
                    });
                }

                // Reference count lens
                let ref_count = count_references(workspace, name);
                lenses.push(CodeLens {
                    range,
                    command: Some(Command {
                        title: format!("{ref_count} references"),
                        command: "solix.showReferences".to_string(),
                        arguments: Some(vec![serde_json::json!(snapshot.uri.to_string())]),
                    }),
                    data: None,
                });
            }
            Declaration::Class(class_decl) => {
                let name = get_span_text(&snapshot.source_text, class_decl.name_span);
                let range = span_to_range(&snapshot.source_text, class_decl.span);

                // Reference count lens on class
                let ref_count = count_references(workspace, name);
                lenses.push(CodeLens {
                    range,
                    command: Some(Command {
                        title: format!("{ref_count} references"),
                        command: "solix.showReferences".to_string(),
                        arguments: Some(vec![serde_json::json!(snapshot.uri.to_string())]),
                    }),
                    data: None,
                });

                for member in &class_decl.members {
                    if let ClassMember::Method(method_decl) = member {
                        let method_name = get_span_text(&snapshot.source_text, method_decl.name_span);
                        let method_range = span_to_range(&snapshot.source_text, method_decl.span);

                        if method_name == "main" && method_decl.modifiers.contains(&Modifier::Static) {
                            lenses.push(CodeLens {
                                range: method_range,
                                command: Some(Command {
                                    title: "▶ Run Main".to_string(),
                                    command: "solix.runMain".to_string(),
                                    arguments: Some(vec![serde_json::json!(snapshot.uri.to_string())]),
                                }),
                                data: None,
                            });
                            lenses.push(CodeLens {
                                range: method_range,
                                command: Some(Command {
                                    title: "🐞 Debug Main".to_string(),
                                    command: "solix.debugMain".to_string(),
                                    arguments: Some(vec![serde_json::json!(snapshot.uri.to_string())]),
                                }),
                                data: None,
                            });
                        }
                    }
                }
            }
            _ => {}
        }
    }

    lenses
}

fn count_references(workspace: &WorkspaceManager, symbol_name: &str) -> usize {
    let mut count = 0;
    for snapshot in workspace.all_snapshots() {
        if let Some(semantic) = &snapshot.last_good_semantic {
            for symbol_id in semantic.resolved_symbols.values() {
                if let Some(symbol) = semantic.symbol_table.get_symbol(*symbol_id)
                    && symbol.name == symbol_name {
                        count += 1;
                    }
            }
        }
    }
    count
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
