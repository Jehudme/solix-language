//! Multi-file safe symbol rename and prepare rename validation.

#![forbid(unsafe_code)]

use crate::diagnostics::{position_to_byte_offset, span_to_range};
use crate::navigation::find_references;
use crate::workspace::{AnalysisSnapshot, WorkspaceManager};
use solix_common::tokens::lookup_keyword;
use std::collections::HashMap;
use tower_lsp::lsp_types::{Position, PrepareRenameResponse, TextEdit, Url, WorkspaceEdit};

/// Validates whether the symbol at `position` can be safely renamed.
#[must_use]
pub fn prepare_rename(
    snapshot: &AnalysisSnapshot,
    position: Position,
) -> Option<PrepareRenameResponse> {
    let offset = position_to_byte_offset(&snapshot.source_text, position);
    let symbol = snapshot.symbol_at_offset(offset)?;

    // Prohibit renaming built-in main entry point or external/read-only dependency symbols
    if symbol.name == "main"
        || symbol.source_id != snapshot.source_id
        || symbol.span == solix_common::Span::default()
    {
        return None;
    }

    let range = span_to_range(&snapshot.source_text, symbol.span);
    Some(PrepareRenameResponse::Range(range))
}

/// Executes safe multi-file rename, returning a descriptive error message when invalid.
pub fn rename_symbol_checked(
    workspace: &WorkspaceManager,
    snapshot: &AnalysisSnapshot,
    position: Position,
    new_name: String,
) -> Result<WorkspaceEdit, String> {
    // 1. Validate new name is a legal Solix identifier and not a reserved keyword
    if !is_valid_identifier(&new_name) {
        return Err(format!("'{new_name}' is not a valid Solix identifier"));
    }
    if lookup_keyword(&new_name).is_some() {
        return Err(format!(
            "Cannot rename symbol to reserved keyword '{new_name}'"
        ));
    }

    let offset = position_to_byte_offset(&snapshot.source_text, position);
    let target_symbol = snapshot
        .symbol_at_offset(offset)
        .ok_or_else(|| "No renameable symbol found at cursor position".to_string())?;

    if target_symbol.name == "main" {
        return Err("Cannot rename the 'main' entry point".to_string());
    }
    if target_symbol.source_id != snapshot.source_id
        || target_symbol.span == solix_common::Span::default()
    {
        return Err("Cannot rename external or read-only library symbol".to_string());
    }

    // 2. Check for scope collision with existing symbol in the same enclosing scope
    if let Some(semantic) = &snapshot.last_good_semantic {
        let has_collision = semantic.symbol_table.symbols.iter().any(|candidate| {
            candidate.id != target_symbol.id
                && candidate.source_id == snapshot.source_id
                && candidate.parent_symbol == target_symbol.parent_symbol
                && candidate.name == new_name
        });
        if has_collision {
            return Err(format!("Symbol '{new_name}' already exists in this scope"));
        }
    }

    // 3. Locate all references including the declaration across the workspace
    let references = find_references(workspace, snapshot, position, true)
        .ok_or_else(|| "Could not resolve references for symbol".to_string())?;

    let mut changes_map: HashMap<Url, Vec<TextEdit>> = HashMap::new();

    for location in references {
        changes_map.entry(location.uri).or_default().push(TextEdit {
            range: location.range,
            new_text: new_name.clone(),
        });
    }

    Ok(WorkspaceEdit {
        changes: Some(changes_map),
        ..Default::default()
    })
}

/// Executes safe multi-file rename, rejecting reserved keywords and invalid identifiers.
#[must_use]
pub fn rename_symbol(
    workspace: &WorkspaceManager,
    snapshot: &AnalysisSnapshot,
    position: Position,
    new_name: String,
) -> Option<WorkspaceEdit> {
    rename_symbol_checked(workspace, snapshot, position, new_name).ok()
}

fn is_valid_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_alphabetic() && first != '_' {
        return false;
    }
    chars.all(|ch| ch.is_alphanumeric() || ch == '_')
}
