//! Source code navigation: Go to Definition (with chaining), Type Definition, Implementation, References, and Document Highlights.

#![forbid(unsafe_code)]

use crate::diagnostics::{position_to_byte_offset, span_to_range};
use crate::workspace::{AnalysisSnapshot, WorkspaceManager};
use solix_common::Span;
use solix_common::tokens::TokenKind;
use solix_common::types::Type;
use solix_compilation::phases::parser::statements::declarations::{ClassMember, Declaration};
use solix_compilation::phases::parser::statements::expressions::Expression;
use solix_compilation::phases::parser::statements::{ProgramAst, Statement};
use tower_lsp::lsp_types::{
    DocumentHighlight, DocumentHighlightKind, DocumentLink, GotoDefinitionResponse, Location,
    Position, Url,
};

/// Resolves `textDocument/definition` for a given position in a document.
///
/// Implements Definition Chaining: if the cursor is already on a declaration,
/// it navigates to that symbol's type definition.
#[must_use]
pub fn find_definition(
    workspace: &WorkspaceManager,
    snapshot: &AnalysisSnapshot,
    position: Position,
) -> Option<GotoDefinitionResponse> {
    let offset = position_to_byte_offset(&snapshot.source_text, position);
    let target_offset = offset as u32;

    // Check if cursor is on an import statement
    if let Some(ast) = &snapshot.ast {
        for import_stmt in &ast.imports {
            if import_stmt.span.start <= target_offset && target_offset <= import_stmt.span.end {
                let last_segment = import_stmt.path.last()?;
                let symbol_name = get_span_text(&snapshot.source_text, *last_segment);
                if let Some(target_location) =
                    find_symbol_declaration_by_name(workspace, symbol_name)
                {
                    return Some(GotoDefinitionResponse::Scalar(target_location));
                }
            }
        }
    }

    let symbol = snapshot.symbol_at_offset(offset)?;
    let semantic = snapshot.last_good_semantic.as_ref()?;

    // Check if cursor is already on the symbol's declaration
    let is_on_declaration = symbol.span.start <= target_offset && target_offset <= symbol.span.end;

    if is_on_declaration {
        // Chain to type definition!
        if let Some(type_location) = resolve_type_location(workspace, semantic, &symbol.symbol_type)
        {
            return Some(GotoDefinitionResponse::Scalar(type_location));
        }
    }

    // Navigate to the symbol declaration
    let target_url = resolve_source_url(workspace, snapshot, symbol.source_id);
    let decl_range = span_to_range(&snapshot.source_text, symbol.span);

    Some(GotoDefinitionResponse::Scalar(Location {
        uri: target_url,
        range: decl_range,
    }))
}

/// Resolves `textDocument/typeDefinition` for a given position.
#[must_use]
pub fn find_type_definition(
    workspace: &WorkspaceManager,
    snapshot: &AnalysisSnapshot,
    position: Position,
) -> Option<GotoDefinitionResponse> {
    let offset = position_to_byte_offset(&snapshot.source_text, position);
    let evaluated_type = snapshot.type_at_offset(offset)?;
    let semantic = snapshot.last_good_semantic.as_ref()?;

    let type_location = resolve_type_location(workspace, semantic, &evaluated_type)?;
    Some(GotoDefinitionResponse::Scalar(type_location))
}

/// Resolves `textDocument/implementation` for a given interface or virtual method.
#[must_use]
pub fn find_implementations(
    workspace: &WorkspaceManager,
    snapshot: &AnalysisSnapshot,
    position: Position,
) -> Option<Vec<Location>> {
    let offset = position_to_byte_offset(&snapshot.source_text, position);
    let symbol = snapshot.symbol_at_offset(offset)?;
    let semantic = snapshot.last_good_semantic.as_ref()?;

    let mut locations = Vec::new();

    // Case 1: Cursor is on an Interface declaration
    for class_layout in semantic.layouts.values() {
        if (class_layout.itable_methods.contains_key(&symbol.name)
            || class_layout
                .itable_methods
                .contains_key(&symbol.fully_qualified_name))
            && let Some(class_symbol) = semantic
                .symbol_table
                .get_symbol(class_layout.class_symbol_id)
        {
            let target_url = resolve_source_url(workspace, snapshot, class_symbol.source_id);
            let target_range = span_to_range(&snapshot.source_text, class_symbol.span);
            locations.push(Location {
                uri: target_url,
                range: target_range,
            });
        }
    }

    // Case 2: Cursor is on an Interface method or Virtual method
    for class_layout in semantic.layouts.values() {
        if let Some(method_ids) = class_layout.itable_methods.get(&symbol.name) {
            for &method_id in method_ids {
                if let Some(impl_method) = semantic.symbol_table.get_symbol(method_id) {
                    let target_url = resolve_source_url(workspace, snapshot, impl_method.source_id);
                    let target_range = span_to_range(&snapshot.source_text, impl_method.span);
                    locations.push(Location {
                        uri: target_url,
                        range: target_range,
                    });
                }
            }
        }

        // Check virtual method overrides
        for &vtable_method_id in &class_layout.vtable_methods {
            if let Some(impl_method) = semantic.symbol_table.get_symbol(vtable_method_id)
                && impl_method.name == symbol.name
                && impl_method.id != symbol.id
            {
                let target_url = resolve_source_url(workspace, snapshot, impl_method.source_id);
                let target_range = span_to_range(&snapshot.source_text, impl_method.span);
                locations.push(Location {
                    uri: target_url,
                    range: target_range,
                });
            }
        }
    }

    if locations.is_empty() {
        None
    } else {
        Some(locations)
    }
}

