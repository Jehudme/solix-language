//! Intelligent context-aware code completions for Solix.
//!
//! Enforces strict Solix v2.0 semantics:
//! - `.` proposes only instance fields, instance methods (with generic substitutions), and postfix completions.
//!   If invoked on a type name, provides static completions with an auto-rewrite edit from `.` to `::`.
//! - `::` proposes only static methods, static `const` fields, `enum` variants, and package sub-namespaces.

#![forbid(unsafe_code)]

use crate::diagnostics::position_to_byte_offset;
use crate::workspace::{AnalysisSnapshot, WorkspaceManager};
use solix_common::Span;
use solix_common::types::Type;
use solix_compilation::phases::binder::symbols::SymbolEntry;
use solix_compilation::phases::parser::statements::declarations::Declaration;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionList,
    CompletionParams, Documentation, InsertTextFormat, MarkupContent, MarkupKind, Position, Range,
    TextEdit,
};

/// Postfix completion templates supported in Solix.
const POSTFIX_TEMPLATES: &[(&str, &str, &str)] = &[
    ("var", "let name = $EXPR;", "Introduce variable"),
    (
        "null",
        "if ($EXPR == null) {\n    $0\n}",
        "Check if ($EXPR == null)",
    ),
    (
        "nn",
        "if ($EXPR != null) {\n    $0\n}",
        "Check if ($EXPR != null)",
    ),
    (
        "notnull",
        "if ($EXPR != null) {\n    $0\n}",
        "Check if ($EXPR != null)",
    ),
    ("if", "if ($EXPR) {\n    $0\n}", "Wrap in if ($EXPR)"),
    ("return", "return $EXPR;", "Return $EXPR"),
    ("ret", "return $EXPR;", "Return $EXPR"),
    (
        "println",
        "Console::println($EXPR);",
        "Wrap in Console::println($EXPR)",
    ),
];

/// Solix declaration keywords.
const DECLARATION_KEYWORDS: &[&str] = &[
    "class",
    "interface",
    "enum",
    "operator",
    "package",
    "import",
    "extern import",
    "alias",
];

/// Solix control flow keywords.
const CONTROL_FLOW_KEYWORDS: &[&str] = &[
    "if", "else", "while", "do", "for", "switch", "case", "default", "break", "continue", "return",
    "try", "catch", "finally", "throw", "assert", "exit",
];

/// Solix modifiers.
const MODIFIER_KEYWORDS: &[&str] = &[
    "public",
    "private",
    "protected",
    "internal",
    "static",
    "inline",
    "native",
    "const",
    "virtual",
    "override",
    "weak",
    "abstract",
    "extern",
];

/// Solix primitive types.
const PRIMITIVE_TYPES: &[&str] = &[
    "int8", "int16", "int32", "int64", "uint8", "uint16", "uint32", "uint64", "float32", "float64",
    "bool", "char", "string", "void",
];

/// Delimiter identifying the context of a member or scope access.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessDelimiter {
    /// Dot (`.`) instance member or type auto-rewrite delimiter.
    Dot,
    /// Double colon (`::`) static, enum variant, or namespace scope delimiter.
    DoubleColon,
}

