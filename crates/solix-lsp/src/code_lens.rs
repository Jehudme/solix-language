//! CodeLens actions for running main/test functions and showing reference counts.

#![forbid(unsafe_code)]

use crate::diagnostics::span_to_range;
use crate::workspace::{AnalysisSnapshot, WorkspaceManager};
use solix_common::Span;
use solix_compilation::phases::parser::statements::declarations::{
    ClassMember, Declaration, Modifier,
};
use tower_lsp::lsp_types::{CodeLens, Command};

/// Resolves `textDocument/codeLens` for entry points and references.
#[must_use]
pub fn code_lenses(workspace: &WorkspaceManager, snapshot: &AnalysisSnapshot) -> Vec<CodeLens> {
    let ast = match snapshot.ast.as_ref() {
        Some(program_ast) => program_ast,
        None => return Vec::new(),
    };

    let (configured_main, configured_test) =
        if let Some(manifest) = workspace.project_manifest_for_uri(&snapshot.uri) {
            let main_symbol = manifest
                .main_entrypoint()
                .unwrap_or_else(|| "main".to_string());
            let test_symbol = manifest
                .test_entrypoint()
                .unwrap_or_else(|| "test".to_string());
            (main_symbol, test_symbol)
        } else {
            ("main".to_string(), "test".to_string())
        };

    let file_package = if let Some(package_statement) = &ast.package {
        let mut segments = Vec::new();
        for span in &package_statement.path {
            let start_index = span.start as usize;
            let end_index = span.end as usize;
            if end_index <= snapshot.source_text.len() {
                segments.push(&snapshot.source_text[start_index..end_index]);
            }
        }
        segments.join("::")
    } else {
        String::new()
    };

    let mut lenses = Vec::new();

    for declaration in &ast.declarations {
        match declaration {
            Declaration::Method(method_decl) => {
                let name = get_span_text(&snapshot.source_text, method_decl.name_span);
                let range = span_to_range(&snapshot.source_text, method_decl.span);
                let qualifier = if file_package.is_empty() {
                    None
                } else {
                    Some(file_package.as_str())
                };

                let is_main = matches_entrypoint(name, qualifier, &configured_main);
                let is_test = (name.starts_with("test") && method_decl.parameters.is_empty())
                    || matches_entrypoint(name, qualifier, &configured_test);

                if is_main {
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
                            title: "Debug Main".to_string(),
                            command: "solix.debugMain".to_string(),
                            arguments: Some(vec![serde_json::json!(snapshot.uri.to_string())]),
                        }),
                        data: None,
                    });
                } else if is_test {
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
                            title: "Debug Test".to_string(),
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
                let ref_count = count_references(workspace, snapshot, method_decl.span, name);
                lenses.push(CodeLens {
                    range,
                    command: Some(Command {
                        title: format!("{ref_count} references"),
                        command: "solix.showReferences".to_string(),
                        arguments: Some(vec![
                            serde_json::json!(snapshot.uri.to_string()),
                            serde_json::json!({
                                "line": range.start.line,
                                "character": range.start.character,
                            }),
                        ]),
                    }),
                    data: None,
                });
            }
            Declaration::Class(class_decl) => {
                let name = get_span_text(&snapshot.source_text, class_decl.name_span);
                let range = span_to_range(&snapshot.source_text, class_decl.span);

                // Reference count lens on class
                let ref_count = count_references(workspace, snapshot, class_decl.span, name);
                lenses.push(CodeLens {
                    range,
                    command: Some(Command {
                        title: format!("{ref_count} references"),
                        command: "solix.showReferences".to_string(),
                        arguments: Some(vec![
                            serde_json::json!(snapshot.uri.to_string()),
                            serde_json::json!({
                                "line": range.start.line,
                                "character": range.start.character,
                            }),
                        ]),
                    }),
                    data: None,
                });

                let class_qualifier = if file_package.is_empty() {
                    name.to_string()
                } else {
                    format!("{file_package}::{name}")
                };

                for member in &class_decl.members {
                    if let ClassMember::Method(method_decl) = member {
                        let method_name =
                            get_span_text(&snapshot.source_text, method_decl.name_span);
                        let method_range = span_to_range(&snapshot.source_text, method_decl.span);

                        let is_main = matches_entrypoint(
                            method_name,
                            Some(&class_qualifier),
                            &configured_main,
                        ) && method_decl.modifiers.contains(&Modifier::Static);
                        let is_test = ((method_name.starts_with("test")
                            && method_decl.parameters.is_empty())
                            || matches_entrypoint(
                                method_name,
                                Some(&class_qualifier),
                                &configured_test,
                            ))
                            && method_decl.modifiers.contains(&Modifier::Static);

                        if is_main {
                            lenses.push(CodeLens {
                                range: method_range,
                                command: Some(Command {
                                    title: "▶ Run Main".to_string(),
                                    command: "solix.runMain".to_string(),
                                    arguments: Some(vec![serde_json::json!(
                                        snapshot.uri.to_string()
                                    )]),
                                }),
                                data: None,
                            });
                            lenses.push(CodeLens {
                                range: method_range,
                                command: Some(Command {
                                    title: "Debug Main".to_string(),
                                    command: "solix.debugMain".to_string(),
                                    arguments: Some(vec![serde_json::json!(
                                        snapshot.uri.to_string()
                                    )]),
                                }),
                                data: None,
                            });
                        } else if is_test {
                            lenses.push(CodeLens {
                                range: method_range,
                                command: Some(Command {
                                    title: "▶ Run Test".to_string(),
                                    command: "solix.runTest".to_string(),
                                    arguments: Some(vec![
                                        serde_json::json!(snapshot.uri.to_string()),
                                        serde_json::json!(method_name),
                                    ]),
                                }),
                                data: None,
                            });
                            lenses.push(CodeLens {
                                range: method_range,
                                command: Some(Command {
                                    title: "Debug Test".to_string(),
                                    command: "solix.debugTest".to_string(),
                                    arguments: Some(vec![
                                        serde_json::json!(snapshot.uri.to_string()),
                                        serde_json::json!(method_name),
                                    ]),
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

fn matches_entrypoint(candidate_name: &str, qualifier: Option<&str>, target_symbol: &str) -> bool {
    if candidate_name == target_symbol || target_symbol.ends_with(&format!("::{candidate_name}")) {
        return true;
    }
    if let Some(qualifier_str) = qualifier {
        let qualified_candidate = format!("{qualifier_str}::{candidate_name}");
        if qualified_candidate == target_symbol
            || target_symbol.ends_with(&format!("::{qualified_candidate}"))
            || qualified_candidate.ends_with(&format!("::{target_symbol}"))
        {
            return true;
        }
    }
    false
}

fn count_references(
    workspace: &WorkspaceManager,
    target_snapshot: &AnalysisSnapshot,
    declaration_span: Span,
    symbol_name: &str,
) -> usize {
    let target_fqn: Option<String> =
        target_snapshot
            .last_good_semantic
            .as_ref()
            .and_then(|semantic| {
                semantic
                    .symbol_table
                    .symbols
                    .iter()
                    .find(|sym| {
                        sym.source_id == target_snapshot.source_id
                            && sym.name == symbol_name
                            && (sym.span == declaration_span
                                || (sym.span.start >= declaration_span.start
                                    && sym.span.end <= declaration_span.end))
                    })
                    .map(|sym| sym.fully_qualified_name.clone())
            });

    let mut count = 0;
    for snapshot in workspace.all_snapshots() {
        if let Some(semantic) = &snapshot.last_good_semantic {
            for symbol_id in semantic.resolved_symbols.values() {
                if let Some(symbol) = semantic.symbol_table.get_symbol(*symbol_id) {
                    let matches = match &target_fqn {
                        Some(fqn) => symbol.fully_qualified_name == *fqn,
                        None => symbol.name == symbol_name,
                    };
                    if matches {
                        count += 1;
                    }
                }
            }
        }
    }
    count
}

fn get_span_text(source: &str, span: Span) -> &str {
    let start_offset = span.start as usize;
    let end_offset = span.end as usize;
    if start_offset <= end_offset
        && end_offset <= source.len()
        && source.is_char_boundary(start_offset)
        && source.is_char_boundary(end_offset)
    {
        &source[start_offset..end_offset]
    } else {
        ""
    }
}
