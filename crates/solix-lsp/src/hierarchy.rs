//! Call hierarchy and type hierarchy explorers for Solix.

#![forbid(unsafe_code)]

use crate::diagnostics::{position_to_byte_offset, span_to_range};
use crate::workspace::{AnalysisSnapshot, WorkspaceManager};
use solix_common::Span;
use solix_common::types::Type;
use solix_compilation::phases::parser::statements::declarations::{ClassMember, Declaration};
use solix_compilation::phases::parser::statements::expressions::Expression;
use solix_compilation::phases::parser::statements::{ProgramAst, Statement};
use tower_lsp::lsp_types::{
    CallHierarchyIncomingCall, CallHierarchyItem, CallHierarchyOutgoingCall, Position, Range,
    SymbolKind, TypeHierarchyItem,
};

/// Prepares a call hierarchy entry for the function or method at the given position.
#[must_use]
pub fn prepare_call_hierarchy(
    snapshot: &AnalysisSnapshot,
    position: Position,
) -> Option<Vec<CallHierarchyItem>> {
    let offset = position_to_byte_offset(&snapshot.source_text, position);
    let symbol = snapshot.symbol_at_offset(offset)?;

    let item = make_call_hierarchy_item(snapshot, &symbol.name, symbol.span, symbol.span);
    Some(vec![item])
}

/// Resolves callers (`incomingCalls`) for a call hierarchy item.
#[must_use]
pub fn incoming_calls(
    workspace: &WorkspaceManager,
    item: &CallHierarchyItem,
) -> Option<Vec<CallHierarchyIncomingCall>> {
    let target_name = &item.name;
    let mut incoming = Vec::new();

    for snapshot in workspace.all_snapshots() {
        if let Some(ast) = &snapshot.ast {
            collect_callers_in_ast(&snapshot, ast, target_name, &mut incoming);
        }
    }

    if incoming.is_empty() {
        None
    } else {
        Some(incoming)
    }
}

/// Resolves callees (`outgoingCalls`) for a call hierarchy item.
#[must_use]
pub fn outgoing_calls(
    workspace: &WorkspaceManager,
    item: &CallHierarchyItem,
) -> Option<Vec<CallHierarchyOutgoingCall>> {
    let snapshot = workspace.get_snapshot(&item.uri)?;
    let ast = snapshot.ast.as_ref()?;
    let offset = position_to_byte_offset(&snapshot.source_text, item.range.start);
    let target_offset = offset as u32;

    let mut outgoing = Vec::new();

    for decl in &ast.declarations {
        match decl {
            Declaration::Method(method_decl) => {
                if method_decl.span.start <= target_offset && target_offset <= method_decl.span.end
                    && let Some(body) = &method_decl.body {
                        collect_callees_in_statements(&snapshot, &body.statements, &mut outgoing);
                    }
            }
            Declaration::Class(class_decl) => {
                for member in &class_decl.members {
                    if let ClassMember::Method(method_decl) = member
                        && method_decl.span.start <= target_offset && target_offset <= method_decl.span.end
                            && let Some(body) = &method_decl.body {
                                collect_callees_in_statements(&snapshot, &body.statements, &mut outgoing);
                            }
                }
            }
            _ => {}
        }
    }

    if outgoing.is_empty() {
        None
    } else {
        Some(outgoing)
    }
}

/// Prepares a type hierarchy entry for the class or interface at the given position.
#[must_use]
pub fn prepare_type_hierarchy(
    snapshot: &AnalysisSnapshot,
    position: Position,
) -> Option<Vec<TypeHierarchyItem>> {
    let offset = position_to_byte_offset(&snapshot.source_text, position);
    let symbol = snapshot.symbol_at_offset(offset)?;

    if matches!(symbol.symbol_type, Type::Class { .. }) {
        let kind = if symbol.modifiers.contains(&solix_compilation::phases::parser::statements::Modifier::Abstract) {
            SymbolKind::INTERFACE
        } else {
            SymbolKind::CLASS
        };

        let range = span_to_range(&snapshot.source_text, symbol.span);
        Some(vec![TypeHierarchyItem {
            name: symbol.name,
            kind,
            tags: None,
            detail: Some(symbol.fully_qualified_name),
            uri: snapshot.uri.clone(),
            range,
            selection_range: range,
            data: None,
        }])
    } else {
        None
    }
}

