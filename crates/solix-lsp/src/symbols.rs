//! Document symbols outline hierarchy and workspace symbol search.

#![forbid(unsafe_code)]

use crate::diagnostics::span_to_range;
use crate::workspace::{AnalysisSnapshot, WorkspaceManager};
use solix_common::Span;
use solix_compilation::phases::parser::statements::declarations::{ClassMember, Declaration};
use tower_lsp::lsp_types::{
    DocumentSymbol, DocumentSymbolResponse, Location, SymbolKind, WorkspaceSymbol,
    WorkspaceSymbolResponse,
};

/// Resolves `textDocument/documentSymbol` into a nested hierarchical outline.
#[must_use]
#[allow(deprecated)]
pub fn document_symbols(snapshot: &AnalysisSnapshot) -> Option<DocumentSymbolResponse> {
    let ast = snapshot.ast.as_ref()?;
    let mut symbols = Vec::new();

    // Package symbol if declared
    if let Some(pkg) = &ast.package {
        let pkg_name = pkg
            .path
            .iter()
            .map(|span| get_span_text(&snapshot.source_text, *span))
            .collect::<Vec<_>>()
            .join(".");
        let range = span_to_range(&snapshot.source_text, pkg.span);
        symbols.push(DocumentSymbol {
            name: pkg_name,
            detail: Some("package".to_string()),
            kind: SymbolKind::PACKAGE,
            tags: None,
            range,
            selection_range: range,
            children: None,
            deprecated: None,
        });
    }

    // Top-level declarations
    for decl in &ast.declarations {
        match decl {
            Declaration::Class(class_decl) => {
                let name = get_span_text(&snapshot.source_text, class_decl.name_span).to_string();
                let range = span_to_range(&snapshot.source_text, class_decl.span);
                let selection_range = span_to_range(&snapshot.source_text, class_decl.name_span);
                let mut children = Vec::new();

                for member in &class_decl.members {
                    match member {
                        ClassMember::Field(field_decl) => {
                            let field_name =
                                get_span_text(&snapshot.source_text, field_decl.name_span)
                                    .to_string();
                            let field_range = span_to_range(&snapshot.source_text, field_decl.span);
                            let field_sel =
                                span_to_range(&snapshot.source_text, field_decl.name_span);
                            children.push(DocumentSymbol {
                                name: field_name,
                                detail: None,
                                kind: SymbolKind::FIELD,
                                tags: None,
                                range: field_range,
                                selection_range: field_sel,
                                children: None,
                                deprecated: None,
                            });
                        }
                        ClassMember::Method(method_decl) => {
                            let method_name =
                                get_span_text(&snapshot.source_text, method_decl.name_span)
                                    .to_string();
                            let method_range =
                                span_to_range(&snapshot.source_text, method_decl.span);
                            let method_sel =
                                span_to_range(&snapshot.source_text, method_decl.name_span);
                            children.push(DocumentSymbol {
                                name: method_name,
                                detail: None,
                                kind: SymbolKind::METHOD,
                                tags: None,
                                range: method_range,
                                selection_range: method_sel,
                                children: None,
                                deprecated: None,
                            });
                        }
                        ClassMember::Constructor(ctor_decl) => {
                            let ctor_name =
                                get_span_text(&snapshot.source_text, ctor_decl.name_span)
                                    .to_string();
                            let ctor_range = span_to_range(&snapshot.source_text, ctor_decl.span);
                            let ctor_sel =
                                span_to_range(&snapshot.source_text, ctor_decl.name_span);
                            children.push(DocumentSymbol {
                                name: ctor_name,
                                detail: Some("constructor".to_string()),
                                kind: SymbolKind::CONSTRUCTOR,
                                tags: None,
                                range: ctor_range,
                                selection_range: ctor_sel,
                                children: None,
                                deprecated: None,
                            });
                        }
                        ClassMember::Destructor(dtor_decl) => {
                            let dtor_name = format!(
                                "~{}",
                                get_span_text(&snapshot.source_text, dtor_decl.name_span)
                            );
                            let dtor_range = span_to_range(&snapshot.source_text, dtor_decl.span);
                            let dtor_sel =
                                span_to_range(&snapshot.source_text, dtor_decl.name_span);
                            children.push(DocumentSymbol {
                                name: dtor_name,
                                detail: Some("destructor".to_string()),
                                kind: SymbolKind::METHOD,
                                tags: None,
                                range: dtor_range,
                                selection_range: dtor_sel,
                                children: None,
                                deprecated: None,
                            });
                        }
                        _ => {}
                    }
                }

                symbols.push(DocumentSymbol {
                    name,
                    detail: Some("class".to_string()),
                    kind: SymbolKind::CLASS,
                    tags: None,
                    range,
                    selection_range,
                    children: if children.is_empty() {
                        None
                    } else {
                        Some(children)
                    },
                    deprecated: None,
                });
            }
            Declaration::Interface(iface_decl) => {
                let name = get_span_text(&snapshot.source_text, iface_decl.name_span).to_string();
                let range = span_to_range(&snapshot.source_text, iface_decl.span);
                let selection_range = span_to_range(&snapshot.source_text, iface_decl.name_span);
                let mut children = Vec::new();

                for method_decl in &iface_decl.methods {
                    let method_name =
                        get_span_text(&snapshot.source_text, method_decl.name_span).to_string();
                    let method_range = span_to_range(&snapshot.source_text, method_decl.span);
                    let method_sel = span_to_range(&snapshot.source_text, method_decl.name_span);
                    children.push(DocumentSymbol {
                        name: method_name,
                        detail: None,
                        kind: SymbolKind::METHOD,
                        tags: None,
                        range: method_range,
                        selection_range: method_sel,
                        children: None,
                        deprecated: None,
                    });
                }

                symbols.push(DocumentSymbol {
                    name,
                    detail: Some("interface".to_string()),
                    kind: SymbolKind::INTERFACE,
                    tags: None,
                    range,
                    selection_range,
                    children: if children.is_empty() {
                        None
                    } else {
                        Some(children)
                    },
                    deprecated: None,
                });
            }
            Declaration::Enum(enum_decl) => {
                let name = get_span_text(&snapshot.source_text, enum_decl.name_span).to_string();
                let range = span_to_range(&snapshot.source_text, enum_decl.span);
                let selection_range = span_to_range(&snapshot.source_text, enum_decl.name_span);
                let mut children = Vec::new();

                for variant in &enum_decl.variants {
                    let var_name =
                        get_span_text(&snapshot.source_text, variant.name_span).to_string();
                    let var_range = span_to_range(&snapshot.source_text, variant.span);
                    let var_sel = span_to_range(&snapshot.source_text, variant.name_span);
                    children.push(DocumentSymbol {
                        name: var_name,
                        detail: None,
                        kind: SymbolKind::ENUM_MEMBER,
                        tags: None,
                        range: var_range,
                        selection_range: var_sel,
                        children: None,
                        deprecated: None,
                    });
                }

                symbols.push(DocumentSymbol {
                    name,
                    detail: Some("enum".to_string()),
                    kind: SymbolKind::ENUM,
                    tags: None,
                    range,
                    selection_range,
                    children: if children.is_empty() {
                        None
                    } else {
                        Some(children)
                    },
                    deprecated: None,
                });
            }
            Declaration::Method(method_decl) => {
                let name = get_span_text(&snapshot.source_text, method_decl.name_span).to_string();
                let range = span_to_range(&snapshot.source_text, method_decl.span);
                let selection_range = span_to_range(&snapshot.source_text, method_decl.name_span);
                symbols.push(DocumentSymbol {
                    name,
                    detail: Some("function".to_string()),
                    kind: SymbolKind::FUNCTION,
                    tags: None,
                    range,
                    selection_range,
                    children: None,
                    deprecated: None,
                });
            }
            _ => {}
        }
    }

    Some(DocumentSymbolResponse::Nested(symbols))
}