/// Resolves `textDocument/references` across all workspace snapshots.
#[must_use]
pub fn find_references(
    workspace: &WorkspaceManager,
    snapshot: &AnalysisSnapshot,
    position: Position,
    include_declaration: bool,
) -> Option<Vec<Location>> {
    let offset = position_to_byte_offset(&snapshot.source_text, position);
    let target_symbol = snapshot.symbol_at_offset(offset)?;
    let mut locations = Vec::new();

    // Include declaration if requested
    if include_declaration {
        let decl_url = resolve_source_url(workspace, snapshot, target_symbol.source_id);
        let decl_range = span_to_range(&snapshot.source_text, target_symbol.span);
        locations.push(Location {
            uri: decl_url,
            range: decl_range,
        });
    }

    // Search across all cached workspace snapshots
    for other_snapshot in workspace.all_snapshots() {
        if let Some(semantic) = &other_snapshot.last_good_semantic {
            for (usage_span, symbol_id) in &semantic.resolved_symbols {
                if *symbol_id == target_symbol.id {
                    let usage_range = span_to_range(&other_snapshot.source_text, *usage_span);
                    locations.push(Location {
                        uri: other_snapshot.uri.clone(),
                        range: usage_range,
                    });
                }
            }
        }
    }

    if locations.is_empty() {
        None
    } else {
        Some(locations)
    }
}

/// Resolves `textDocument/documentHighlight` for symbol under cursor in the current document.
#[must_use]
pub fn document_highlights(
    snapshot: &AnalysisSnapshot,
    position: Position,
) -> Option<Vec<DocumentHighlight>> {
    let offset = position_to_byte_offset(&snapshot.source_text, position);
    let target_symbol = snapshot.symbol_at_offset(offset)?;
    let semantic = snapshot.last_good_semantic.as_ref()?;
    let ast = snapshot.ast.as_ref()?;

    let mut write_spans = std::collections::HashSet::new();
    collect_write_spans_in_ast(ast, &mut write_spans);

    let mut highlights = Vec::new();

    // Add declaration highlight
    if target_symbol.source_id == snapshot.source_id {
        let decl_range = span_to_range(&snapshot.source_text, target_symbol.span);
        highlights.push(DocumentHighlight {
            range: decl_range,
            kind: Some(DocumentHighlightKind::READ),
        });
    }

    // Add usage highlights
    for (usage_span, symbol_id) in &semantic.resolved_symbols {
        if *symbol_id == target_symbol.id {
            let kind = if write_spans.contains(usage_span) {
                DocumentHighlightKind::WRITE
            } else {
                DocumentHighlightKind::READ
            };
            let range = span_to_range(&snapshot.source_text, *usage_span);
            highlights.push(DocumentHighlight {
                range,
                kind: Some(kind),
            });
        }
    }

    if highlights.is_empty() {
        None
    } else {
        Some(highlights)
    }
}

/// Resolves `textDocument/documentLink` for imports and `"res://"` strings.
#[must_use]
pub fn document_links(
    workspace: &WorkspaceManager,
    snapshot: &AnalysisSnapshot,
) -> Vec<DocumentLink> {
    let mut links = Vec::new();

    if let Some(ast) = &snapshot.ast {
        // Import statements
        for import_stmt in &ast.imports {
            if let Some(last_segment) = import_stmt.path.last() {
                let symbol_name = get_span_text(&snapshot.source_text, *last_segment);
                if let Some(target_location) =
                    find_symbol_declaration_by_name(workspace, symbol_name)
                {
                    let range = span_to_range(&snapshot.source_text, import_stmt.span);
                    links.push(DocumentLink {
                        range,
                        target: Some(target_location.uri),
                        tooltip: Some(format!("Open definition of {symbol_name}")),
                        data: None,
                    });
                }
            }
        }
    }

    // Scan for "res://" resource references in string literals
    for token in &snapshot.tokens {
        if matches!(token.kind, TokenKind::String) {
            let text = get_span_text(&snapshot.source_text, token.span);
            let unquoted = text.trim_matches('"');
            if let Some(resource_path) = unquoted.strip_prefix("res://")
                && let Some(root) = workspace.workspace_root()
            {
                let full_resource_path = root.join("assets").join(resource_path);
                if let Ok(target_url) = Url::from_file_path(full_resource_path) {
                    let range = span_to_range(&snapshot.source_text, token.span);
                    links.push(DocumentLink {
                        range,
                        target: Some(target_url),
                        tooltip: Some(format!("Open asset {unquoted}")),
                        data: None,
                    });
                }
            }
        }
    }

    links
}