/// Resolves supertypes (base class + implemented interfaces) for a type hierarchy item.
#[must_use]
pub fn supertypes(
    workspace: &WorkspaceManager,
    item: &TypeHierarchyItem,
) -> Option<Vec<TypeHierarchyItem>> {
    let snapshot = workspace.get_snapshot(&item.uri)?;
    let ast = snapshot.ast.as_ref()?;

    let mut result = Vec::new();

    for decl in &ast.declarations {
        if let Declaration::Class(class_decl) = decl {
            let class_name = get_span_text(&snapshot.source_text, class_decl.name_span);
            if class_name == item.name {
                if let Some(super_class) = &class_decl.super_class {
                    let super_name = get_span_text(&snapshot.source_text, super_class.span());
                    if let Some(super_item) = find_type_item(workspace, super_name) {
                        result.push(super_item);
                    }
                }
                for interface_ann in &class_decl.interfaces {
                    let iface_name = get_span_text(&snapshot.source_text, interface_ann.span());
                    if let Some(iface_item) = find_type_item(workspace, iface_name) {
                        result.push(iface_item);
                    }
                }
            }
        }
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

/// Resolves subtypes (derived subclasses + implementing classes) for a type hierarchy item.
#[must_use]
pub fn subtypes(
    workspace: &WorkspaceManager,
    item: &TypeHierarchyItem,
) -> Option<Vec<TypeHierarchyItem>> {
    let mut result = Vec::new();

    for other_snapshot in workspace.all_snapshots() {
        if let Some(ast) = &other_snapshot.ast {
            for decl in &ast.declarations {
                if let Declaration::Class(class_decl) = decl {
                    let is_subclass = class_decl
                        .super_class
                        .as_ref()
                        .is_some_and(|super_type| {
                            get_span_text(&other_snapshot.source_text, super_type.span()) == item.name
                        });
                    let is_impl = class_decl.interfaces.iter().any(|iface| {
                        get_span_text(&other_snapshot.source_text, iface.span()) == item.name
                    });

                    if is_subclass || is_impl {
                        let name = get_span_text(&other_snapshot.source_text, class_decl.name_span).to_string();
                        let range = span_to_range(&other_snapshot.source_text, class_decl.span);
                        result.push(TypeHierarchyItem {
                            name,
                            kind: SymbolKind::CLASS,
                            tags: None,
                            detail: None,
                            uri: other_snapshot.uri.clone(),
                            range,
                            selection_range: range,
                            data: None,
                        });
                    }
                }
            }
        }
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

fn make_call_hierarchy_item(
    snapshot: &AnalysisSnapshot,
    name: &str,
    span: Span,
    name_span: Span,
) -> CallHierarchyItem {
    let range = span_to_range(&snapshot.source_text, span);
    let selection_range = span_to_range(&snapshot.source_text, name_span);
    CallHierarchyItem {
        name: name.to_string(),
        kind: SymbolKind::METHOD,
        tags: None,
        detail: None,
        uri: snapshot.uri.clone(),
        range,
        selection_range,
        data: None,
    }
}

fn collect_callers_in_ast(
    snapshot: &AnalysisSnapshot,
    ast: &ProgramAst,
    target_name: &str,
    incoming: &mut Vec<CallHierarchyIncomingCall>,
) {
    for decl in &ast.declarations {
        match decl {
            Declaration::Method(method_decl) => {
                if let Some(body) = &method_decl.body {
                    let mut call_ranges = Vec::new();
                    find_calls_in_statements(&snapshot.source_text, &body.statements, target_name, &mut call_ranges);
                    if !call_ranges.is_empty() {
                        let caller_name = get_span_text(&snapshot.source_text, method_decl.name_span);
                        let from_item = make_call_hierarchy_item(snapshot, caller_name, method_decl.span, method_decl.name_span);
                        incoming.push(CallHierarchyIncomingCall {
                            from: from_item,
                            from_ranges: call_ranges,
                        });
                    }
                }
            }
            Declaration::Class(class_decl) => {
                for member in &class_decl.members {
                    if let ClassMember::Method(method_decl) = member
                        && let Some(body) = &method_decl.body {
                            let mut call_ranges = Vec::new();
                            find_calls_in_statements(&snapshot.source_text, &body.statements, target_name, &mut call_ranges);
                            if !call_ranges.is_empty() {
                                let caller_name = get_span_text(&snapshot.source_text, method_decl.name_span);
                                let from_item = make_call_hierarchy_item(snapshot, caller_name, method_decl.span, method_decl.name_span);
                                incoming.push(CallHierarchyIncomingCall {
                                    from: from_item,
                                    from_ranges: call_ranges,
                                });
                            }
                        }
                }
            }
            _ => {}
        }
    }
}

fn find_calls_in_statements(
    source_text: &str,
    statements: &[Statement],
    target_name: &str,
    call_ranges: &mut Vec<Range>,
) {
    for stmt in statements {
        match stmt {
            Statement::Expression(expr_stmt) => {
                find_calls_in_expression(source_text, &expr_stmt.expression, target_name, call_ranges);
            }
            Statement::Block(block) => {
                find_calls_in_statements(source_text, &block.statements, target_name, call_ranges);
            }
            _ => {}
        }
    }
}

fn find_calls_in_expression(
    source_text: &str,
    expression: &Expression,
    target_name: &str,
    call_ranges: &mut Vec<Range>,
) {
    if let Expression::MethodCall { callee, span, .. } = expression {
        let callee_name = match &**callee {
            Expression::Identifier { span: id_span } => get_span_text(source_text, *id_span),
            Expression::MemberAccess { member_span, .. } => get_span_text(source_text, *member_span),
            _ => "",
        };

        if callee_name == target_name {
            call_ranges.push(span_to_range(source_text, *span));
        }
    }
}

fn collect_callees_in_statements(
    snapshot: &AnalysisSnapshot,
    statements: &[Statement],
    outgoing: &mut Vec<CallHierarchyOutgoingCall>,
) {
    for stmt in statements {
        match stmt {
            Statement::Expression(expr_stmt) => {
                collect_callees_in_expression(snapshot, &expr_stmt.expression, outgoing);
            }
            Statement::Block(block) => {
                collect_callees_in_statements(snapshot, &block.statements, outgoing);
            }
            _ => {}
        }
    }
}

fn collect_callees_in_expression(
    snapshot: &AnalysisSnapshot,
    expression: &Expression,
    outgoing: &mut Vec<CallHierarchyOutgoingCall>,
) {
    if let Expression::MethodCall { callee, span, .. } = expression {
        let callee_name = match &**callee {
            Expression::Identifier { span: id_span } => get_span_text(&snapshot.source_text, *id_span),
            Expression::MemberAccess { member_span, .. } => get_span_text(&snapshot.source_text, *member_span),
            _ => "",
        };

        if !callee_name.is_empty() {
            let to_item = make_call_hierarchy_item(snapshot, callee_name, *span, *span);
            let from_range = span_to_range(&snapshot.source_text, *span);
            outgoing.push(CallHierarchyOutgoingCall {
                to: to_item,
                from_ranges: vec![from_range],
            });
        }
    }
}

fn find_type_item(workspace: &WorkspaceManager, type_name: &str) -> Option<TypeHierarchyItem> {
    for snapshot in workspace.all_snapshots() {
        if let Some(ast) = &snapshot.ast {
            for decl in &ast.declarations {
                if let Declaration::Class(class_decl) = decl {
                    let name = get_span_text(&snapshot.source_text, class_decl.name_span);
                    if name == type_name {
                        let range = span_to_range(&snapshot.source_text, class_decl.span);
                        return Some(TypeHierarchyItem {
                            name: name.to_string(),
                            kind: SymbolKind::CLASS,
                            tags: None,
                            detail: None,
                            uri: snapshot.uri.clone(),
                            range,
                            selection_range: range,
                            data: None,
                        });
                    }
                }
            }
        }
    }
    None
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
