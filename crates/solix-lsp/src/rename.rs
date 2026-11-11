//! Multi-file safe symbol rename and prepare rename validation.

#![forbid(unsafe_code)]

use crate::diagnostics::{position_to_byte_offset, span_to_range};
use crate::navigation::find_references;
use crate::workspace::{AnalysisSnapshot, WorkspaceManager};
use solix_common::tokens::lookup_keyword;
use std::collections::HashMap;
use tower_lsp::lsp_types::{
    Position, PrepareRenameResponse, TextEdit, Url, WorkspaceEdit,
};

/// Validates whether the symbol at `position` can be safely renamed.
#[must_use]
pub fn prepare_rename(
    snapshot: &AnalysisSnapshot,
    position: Position,
) -> Option<PrepareRenameResponse> {
    let offset = position_to_byte_offset(&snapshot.source_text, position);
    let symbol = snapshot.symbol_at_offset(offset)?;

    // Prohibit renaming built-in main entry point
    if symbol.name == "main" {
        return None;
    }

    let range = span_to_range(&snapshot.source_text, symbol.span);
    Some(PrepareRenameResponse::Range(range))
}

/// Executes safe multi-file rename, rejecting reserved keywords and invalid identifiers.
#[must_use]
pub fn rename_symbol(
    workspace: &WorkspaceManager,
    snapshot: &AnalysisSnapshot,
    position: Position,
    new_name: String,
) -> Option<WorkspaceEdit> {
    // 1. Validate new name is a legal Solix identifier and not a reserved keyword
    if !is_valid_identifier(&new_name) || lookup_keyword(&new_name).is_some() {
        return None;
    }

    // 2. Locate all references including the declaration across the workspace
    let references = find_references(workspace, snapshot, position, true)?;

    let mut changes_map: HashMap<Url, Vec<TextEdit>> = HashMap::new();

    for location in references {
        changes_map
            .entry(location.uri)
            .or_default()
            .push(TextEdit {
                range: location.range,
                new_text: new_name.clone(),
            });
    }

    Some(WorkspaceEdit {
        changes: Some(changes_map),
        ..Default::default()
    })
}

fn is_valid_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let mut chars = name.chars();
    let first = chars.next().unwrap();
    if !first.is_alphabetic() && first != '_' {
        return false;
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
}