/// Resolves `workspace/symbol` fuzzy search across all workspace symbols.
#[must_use]
pub fn workspace_symbols(
    workspace: &WorkspaceManager,
    query: &str,
) -> Option<WorkspaceSymbolResponse> {
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();

    for snapshot in workspace.all_snapshots() {
        if let Some(semantic) = &snapshot.last_good_semantic {
            for symbol in &semantic.symbol_table.symbols {
                if query_lower.is_empty()
                    || symbol.name.to_lowercase().contains(&query_lower)
                    || symbol
                        .fully_qualified_name
                        .to_lowercase()
                        .contains(&query_lower)
                {
                    let kind = match &symbol.symbol_type {
                        solix_common::types::Type::Class { .. } => SymbolKind::CLASS,
                        solix_common::types::Type::FunctionPointer { .. } => SymbolKind::METHOD,
                        _ => SymbolKind::FIELD,
                    };
                    let range = span_to_range(&snapshot.source_text, symbol.span);
                    results.push(WorkspaceSymbol {
                        name: symbol.name.clone(),
                        kind,
                        tags: None,
                        container_name: Some(symbol.fully_qualified_name.clone()),
                        location: tower_lsp::lsp_types::OneOf::Left(Location {
                            uri: snapshot.uri.clone(),
                            range,
                        }),
                        data: None,
                    });
                }
            }
        }
    }

    Some(WorkspaceSymbolResponse::Nested(results))
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
