//! Code actions: Quick Fixes (multi-level auto-imports, `.` <-> `::`, `override`/visibility,
//! missing return, method stubs, inexhaustive switch, dead code, syntax fixes) and Refactorings.

#![forbid(unsafe_code)]

use crate::diagnostics::span_to_range;
use crate::diagnostics::{byte_offset_to_position, position_to_byte_offset};
use crate::workspace::{AnalysisSnapshot, WorkspaceManager};
use solix_common::Span;
use solix_compilation::phases::parser::statements::ProgramAst;
use solix_compilation::phases::parser::statements::declarations::{Declaration, Modifier};
use std::collections::{BTreeSet, HashMap, HashSet};
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams, Diagnostic, NumberOrString,
    Position, Range, TextEdit, Url, WorkspaceEdit,
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
            Some(NumberOrString::String(code_text)) => code_text.as_str(),
            _ => "",
        };

        add_access_delimiter_actions(snapshot, diagnostic, code_str, &mut actions);
        add_undefined_symbol_actions(workspace, snapshot, diagnostic, code_str, &mut actions);
        add_override_and_visibility_actions(snapshot, diagnostic, code_str, &mut actions);
        add_missing_return_action(snapshot, diagnostic, code_str, &mut actions);
        add_missing_method_stub_action(snapshot, diagnostic, code_str, &mut actions);
        add_switch_exhaustiveness_action(snapshot, diagnostic, code_str, &mut actions);
        add_syntax_and_cleanup_actions(snapshot, diagnostic, code_str, &mut actions);
    }

    // 2. Refactorings: Organize Imports
    add_organize_imports_action(snapshot, &mut actions);

    if let Some(requested_kinds) = &params.context.only {
        actions.retain(|action| match action {
            CodeActionOrCommand::CodeAction(code_action) => {
                if let Some(action_kind) = &code_action.kind {
                    requested_kinds.iter().any(|requested_kind| {
                        action_kind.as_str().starts_with(requested_kind.as_str())
                    })
                } else {
                    true
                }
            }
            CodeActionOrCommand::Command(_) => true,
        });
    }

    actions
}

fn build_single_edit_quickfix(
    uri: &Url,
    diagnostic: &Diagnostic,
    title: String,
    range: Range,
    new_text: String,
    is_preferred: bool,
) -> CodeActionOrCommand {
    let mut changes = HashMap::new();
    changes.insert(uri.clone(), vec![TextEdit { range, new_text }]);

    CodeActionOrCommand::CodeAction(CodeAction {
        title,
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![diagnostic.clone()]),
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            ..Default::default()
        }),
        is_preferred: Some(is_preferred),
        ..Default::default()
    })
}

fn add_access_delimiter_actions(
    snapshot: &AnalysisSnapshot,
    diagnostic: &Diagnostic,
    code_str: &str,
    actions: &mut Vec<CodeActionOrCommand>,
) {
    if code_str == "E0226"
        || (diagnostic.message.contains("static") && diagnostic.message.contains("via '.'"))
    {
        actions.push(build_single_edit_quickfix(
            &snapshot.uri,
            diagnostic,
            "Replace '.' with '::'".to_string(),
            diagnostic.range,
            "::".to_string(),
            true,
        ));
    }

    if code_str == "E0227"
        || (diagnostic.message.contains("instance") && diagnostic.message.contains("via '::'"))
    {
        actions.push(build_single_edit_quickfix(
            &snapshot.uri,
            diagnostic,
            "Replace '::' with '.'".to_string(),
            diagnostic.range,
            ".".to_string(),
            true,
        ));
    }
}