/// Parses the line prefix to determine if the cursor is in a member or scope access context.
///
/// Returns `Some((receiver_expr, delimiter, partial_member_prefix))` if an access delimiter
/// is present before the cursor, preserving any trailing partial member name.
#[must_use]
pub fn parse_member_access_context(line_prefix: &str) -> Option<(String, AccessDelimiter, String)> {
    let trimmed_line = line_prefix.trim_start();
    if trimmed_line.starts_with("//")
        || trimmed_line.starts_with("import ")
        || trimmed_line.starts_with("extern import ")
        || trimmed_line.starts_with("package ")
    {
        return None;
    }

    let characters: Vec<(usize, char)> = line_prefix.char_indices().collect();
    if characters.is_empty() {
        return None;
    }

    let end_index = line_prefix.len();
    let mut partial_start = end_index;
    for (char_byte_offset, character) in characters.iter().rev() {
        if character.is_alphanumeric() || *character == '_' {
            partial_start = *char_byte_offset;
        } else {
            break;
        }
    }
    let partial_member_prefix = line_prefix[partial_start..end_index].to_string();

    let prefix_before_partial = &line_prefix[..partial_start];
    let (delimiter, delimiter_byte_len) = if prefix_before_partial.ends_with("::") {
        (AccessDelimiter::DoubleColon, 2)
    } else {
        let before_dot = prefix_before_partial.strip_suffix('.')?;
        if before_dot.ends_with('.') {
            return None;
        }
        (AccessDelimiter::Dot, 1)
    };

    let prefix_before_delimiter =
        &prefix_before_partial[..prefix_before_partial.len() - delimiter_byte_len];
    let trimmed_before_delimiter = prefix_before_delimiter.trim_end();
    if trimmed_before_delimiter.is_empty() {
        return None;
    }

    let receiver_chars: Vec<(usize, char)> = trimmed_before_delimiter.char_indices().collect();
    let mut paren_depth: usize = 0;
    let mut bracket_depth: usize = 0;
    let mut brace_depth: usize = 0;
    let mut receiver_start = 0;

    for (char_byte_offset, character) in receiver_chars.iter().rev() {
        match *character {
            ')' => paren_depth += 1,
            '(' => {
                if paren_depth > 0 {
                    paren_depth -= 1;
                } else {
                    receiver_start = char_byte_offset + character.len_utf8();
                    break;
                }
            }
            ']' => bracket_depth += 1,
            '[' => {
                if bracket_depth > 0 {
                    bracket_depth -= 1;
                } else {
                    receiver_start = char_byte_offset + character.len_utf8();
                    break;
                }
            }
            '}' => brace_depth += 1,
            '{' => {
                if brace_depth > 0 {
                    brace_depth -= 1;
                } else {
                    receiver_start = char_byte_offset + character.len_utf8();
                    break;
                }
            }
            _ => {
                if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 {
                    if character.is_alphanumeric()
                        || *character == '_'
                        || *character == '.'
                        || *character == ':'
                    {
                        receiver_start = *char_byte_offset;
                    } else {
                        receiver_start = char_byte_offset + character.len_utf8();
                        break;
                    }
                }
            }
        }
    }

    let receiver_expr = trimmed_before_delimiter[receiver_start..]
        .trim()
        .to_string();
    if receiver_expr.is_empty() {
        return None;
    }

    if receiver_expr
        .chars()
        .all(|character| character.is_ascii_digit())
    {
        return None;
    }

    Some((receiver_expr, delimiter, partial_member_prefix))
}

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
    let line_suffix = get_line_suffix(&snapshot.source_text, offset);
    let has_trailing_semicolon = line_suffix.trim_start().starts_with(';');

    // If typing a single colon (e.g. `solix::collections:`), suppress completion until the second colon `::`
    if line_prefix.ends_with(':') && !line_prefix.ends_with("::") {
        return CompletionList {
            is_incomplete: false,
            items: Vec::new(),
        };
    }

    let mut items = Vec::new();

    let trimmed_prefix = line_prefix.trim_start();
    let import_keyword = if let Some(stripped) = trimmed_prefix.strip_prefix("extern import ") {
        Some(("extern import ", stripped))
    } else {
        trimmed_prefix
            .strip_prefix("import ")
            .map(|stripped| ("import ", stripped))
    };

    if let Some((_, import_path_prefix)) = import_keyword {
        handle_import_completion(
            workspace,
            snapshot,
            import_path_prefix,
            has_trailing_semicolon,
            &mut items,
        );
    } else if let Some((receiver_expr, access_delimiter, partial_member_prefix)) =
        parse_member_access_context(line_prefix)
    {
        match access_delimiter {
            AccessDelimiter::Dot => {
                handle_dot_completion(
                    workspace,
                    snapshot,
                    &receiver_expr,
                    &partial_member_prefix,
                    position,
                    &mut items,
                );
            }
            AccessDelimiter::DoubleColon => {
                handle_scope_resolution_completion(
                    workspace,
                    snapshot,
                    &receiver_expr,
                    &partial_member_prefix,
                    &mut items,
                );
            }
        }
    } else {
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
    if item.documentation.is_none()
        && let Some(semantic) = &snapshot.last_good_semantic
    {
        for symbol in &semantic.symbol_table.symbols {
            if symbol.name == item.label
                && let Some(doc) = snapshot.extract_doc_comment_for_span(symbol.span)
            {
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

fn resolve_receiver_type(
    _workspace: &WorkspaceManager,
    snapshot: &AnalysisSnapshot,
    receiver_expr: &str,
    position: Position,
) -> Option<Type> {
    let semantic = snapshot.last_good_semantic.as_ref()?;
    let target_offset = position_to_byte_offset(&snapshot.source_text, position) as u32;

    let resolve_simple_ident = |ident: &str| -> Option<Type> {
        if ident == "this" {
            for symbol in &semantic.symbol_table.symbols {
                if symbol.source_id == snapshot.source_id
                    && matches!(symbol.symbol_type, Type::Class { .. })
                    && symbol.span.start <= target_offset
                    && target_offset <= symbol.span.end
                {
                    return Some(symbol.symbol_type.clone());
                }
            }
            return None;
        }

        for local_symbol in &semantic.symbol_table.local_symbols {
            if local_symbol.source_id == snapshot.source_id
                && local_symbol.name == ident
                && local_symbol.scope_span.start <= target_offset
                && target_offset <= local_symbol.scope_span.end
            {
                return Some(local_symbol.symbol_type.clone());
            }
        }

        for local_symbol in &semantic.symbol_table.local_symbols {
            if local_symbol.source_id == snapshot.source_id && local_symbol.name == ident {
                return Some(local_symbol.symbol_type.clone());
            }
        }

        for symbol in &semantic.symbol_table.symbols {
            if symbol.source_id == snapshot.source_id
                && matches!(symbol.symbol_type, Type::Class { .. })
                && symbol.span.start <= target_offset
                && target_offset <= symbol.span.end
            {
                for member_id in &symbol.members {
                    if let Some(member) = semantic.symbol_table.get_symbol(*member_id)
                        && member.name == ident
                        && !member.is_static
                    {
                        return Some(member.symbol_type.clone());
                    }
                }
            }
        }

        for symbol in &semantic.symbol_table.symbols {
            if symbol.name == ident && !symbol.is_static {
                return Some(symbol.symbol_type.clone());
            }
        }

        if let Some(ref program_ast) = snapshot.ast {
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
                            if declared_var_name == ident {
                                return Some(solix_compilation::phases::binder::type_resolver::resolve_type_annotation(
                                    snapshot.source_id,
                                    &snapshot.source_text,
                                    &var_decl.var_type,
                                    &semantic.symbol_table,
                                ));
                            }
                        }
                    }
                }
            }
        }

        None
    };

    fn parse_segment_name_call_index(segment_text: &str) -> (&str, bool, bool) {
        match (segment_text.find('('), segment_text.find('[')) {
            (Some(parenthesis_index), Some(bracket_index)) => {
                if parenthesis_index < bracket_index {
                    (&segment_text[..parenthesis_index], true, true)
                } else {
                    (&segment_text[..bracket_index], true, true)
                }
            }
            (Some(parenthesis_index), None) => (&segment_text[..parenthesis_index], true, false),
            (None, Some(bracket_index)) => (&segment_text[..bracket_index], false, true),
            (None, None) => (segment_text, false, false),
        }
    }

    let resolve_member_on_type = |parent_type: &Type, member_segment: &str| -> Option<Type> {
        let (member_name, is_call, is_index) = parse_segment_name_call_index(member_segment);

        let member_type = match parent_type {
            Type::Class { symbol_name, .. } => {
                let mut found_member_type = None;
                for symbol in &semantic.symbol_table.symbols {
                    if (symbol.name == *symbol_name || symbol.fully_qualified_name == *symbol_name)
                        && matches!(symbol.symbol_type, Type::Class { .. })
                    {
                        for member_id in &symbol.members {
                            if let Some(member) = semantic.symbol_table.get_symbol(*member_id)
                                && member.name == member_name
                                && !member.is_static
                            {
                                found_member_type = Some(member.symbol_type.clone());
                                break;
                            }
                        }
                        if found_member_type.is_some() {
                            break;
                        }
                    }
                }
                found_member_type?
            }
            Type::Array { element_type, .. } if is_index => {
                return Some((**element_type).clone());
            }
            _ => return None,
        };

        if is_call {
            if let Type::FunctionPointer { return_type, .. } = member_type {
                let return_boxed = *return_type;
                if is_index {
                    if let Type::Array { element_type, .. } = return_boxed {
                        Some(*element_type)
                    } else {
                        None
                    }
                } else {
                    Some(return_boxed)
                }
            } else {
                None
            }
        } else if is_index {
            if let Type::Array { element_type, .. } = member_type {
                Some(*element_type)
            } else {
                None
            }
        } else {
            Some(member_type)
        }
    };

    if receiver_expr.contains('.') {
        let segments: Vec<&str> = receiver_expr.split('.').collect();
        if segments.is_empty() {
            return None;
        }

        let first_segment = segments[0];
        let (first_name, first_is_call, first_is_index) =
            parse_segment_name_call_index(first_segment);

        let mut current_type = resolve_simple_ident(first_name)?;
        if first_is_call {
            if let Type::FunctionPointer { return_type, .. } = current_type {
                current_type = *return_type;
            } else {
                return None;
            }
        }
        if first_is_index {
            if let Type::Array { element_type, .. } = current_type {
                current_type = *element_type;
            } else {
                return None;
            }
        }

        for segment in &segments[1..] {
            current_type = resolve_member_on_type(&current_type, segment)?;
        }

        Some(current_type)
    } else {
        let (base_name, is_call, is_index) = parse_segment_name_call_index(receiver_expr);

        let mut base_type = resolve_simple_ident(base_name)?;
        if is_call {
            if let Type::FunctionPointer { return_type, .. } = base_type {
                base_type = *return_type;
            } else {
                return None;
            }
        }
        if is_index {
            if let Type::Array { element_type, .. } = base_type {
                base_type = *element_type;
            } else {
                return None;
            }
        }
        Some(base_type)
    }
}