fn resolve_source_url(
    workspace: &WorkspaceManager,
    snapshot: &AnalysisSnapshot,
    source_id: u16,
) -> Url {
    if source_id == snapshot.source_id {
        return snapshot.uri.clone();
    }
    for other_snapshot in workspace.all_snapshots() {
        if other_snapshot.source_id == source_id {
            return other_snapshot.uri;
        }
    }
    snapshot.uri.clone()
}

fn resolve_type_location(
    workspace: &WorkspaceManager,
    semantic: &crate::workspace::BoundSemanticSnapshot,
    evaluated_type: &Type,
) -> Option<Location> {
    let type_name = match evaluated_type {
        Type::Class { symbol_name, .. } => symbol_name.as_str(),
        Type::Array { element_type, .. } => match &**element_type {
            Type::Class { symbol_name, .. } => symbol_name.as_str(),
            _ => return None,
        },
        _ => return None,
    };

    // Look up in symbol table
    for symbol_entry in &semantic.symbol_table.symbols {
        if (symbol_entry.name == type_name || symbol_entry.fully_qualified_name == type_name)
            && matches!(symbol_entry.symbol_type, Type::Class { .. })
            && let Some(snapshot) = workspace
                .all_snapshots()
                .into_iter()
                .find(|snap| snap.source_id == symbol_entry.source_id)
        {
            let range = span_to_range(&snapshot.source_text, symbol_entry.span);
            return Some(Location {
                uri: snapshot.uri,
                range,
            });
        }
    }

    find_symbol_declaration_by_name(workspace, type_name)
}

fn find_symbol_declaration_by_name(
    workspace: &WorkspaceManager,
    symbol_name: &str,
) -> Option<Location> {
    for snapshot in workspace.all_snapshots() {
        if let Some(semantic) = &snapshot.last_good_semantic {
            for symbol_entry in &semantic.symbol_table.symbols {
                if symbol_entry.name == symbol_name
                    || symbol_entry.fully_qualified_name == symbol_name
                {
                    let range = span_to_range(&snapshot.source_text, symbol_entry.span);
                    return Some(Location {
                        uri: snapshot.uri.clone(),
                        range,
                    });
                }
            }
        }
    }
    None
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

fn collect_write_spans_in_ast(ast: &ProgramAst, write_spans: &mut std::collections::HashSet<Span>) {
    for decl in &ast.declarations {
        match decl {
            Declaration::Method(method_decl) => {
                if let Some(body) = &method_decl.body {
                    collect_write_spans_in_statements(&body.statements, write_spans);
                }
            }
            Declaration::Class(class_decl) => {
                for member in &class_decl.members {
                    if let ClassMember::Method(method_decl) = member
                        && let Some(body) = &method_decl.body
                    {
                        collect_write_spans_in_statements(&body.statements, write_spans);
                    }
                }
            }
            _ => {}
        }
    }
}

fn collect_write_spans_in_statements(
    statements: &[Statement],
    write_spans: &mut std::collections::HashSet<Span>,
) {
    for statement in statements {
        match statement {
            Statement::Expression(expr_stmt) => {
                collect_write_spans_in_expression(&expr_stmt.expression, write_spans);
            }
            Statement::Block(block) => {
                collect_write_spans_in_statements(&block.statements, write_spans);
            }
            Statement::If(if_stmt) => {
                collect_write_spans_in_statement(&if_stmt.then_branch, write_spans);
                if let Some(else_branch) = &if_stmt.else_branch {
                    collect_write_spans_in_statement(else_branch, write_spans);
                }
            }
            Statement::While(while_stmt) => {
                collect_write_spans_in_statement(&while_stmt.body, write_spans);
            }
            Statement::For(for_stmt) => {
                collect_write_spans_in_statement(&for_stmt.body, write_spans);
            }
            _ => {}
        }
    }
}

fn collect_write_spans_in_statement(
    statement: &Statement,
    write_spans: &mut std::collections::HashSet<Span>,
) {
    if let Statement::Block(block) = statement {
        collect_write_spans_in_statements(&block.statements, write_spans);
    } else if let Statement::Expression(expr_stmt) = statement {
        collect_write_spans_in_expression(&expr_stmt.expression, write_spans);
    }
}

fn collect_write_spans_in_expression(
    expression: &Expression,
    write_spans: &mut std::collections::HashSet<Span>,
) {
    match expression {
        Expression::Assignment { target, value, .. } => {
            write_spans.insert(target.span());
            collect_write_spans_in_expression(value, write_spans);
        }
        Expression::Unary {
            operand, operator, ..
        } => {
            if matches!(
                operator,
                solix_common::tokens::TokenKind::OperatorIncrement
                    | solix_common::tokens::TokenKind::OperatorDecrement
            ) {
                write_spans.insert(operand.span());
            }
        }
        Expression::Binary { left, right, .. } => {
            collect_write_spans_in_expression(left, write_spans);
            collect_write_spans_in_expression(right, write_spans);
        }
        _ => {}
    }
}