fn add_undefined_symbol_actions(
    workspace: &WorkspaceManager,
    snapshot: &AnalysisSnapshot,
    diagnostic: &Diagnostic,
    code_str: &str,
    actions: &mut Vec<CodeActionOrCommand>,
) {
    if code_str != "E0201"
        && !diagnostic.message.contains("Undefined symbol")
        && !diagnostic.message.contains("Unresolved symbol")
    {
        return;
    }

    let unresolved_name = extract_symbol_name_from_message(&diagnostic.message);
    if unresolved_name.is_empty() {
        return;
    }

    let candidates = collect_import_candidates(workspace, snapshot, unresolved_name);
    let existing_imports = collect_existing_import_texts(snapshot);
    let (insert_pos, leading_prefix, trailing_suffix) =
        compute_import_insert_position(&snapshot.source_text, snapshot.ast.as_ref());

    let single_match = candidates.len() == 1;
    let mut emitted_wildcards = HashSet::new();

    for (namespace_path, symbol_name) in &candidates {
        if namespace_path.is_empty() {
            let import_stmt = format!("import {symbol_name};");
            if !existing_imports.contains(&import_stmt) {
                actions.push(build_single_edit_quickfix(
                    &snapshot.uri,
                    diagnostic,
                    format!("Import '{symbol_name}'"),
                    Range::new(insert_pos, insert_pos),
                    format!("{leading_prefix}{import_stmt}{trailing_suffix}"),
                    single_match,
                ));
            }
            continue;
        }

        let exact_stmt = format!("import {namespace_path}.{symbol_name};");
        if !existing_imports.contains(&exact_stmt) {
            actions.push(build_single_edit_quickfix(
                &snapshot.uri,
                diagnostic,
                format!("Import '{namespace_path}.{symbol_name}'"),
                Range::new(insert_pos, insert_pos),
                format!("{leading_prefix}{exact_stmt}{trailing_suffix}"),
                single_match,
            ));
        }

        if emitted_wildcards.insert(namespace_path.clone()) {
            let wildcard_stmt = format!("import {namespace_path}.*;");
            if !existing_imports.contains(&wildcard_stmt) {
                actions.push(build_single_edit_quickfix(
                    &snapshot.uri,
                    diagnostic,
                    format!("Import all from '{namespace_path}.*'"),
                    Range::new(insert_pos, insert_pos),
                    format!("{leading_prefix}{wildcard_stmt}{trailing_suffix}"),
                    false,
                ));
            }
        }
    }

    // Typo suggestion ("Did you mean ...?") via Levenshtein distance <= 2
    if let Some(semantic) = &snapshot.last_good_semantic {
        let mut typo_candidates: BTreeSet<String> = BTreeSet::new();
        for symbol in &semantic.symbol_table.symbols {
            if (symbol.source_id == snapshot.source_id || symbol.is_public())
                && symbol.name != unresolved_name
                && levenshtein_distance(unresolved_name, &symbol.name) <= 2
            {
                typo_candidates.insert(symbol.name.clone());
            }
        }
        for typo_candidate in typo_candidates.into_iter().take(3) {
            actions.push(build_single_edit_quickfix(
                &snapshot.uri,
                diagnostic,
                format!("Replace with '{typo_candidate}'"),
                diagnostic.range,
                typo_candidate,
                candidates.is_empty(),
            ));
        }
    }
}

fn add_override_and_visibility_actions(
    snapshot: &AnalysisSnapshot,
    diagnostic: &Diagnostic,
    code_str: &str,
    actions: &mut Vec<CodeActionOrCommand>,
) {
    if code_str == "E0207"
        || (diagnostic.message.contains("override") && diagnostic.message.contains("missing"))
    {
        actions.push(build_single_edit_quickfix(
            &snapshot.uri,
            diagnostic,
            "Add 'override' modifier".to_string(),
            Range::new(diagnostic.range.start, diagnostic.range.start),
            "override ".to_string(),
            true,
        ));
    }

    if (code_str == "E0282" || code_str == "E0283")
        && let Some(override_range) = find_keyword_range_near_line(
            &snapshot.source_text,
            diagnostic.range.start.line,
            "override ",
        )
    {
        actions.push(build_single_edit_quickfix(
            &snapshot.uri,
            diagnostic,
            "Remove 'override' modifier".to_string(),
            override_range,
            String::new(),
            true,
        ));
    }

    if code_str == "E0268" {
        for visibility_keyword in ["private", "protected", "internal"] {
            if let Some(vis_range) = find_keyword_range_near_line(
                &snapshot.source_text,
                diagnostic.range.start.line,
                visibility_keyword,
            ) {
                actions.push(build_single_edit_quickfix(
                    &snapshot.uri,
                    diagnostic,
                    "Change visibility to 'public'".to_string(),
                    vis_range,
                    "public".to_string(),
                    true,
                ));
                break;
            }
        }
    }
}