fn handle_dot_completion(
    workspace: &WorkspaceManager,
    snapshot: &AnalysisSnapshot,
    receiver_expr: &str,
    partial_member_prefix: &str,
    position: Position,
    items: &mut Vec<CompletionItem>,
) {
    let semantic = snapshot.last_good_semantic.as_ref();

    // 1. Check if receiver is a known Class Name or Type Name
    let mut is_type_name = false;
    if let Some(semantic_snapshot) = semantic {
        for symbol in &semantic_snapshot.symbol_table.symbols {
            if (symbol.name == receiver_expr || symbol.fully_qualified_name == receiver_expr)
                && let Type::Class { symbol_name, .. } = &symbol.symbol_type
                && (symbol.name == *symbol_name || symbol.fully_qualified_name == *symbol_name)
            {
                is_type_name = true;
                let dot_char = position
                    .character
                    .saturating_sub(1 + partial_member_prefix.len() as u32);
                let replace_range = Range {
                    start: Position {
                        line: position.line,
                        character: dot_char,
                    },
                    end: Position {
                        line: position.line,
                        character: dot_char + 1,
                    },
                };

                for member_id in &symbol.members {
                    if let Some(member) = semantic_snapshot.symbol_table.get_symbol(*member_id)
                        && (member.is_static || member.is_const)
                        && member.name != "constructor"
                        && !member.name.starts_with('~')
                    {
                        let mut item = make_symbol_completion_item(snapshot, member);
                        item.additional_text_edits = Some(vec![TextEdit {
                            range: replace_range,
                            new_text: "::".to_string(),
                        }]);
                        item.detail = Some(format!("(Static) {}", member.to_signature_string()));
                        item.sort_text = Some(format!("0_{}", member.name));
                        items.push(item);
                    }
                }
                break;
            }
        }
    }

    if is_type_name {
        let lower_partial = partial_member_prefix.to_ascii_lowercase();
        if !lower_partial.is_empty() {
            items.retain(|item| item.label.to_ascii_lowercase().starts_with(&lower_partial));
        }
        return;
    }

    // 2. Receiver is an instance expression
    if let Some(semantic_snapshot) = semantic {
        let receiver_type = resolve_receiver_type(workspace, snapshot, receiver_expr, position);

        if let Some(Type::Class { symbol_name, .. }) = receiver_type {
            let mut direct_member_names = std::collections::HashSet::new();

            for symbol in &semantic_snapshot.symbol_table.symbols {
                if (symbol.name == symbol_name || symbol.fully_qualified_name == symbol_name)
                    && let Type::Class {
                        symbol_name: class_target,
                        ..
                    } = &symbol.symbol_type
                    && (symbol.name == *class_target
                        || symbol.fully_qualified_name == *class_target)
                {
                    // Direct class members
                    for member_id in &symbol.members {
                        if let Some(member) = semantic_snapshot.symbol_table.get_symbol(*member_id)
                            && !member.is_static
                            && member.name != "constructor"
                            && !member.name.starts_with('~')
                        {
                            let mut item = make_symbol_completion_item(snapshot, member);
                            item.sort_text = Some(format!("0_{}", member.name));
                            direct_member_names.insert(member.name.clone());
                            items.push(item);
                        }
                    }

                    // Inherited superclass members
                    let mut current_parent_id = symbol.parent_symbol;
                    let mut visited_classes = std::collections::HashSet::new();
                    visited_classes.insert(symbol.id);

                    while let Some(parent_id) = current_parent_id {
                        if !visited_classes.insert(parent_id) {
                            break;
                        }
                        if let Some(parent_class_entry) =
                            semantic_snapshot.symbol_table.get_symbol(parent_id)
                        {
                            for member_id in &parent_class_entry.members {
                                if let Some(member) =
                                    semantic_snapshot.symbol_table.get_symbol(*member_id)
                                    && !member.is_static
                                    && member.name != "constructor"
                                    && !member.name.starts_with('~')
                                    && !direct_member_names.contains(&member.name)
                                {
                                    let mut item = make_symbol_completion_item(snapshot, member);
                                    item.sort_text = Some(format!("1_{}", member.name));
                                    items.push(item);
                                }
                            }
                            current_parent_id = parent_class_entry.parent_symbol;
                        } else {
                            break;
                        }
                    }
                    break;
                }
            }
        }
    }

    // 3. Postfix completions
    if !receiver_expr.is_empty() {
        for (template_name, template_body, description) in POSTFIX_TEMPLATES {
            let replaced_template = template_body.replace("$EXPR", receiver_expr);
            let start_col = position
                .character
                .saturating_sub((receiver_expr.len() + 1 + partial_member_prefix.len()) as u32);
            let replace_range = Range {
                start: Position {
                    line: position.line,
                    character: start_col,
                },
                end: position,
            };

            items.push(CompletionItem {
                label: format!(".{template_name}"),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: Some((*description).to_string()),
                sort_text: Some(format!("2_{template_name}")),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("```solix\n{replaced_template}\n```"),
                })),
                insert_text: Some(replaced_template.clone()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                text_edit: Some(tower_lsp::lsp_types::CompletionTextEdit::Edit(TextEdit {
                    range: replace_range,
                    new_text: replaced_template,
                })),
                ..Default::default()
            });
        }
    }

    // Filter by partial_member_prefix
    let lower_partial = partial_member_prefix.to_ascii_lowercase();
    if !lower_partial.is_empty() {
        items.retain(|item| item.label.to_ascii_lowercase().starts_with(&lower_partial));
    }
}