fn add_missing_return_action(
    snapshot: &AnalysisSnapshot,
    diagnostic: &Diagnostic,
    code_str: &str,
    actions: &mut Vec<CodeActionOrCommand>,
) {
    if code_str != "E0212" && !diagnostic.message.contains("does not return a value") {
        return;
    }

    let diag_offset = position_to_byte_offset(&snapshot.source_text, diagnostic.range.start);
    if let Some((body_end_offset, return_literal)) =
        find_enclosing_method_return_info(snapshot, diag_offset)
    {
        let insert_pos = byte_offset_to_position(&snapshot.source_text, body_end_offset);
        let indent = detect_line_indent(&snapshot.source_text, diagnostic.range.start.line);
        let inner_indent = format!("{indent}    ");
        actions.push(build_single_edit_quickfix(
            &snapshot.uri,
            diagnostic,
            format!("Insert 'return {return_literal};'"),
            Range::new(insert_pos, insert_pos),
            format!("{inner_indent}return {return_literal};\n{indent}"),
            true,
        ));
    }
}

fn add_missing_method_stub_action(
    snapshot: &AnalysisSnapshot,
    diagnostic: &Diagnostic,
    code_str: &str,
    actions: &mut Vec<CodeActionOrCommand>,
) {
    if code_str != "E0265" && code_str != "E0206" {
        return;
    }

    let method_name = extract_symbol_name_from_message(&diagnostic.message);
    if method_name.is_empty() {
        return;
    }

    let diag_offset = position_to_byte_offset(&snapshot.source_text, diagnostic.range.start);
    if let Some(class_end_offset) = find_enclosing_class_end(snapshot, diag_offset) {
        let insert_pos = byte_offset_to_position(&snapshot.source_text, class_end_offset);
        let stub = format!(
            "\n    public override void {method_name}() {{\n        // TODO: implement\n    }}\n"
        );
        actions.push(build_single_edit_quickfix(
            &snapshot.uri,
            diagnostic,
            format!("Generate stub for '{method_name}'"),
            Range::new(insert_pos, insert_pos),
            stub,
            true,
        ));
    }
}

fn add_switch_exhaustiveness_action(
    snapshot: &AnalysisSnapshot,
    diagnostic: &Diagnostic,
    code_str: &str,
    actions: &mut Vec<CodeActionOrCommand>,
) {
    if code_str != "E0306" {
        return;
    }

    let diag_offset = position_to_byte_offset(&snapshot.source_text, diagnostic.range.start);
    if let Some(closing_brace_offset) = find_matching_block_end(&snapshot.source_text, diag_offset)
    {
        let insert_pos = byte_offset_to_position(&snapshot.source_text, closing_brace_offset);
        actions.push(build_single_edit_quickfix(
            &snapshot.uri,
            diagnostic,
            "Add 'default' switch arm".to_string(),
            Range::new(insert_pos, insert_pos),
            "    default: {\n        break;\n    }\n".to_string(),
            true,
        ));
    }
}

fn add_syntax_and_cleanup_actions(
    snapshot: &AnalysisSnapshot,
    diagnostic: &Diagnostic,
    code_str: &str,
    actions: &mut Vec<CodeActionOrCommand>,
) {
    if code_str == "E0101" {
        actions.push(build_single_edit_quickfix(
            &snapshot.uri,
            diagnostic,
            "Insert missing ';'".to_string(),
            Range::new(diagnostic.range.end, diagnostic.range.end),
            ";".to_string(),
            true,
        ));
    }

    if code_str == "E0192" {
        actions.push(build_single_edit_quickfix(
            &snapshot.uri,
            diagnostic,
            "Remove unexpected ';'".to_string(),
            diagnostic.range,
            String::new(),
            true,
        ));
    }

    if code_str == "E0271" || code_str == "W0001" {
        actions.push(build_single_edit_quickfix(
            &snapshot.uri,
            diagnostic,
            "Remove unreachable code".to_string(),
            diagnostic.range,
            String::new(),
            true,
        ));
    }

    if code_str == "E0112" {
        actions.push(build_single_edit_quickfix(
            &snapshot.uri,
            diagnostic,
            "Remove duplicate 'package' statement".to_string(),
            diagnostic.range,
            String::new(),
            true,
        ));
    }

    if code_str == "E0153" {
        let span_text = get_range_text(&snapshot.source_text, diagnostic.range);
        if span_text.starts_with('\'') && span_text.ends_with('\'') && span_text.len() >= 2 {
            let inner = &span_text[1..span_text.len() - 1];
            actions.push(build_single_edit_quickfix(
                &snapshot.uri,
                diagnostic,
                "Convert to string literal".to_string(),
                diagnostic.range,
                format!("\"{inner}\""),
                true,
            ));
        }
    }
}

fn add_organize_imports_action(
    snapshot: &AnalysisSnapshot,
    actions: &mut Vec<CodeActionOrCommand>,
) {
    if let Some(ast) = &snapshot.ast
        && ast.imports.len() > 1
    {
        let mut import_lines: Vec<String> = ast
            .imports
            .iter()
            .map(|imp| {
                get_span_text(&snapshot.source_text, imp.span)
                    .trim()
                    .to_string()
            })
            .collect();
        import_lines.sort();
        import_lines.dedup();

        let first_import = &ast.imports[0];
        let last_import = &ast.imports[ast.imports.len() - 1];
        let replace_range = span_to_range(
            &snapshot.source_text,
            first_import.span.combine(last_import.span),
        );

        let mut changes = HashMap::new();
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
}

fn collect_import_candidates(
    workspace: &WorkspaceManager,
    snapshot: &AnalysisSnapshot,
    unresolved_name: &str,
) -> Vec<(String, String)> {
    let mut found: BTreeSet<(String, String)> = BTreeSet::new();

    // 1. Search workspace snapshots (classes, interfaces, enums, functions, public/const fields)
    for other_snap in workspace.all_snapshots() {
        if other_snap.uri == snapshot.uri {
            continue;
        }
        let Some(ast) = &other_snap.ast else {
            continue;
        };
        let package_path = extract_ast_package_path(&other_snap.source_text, ast);
        for decl in &ast.declarations {
            let maybe_name: Option<&str> = match decl {
                Declaration::Class(class_decl)
                    if class_decl.modifiers.contains(&Modifier::Public) =>
                {
                    Some(get_span_text(&other_snap.source_text, class_decl.name_span))
                }
                Declaration::Interface(iface_decl)
                    if iface_decl.modifiers.contains(&Modifier::Public) =>
                {
                    Some(get_span_text(&other_snap.source_text, iface_decl.name_span))
                }
                Declaration::Enum(enum_decl) if enum_decl.modifiers.contains(&Modifier::Public) => {
                    Some(get_span_text(&other_snap.source_text, enum_decl.name_span))
                }
                Declaration::Method(method_decl)
                    if method_decl.modifiers.contains(&Modifier::Public) =>
                {
                    Some(get_span_text(
                        &other_snap.source_text,
                        method_decl.name_span,
                    ))
                }
                Declaration::Field(field_decl)
                    if field_decl.modifiers.contains(&Modifier::Public)
                        || field_decl.modifiers.contains(&Modifier::Const) =>
                {
                    Some(get_span_text(&other_snap.source_text, field_decl.name_span))
                }
                _ => None,
            };
            if let Some(decl_name) = maybe_name
                && decl_name == unresolved_name
            {
                found.insert((package_path.clone(), decl_name.to_string()));
            }
        }
    }

    // 2. Search manifest dependencies, cached dependencies, and standard packages
    let mut package_roots: BTreeSet<String> = BTreeSet::new();
    package_roots.insert("solix".to_string());
    package_roots.insert("std".to_string());
    for cached_name in workspace.cached_dependency_package_names() {
        package_roots.insert(cached_name);
    }
    if let Some(manifest) = workspace.project_manifest_for_uri(&snapshot.uri) {
        for dep in manifest.resolved_dependencies() {
            package_roots.insert(dep.package_name);
        }
    }

    for root_pkg in package_roots {
        let pkg_index = workspace.resolve_package_index(&root_pkg);
        for (namespace_path, symbol_list) in &pkg_index.symbols_by_namespace {
            for (sym_name, _kind, _detail) in symbol_list {
                if sym_name == unresolved_name {
                    found.insert((namespace_path.clone(), sym_name.clone()));
                }
            }
        }
    }

    found.into_iter().collect()
}

fn extract_ast_package_path(source_text: &str, ast: &ProgramAst) -> String {
    let Some(pkg_stmt) = &ast.package else {
        return String::new();
    };
    pkg_stmt
        .path
        .iter()
        .map(|span| get_span_text(source_text, *span))
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join(".")
}

fn collect_existing_import_texts(snapshot: &AnalysisSnapshot) -> HashSet<String> {
    let mut existing = HashSet::new();
    if let Some(ast) = &snapshot.ast {
        for imp in &ast.imports {
            let text = get_span_text(&snapshot.source_text, imp.span)
                .trim()
                .to_string();
            if !text.is_empty() {
                existing.insert(text);
            }
        }
    }
    existing
}

fn compute_import_insert_position(
    source_text: &str,
    ast: Option<&ProgramAst>,
) -> (Position, String, String) {
    if let Some(program) = ast {
        if let Some(last_import) = program.imports.last() {
            let end_offset = (last_import.span.end as usize).min(source_text.len());
            let after_line_offset = advance_past_line_end(source_text, end_offset);
            let pos = byte_offset_to_position(source_text, after_line_offset);
            if after_line_offset == end_offset && !source_text[..end_offset].ends_with('\n') {
                return (pos, "\n".to_string(), "\n".to_string());
            }
            return (pos, String::new(), "\n".to_string());
        }

        if let Some(pkg_stmt) = &program.package {
            let end_offset = (pkg_stmt.span.end as usize).min(source_text.len());
            let after_line_offset = advance_past_line_end(source_text, end_offset);
            let pos = byte_offset_to_position(source_text, after_line_offset);
            if after_line_offset == end_offset && !source_text[..end_offset].ends_with('\n') {
                return (pos, "\n\n".to_string(), "\n".to_string());
            }
            return (pos, "\n".to_string(), "\n".to_string());
        }
    }

    (
        Position {
            line: 0,
            character: 0,
        },
        String::new(),
        "\n".to_string(),
    )
}

fn advance_past_line_end(source_text: &str, offset: usize) -> usize {
    let clamped = offset.min(source_text.len());
    if let Some(rel_nl) = source_text[clamped..].find('\n') {
        clamped + rel_nl + 1
    } else {
        clamped
    }
}

fn find_keyword_range_near_line(source_text: &str, line: u32, keyword: &str) -> Option<Range> {
    let target_line = source_text.lines().nth(line as usize)?;
    let col = target_line.find(keyword)?;
    let start_utf16 = target_line[..col].encode_utf16().count() as u32;
    let end_utf16 = start_utf16 + keyword.encode_utf16().count() as u32;
    Some(Range {
        start: Position {
            line,
            character: start_utf16,
        },
        end: Position {
            line,
            character: end_utf16,
        },
    })
}

fn detect_line_indent(source_text: &str, line: u32) -> String {
    let Some(target_line) = source_text.lines().nth(line as usize) else {
        return String::new();
    };
    target_line
        .chars()
        .take_while(|ch| *ch == ' ' || *ch == '\t')
        .collect()
}

fn find_enclosing_method_return_info(
    snapshot: &AnalysisSnapshot,
    diag_offset: usize,
) -> Option<(usize, &'static str)> {
    let ast = snapshot.ast.as_ref()?;
    let offset_u32 = diag_offset as u32;

    for decl in &ast.declarations {
        match decl {
            Declaration::Method(method_decl) => {
                if let Some(body) = &method_decl.body
                    && method_decl.span.start <= offset_u32
                    && offset_u32 <= method_decl.span.end
                {
                    let ret_text =
                        get_span_text(&snapshot.source_text, method_decl.return_type.span()).trim();
                    let insert_offset = body.span.end.saturating_sub(1) as usize;
                    return Some((insert_offset, default_return_literal(ret_text)));
                }
            }
            Declaration::Class(class_decl) => {
                for member in &class_decl.members {
                    if let solix_compilation::phases::parser::statements::ClassMember::Method(
                        method_decl,
                    ) = member
                        && let Some(body) = &method_decl.body
                        && method_decl.span.start <= offset_u32
                        && offset_u32 <= method_decl.span.end
                    {
                        let ret_text =
                            get_span_text(&snapshot.source_text, method_decl.return_type.span())
                                .trim();
                        let insert_offset = body.span.end.saturating_sub(1) as usize;
                        return Some((insert_offset, default_return_literal(ret_text)));
                    }
                }
            }
            _ => {}
        }
    }
    None
}

fn default_return_literal(return_type_text: &str) -> &'static str {
    match return_type_text {
        "bool" => "false",
        "float32" | "float64" => "0.0",
        "string" => "\"\"",
        "char" => "'\\0'",
        "int8" | "int16" | "int32" | "int64" | "uint8" | "uint16" | "uint32" | "uint64" => "0",
        _ => "null",
    }
}