fn handle_scope_resolution_completion(
    workspace: &WorkspaceManager,
    snapshot: &AnalysisSnapshot,
    receiver_expr: &str,
    partial_member_prefix: &str,
    items: &mut Vec<CompletionItem>,
) {
    let semantic = match snapshot.last_good_semantic.as_ref() {
        Some(semantic_snapshot) => semantic_snapshot,
        None => return,
    };

    // Case 1: Class receiver
    for symbol in &semantic.symbol_table.symbols {
        if (symbol.name == receiver_expr || symbol.fully_qualified_name == receiver_expr)
            && let Type::Class { symbol_name, .. } = &symbol.symbol_type
            && (symbol.name == *symbol_name || symbol.fully_qualified_name == *symbol_name)
        {
            for member_id in &symbol.members {
                if let Some(member) = semantic.symbol_table.get_symbol(*member_id)
                    && (member.is_static || member.is_const)
                    && member.name != "constructor"
                    && !member.name.starts_with('~')
                {
                    let mut item = make_symbol_completion_item(snapshot, member);
                    item.sort_text = Some(format!("0_{}", member.name));
                    items.push(item);
                }
            }
            let lower_partial = partial_member_prefix.to_ascii_lowercase();
            if !lower_partial.is_empty() {
                items.retain(|item| item.label.to_ascii_lowercase().starts_with(&lower_partial));
            }
            return;
        }
    }

    // Case 2: Enum receiver
    for symbol in &semantic.symbol_table.symbols {
        if (symbol.name == receiver_expr || symbol.fully_qualified_name == receiver_expr)
            && let Type::Enum { symbol_name, .. } = &symbol.symbol_type
            && (symbol.name == *symbol_name || symbol.fully_qualified_name == *symbol_name)
        {
            for member_id in &symbol.members {
                if let Some(member) = semantic.symbol_table.get_symbol(*member_id) {
                    items.push(CompletionItem {
                        label: member.name.clone(),
                        kind: Some(CompletionItemKind::ENUM_MEMBER),
                        detail: Some(format!("enum variant {}::{}", symbol.name, member.name)),
                        sort_text: Some(format!("0_{}", member.name)),
                        ..Default::default()
                    });
                }
            }
            let lower_partial = partial_member_prefix.to_ascii_lowercase();
            if !lower_partial.is_empty() {
                items.retain(|item| item.label.to_ascii_lowercase().starts_with(&lower_partial));
            }
            return;
        }
    }

    // Case 3: Package namespace receiver
    let normalized_namespace = receiver_expr.replace("::", ".");
    let root_package = normalized_namespace
        .split('.')
        .next()
        .unwrap_or(&normalized_namespace);
    let package_index = workspace.resolve_package_index(root_package);

    let child_prefix = format!("{normalized_namespace}.");
    let mut suggested_subpackages = std::collections::HashSet::new();
    for namespace_path in &package_index.namespaces {
        if let Some(sub_path) = namespace_path.strip_prefix(&child_prefix) {
            let immediate_child = sub_path.split('.').next().unwrap_or(sub_path);
            if suggested_subpackages.insert(immediate_child.to_string()) {
                items.push(CompletionItem {
                    label: immediate_child.to_string(),
                    kind: Some(CompletionItemKind::MODULE),
                    detail: Some(format!("namespace {receiver_expr}::{immediate_child}")),
                    sort_text: Some(format!("0_{immediate_child}")),
                    ..Default::default()
                });
            }
        }
    }

    if let Some(symbol_list) = package_index
        .symbols_by_namespace
        .get(&normalized_namespace)
    {
        for (symbol_name, symbol_kind, detail_signature) in symbol_list {
            items.push(CompletionItem {
                label: symbol_name.clone(),
                kind: Some(*symbol_kind),
                detail: Some(detail_signature.clone()),
                sort_text: Some(format!("1_{symbol_name}")),
                ..Default::default()
            });
        }
    }

    let qualified_prefix = format!("{receiver_expr}::");
    for symbol in &semantic.symbol_table.symbols {
        if symbol.fully_qualified_name.starts_with(&qualified_prefix) {
            let remaining = &symbol.fully_qualified_name[qualified_prefix.len()..];
            if !remaining.contains("::") && !remaining.is_empty() {
                let mut item = make_symbol_completion_item(snapshot, symbol);
                item.sort_text = Some(format!("1_{remaining}"));
                items.push(item);
            }
        }
    }

    let lower_partial = partial_member_prefix.to_ascii_lowercase();
    if !lower_partial.is_empty() {
        items.retain(|item| item.label.to_ascii_lowercase().starts_with(&lower_partial));
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
        items.push(CompletionItem {
            label: "main;".to_string(),
            kind: Some(CompletionItemKind::MODULE),
            detail: Some("Main package declaration".to_string()),
            ..Default::default()
        });
        return;
    }

    if trimmed.starts_with("import ") || trimmed.starts_with("extern import ") {
        items.push(CompletionItem {
            label: "solix.io.Console;".to_string(),
            kind: Some(CompletionItemKind::MODULE),
            detail: Some("Standard console I/O".to_string()),
            ..Default::default()
        });
        for other_snap in workspace.all_snapshots() {
            if let Some(ast) = &other_snap.ast {
                for decl in &ast.declarations {
                    if let Declaration::Class(class_decl) = decl {
                        let name = get_span_text(&other_snap.source_text, class_decl.name_span);
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
                items.push(make_symbol_completion_item(snapshot, symbol));
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

fn make_symbol_completion_item(
    snapshot: &AnalysisSnapshot,
    symbol: &SymbolEntry,
) -> CompletionItem {
    let (item_kind, detail, label_details) = match &symbol.symbol_type {
        Type::FunctionPointer {
            return_type,
            parameter_types,
        } => {
            let parameter_strings = parameter_types
                .iter()
                .enumerate()
                .map(|(param_index, param_type)| {
                    if let Some(param_name) = symbol.parameter_names.get(param_index)
                        && !param_name.is_empty()
                    {
                        format!("{param_type} {param_name}")
                    } else {
                        param_type.to_string()
                    }
                })
                .collect::<Vec<String>>()
                .join(", ");
            (
                CompletionItemKind::METHOD,
                Some(symbol.to_signature_string()),
                Some(CompletionItemLabelDetails {
                    detail: Some(format!("({parameter_strings})")),
                    description: Some(return_type.to_string()),
                }),
            )
        }
        Type::Class { .. } => (
            CompletionItemKind::CLASS,
            Some(format!("class {}", symbol.name)),
            None,
        ),
        Type::Enum { .. } => (
            CompletionItemKind::ENUM,
            Some(format!("enum {}", symbol.name)),
            None,
        ),
        _ => (
            CompletionItemKind::FIELD,
            Some(format!("{} {}", symbol.symbol_type, symbol.name)),
            Some(CompletionItemLabelDetails {
                detail: None,
                description: Some(symbol.symbol_type.to_string()),
            }),
        ),
    };

    let documentation = snapshot
        .extract_doc_comment_for_span(symbol.span)
        .map(|doc_text| {
            Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: doc_text,
            })
        });

    CompletionItem {
        label: symbol.name.clone(),
        kind: Some(item_kind),
        detail,
        label_details,
        documentation,
        ..Default::default()
    }
}

fn clamp_to_char_boundary(source_text: &str, mut byte_offset: usize) -> usize {
    if byte_offset >= source_text.len() {
        return source_text.len();
    }
    while byte_offset > 0 && !source_text.is_char_boundary(byte_offset) {
        byte_offset -= 1;
    }
    byte_offset
}

fn get_line_prefix(source_text: &str, offset: usize) -> &str {
    let clamped_offset = clamp_to_char_boundary(source_text, offset.min(source_text.len()));
    let prefix_str = &source_text[..clamped_offset];
    let last_newline = prefix_str.rfind('\n').map_or(0, |idx| idx + 1);
    let safe_start = clamp_to_char_boundary(prefix_str, last_newline);
    &prefix_str[safe_start..]
}

fn get_line_suffix(source_text: &str, offset: usize) -> &str {
    let clamped_offset = clamp_to_char_boundary(source_text, offset.min(source_text.len()));
    let suffix_str = &source_text[clamped_offset..];
    let next_newline = suffix_str.find('\n').unwrap_or(suffix_str.len());
    let safe_end = clamp_to_char_boundary(suffix_str, next_newline);
    &suffix_str[..safe_end]
}

fn handle_import_completion(
    workspace: &WorkspaceManager,
    snapshot: &AnalysisSnapshot,
    import_path_prefix: &str,
    has_trailing_semicolon: bool,
    items: &mut Vec<CompletionItem>,
) {
    let clean_path_prefix = import_path_prefix.trim_start();

    if let Some(last_dot_index) = clean_path_prefix.rfind('.') {
        // Inner segment or subpackage completion (e.g. "solix." or "solix.io." or "solix.io.C")
        let parent_namespace = clean_path_prefix[..last_dot_index].trim();
        let current_query = clean_path_prefix[last_dot_index + 1..].trim();

        let root_package_name = parent_namespace
            .split('.')
            .next()
            .unwrap_or(parent_namespace);
        let package_index = workspace.resolve_package_index(root_package_name);

        // 1. Suggest immediate child namespaces
        let mut suggested_subpackages = std::collections::HashSet::new();
        let child_prefix = format!("{parent_namespace}.");
        for namespace_path in &package_index.namespaces {
            if let Some(sub_path) = namespace_path.strip_prefix(&child_prefix) {
                let immediate_child = sub_path.split('.').next().unwrap_or(sub_path);
                if (current_query.is_empty() || immediate_child.starts_with(current_query))
                    && suggested_subpackages.insert(immediate_child.to_string())
                {
                    items.push(CompletionItem {
                        label: immediate_child.to_string(),
                        kind: Some(CompletionItemKind::MODULE),
                        detail: Some(format!("namespace {parent_namespace}.{immediate_child}")),
                        insert_text: Some(immediate_child.to_string()),
                        ..Default::default()
                    });
                }
            }
        }

        // 2. Suggest public symbols in parent_namespace
        if let Some(symbol_list) = package_index.symbols_by_namespace.get(parent_namespace) {
            for (symbol_name, symbol_kind, detail_signature) in symbol_list {
                if current_query.is_empty() || symbol_name.starts_with(current_query) {
                    let insertion_text = if has_trailing_semicolon {
                        symbol_name.clone()
                    } else {
                        format!("{symbol_name};")
                    };
                    items.push(CompletionItem {
                        label: symbol_name.clone(),
                        kind: Some(*symbol_kind),
                        detail: Some(detail_signature.clone()),
                        insert_text: Some(insertion_text),
                        ..Default::default()
                    });
                }
            }
        }

        // 3. Suggest wildcard '*'
        if current_query.is_empty() || "*".starts_with(current_query) {
            let wildcard_insertion = if has_trailing_semicolon {
                "*".to_string()
            } else {
                "*;".to_string()
            };
            items.push(CompletionItem {
                label: "*".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(format!("Import all symbols from {parent_namespace}")),
                insert_text: Some(wildcard_insertion),
                ..Default::default()
            });
        }
    } else {
        // Root segment completion (e.g. "import " or "import sol")
        let current_query = clean_path_prefix.trim();
        let mut package_candidates = Vec::new();

        if let Some(solver) = workspace.solver() {
            package_candidates.extend(solver.list_packages());
        }

        for workspace_snapshot in workspace.all_snapshots() {
            if let Some(ast) = &workspace_snapshot.ast
                && let Some(package_statement) = &ast.package
                && let Some(first_span) = package_statement.path.first()
            {
                let start_index = first_span.start as usize;
                let end_index = first_span.end as usize;
                if end_index <= workspace_snapshot.source_text.len() {
                    package_candidates
                        .push(workspace_snapshot.source_text[start_index..end_index].to_string());
                }
            }
        }

        if let Some(manifest) = workspace.project_manifest_for_uri(&snapshot.uri)
            && !manifest.configuration.name.is_empty()
        {
            package_candidates.push(manifest.configuration.name);
        }

        let mut seen_packages = std::collections::HashSet::new();
        package_candidates.retain(|candidate| seen_packages.insert(candidate.clone()));
        package_candidates.sort();

        for candidate_name in package_candidates {
            if current_query.is_empty() || candidate_name.starts_with(current_query) {
                items.push(CompletionItem {
                    label: candidate_name.clone(),
                    kind: Some(CompletionItemKind::MODULE),
                    detail: Some(format!("Package {candidate_name}")),
                    insert_text: Some(candidate_name),
                    ..Default::default()
                });
            }
        }
    }
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