fn find_enclosing_class_end(snapshot: &AnalysisSnapshot, diag_offset: usize) -> Option<usize> {
    let ast = snapshot.ast.as_ref()?;
    let offset_u32 = diag_offset as u32;
    for decl in &ast.declarations {
        if let Declaration::Class(class_decl) = decl
            && class_decl.span.start <= offset_u32
            && offset_u32 <= class_decl.span.end
        {
            return Some(class_decl.span.end.saturating_sub(1) as usize);
        }
    }
    None
}

fn find_matching_block_end(source_text: &str, start_offset: usize) -> Option<usize> {
    let clamped = start_offset.min(source_text.len());
    let open_rel = source_text[clamped..].find('{')?;
    let mut depth = 0usize;
    for (idx, ch) in source_text[clamped + open_rel..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(clamped + open_rel + idx);
                }
            }
            _ => {}
        }
    }
    None
}

fn levenshtein_distance(left_str: &str, right_str: &str) -> usize {
    let left_chars: Vec<char> = left_str.chars().collect();
    let right_chars: Vec<char> = right_str.chars().collect();
    let left_len = left_chars.len();
    let right_len = right_chars.len();
    if left_len == 0 {
        return right_len;
    }
    if right_len == 0 {
        return left_len;
    }

    let mut prev_row: Vec<usize> = (0..=right_len).collect();
    let mut curr_row = vec![0usize; right_len + 1];

    for (left_idx, left_ch) in left_chars.iter().enumerate() {
        curr_row[0] = left_idx + 1;
        for (right_idx, right_ch) in right_chars.iter().enumerate() {
            let cost = usize::from(left_ch != right_ch);
            curr_row[right_idx + 1] = (prev_row[right_idx + 1] + 1)
                .min(curr_row[right_idx] + 1)
                .min(prev_row[right_idx] + cost);
        }
        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[right_len]
}

fn extract_symbol_name_from_message(msg: &str) -> &str {
    if let Some(start) = msg.find('\'')
        && let Some(end) = msg[start + 1..].find('\'')
    {
        return &msg[start + 1..start + 1 + end];
    }
    ""
}

fn get_span_text(source_text: &str, span: Span) -> &str {
    let start_offset = span.start as usize;
    let end_offset = span.end as usize;
    if start_offset <= end_offset
        && end_offset <= source_text.len()
        && source_text.is_char_boundary(start_offset)
        && source_text.is_char_boundary(end_offset)
    {
        &source_text[start_offset..end_offset]
    } else {
        ""
    }
}

fn get_range_text(source_text: &str, range: Range) -> &str {
    let start_offset = position_to_byte_offset(source_text, range.start);
    let end_offset = position_to_byte_offset(source_text, range.end);
    if start_offset <= end_offset
        && end_offset <= source_text.len()
        && source_text.is_char_boundary(start_offset)
        && source_text.is_char_boundary(end_offset)
    {
        &source_text[start_offset..end_offset]
    } else {
        ""
    }
}
