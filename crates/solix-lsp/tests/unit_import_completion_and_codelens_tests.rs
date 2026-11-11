//! Comprehensive unit tests for solver-driven hierarchical import completion and manifest-aware CodeLens.
//! Milestone 034 verification.

#![forbid(unsafe_code)]

use solix_lsp::code_lens::code_lenses;
use solix_lsp::completion::completions;
use solix_lsp::workspace::WorkspaceManager;
use std::sync::Arc;
use tower_lsp::lsp_types::*;

#[test]
fn test_import_completion_root_packages_and_blt() {
    let workspace = Arc::new(WorkspaceManager::new(None, None));
    let file_uri = Url::parse("file:///workspace/test.slx").unwrap();
    let source_code = "import ";
    let snapshot = workspace.did_open(file_uri.clone(), 1, source_code.to_string());

    let completion_params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: file_uri },
            position: Position {
                line: 0,
                character: 7,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };

    let result = completions(&workspace, &snapshot, &completion_params);
    let item_labels: Vec<String> = result.items.iter().map(|item| item.label.clone()).collect();

    assert!(
        item_labels.contains(&"blt".to_string()),
        "Completion must contain built-in package 'blt': {item_labels:?}"
    );
}

#[test]
fn test_import_completion_blt_hierarchy_and_wildcard() {
    let workspace = Arc::new(WorkspaceManager::new(None, None));
    let file_uri = Url::parse("file:///workspace/test.slx").unwrap();

    // 1. User types 'import blt.'
    let subpackage_code = "import blt.";
    let subpackage_snapshot = workspace.did_open(file_uri.clone(), 1, subpackage_code.to_string());
    let subpackage_params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier {
                uri: file_uri.clone(),
            },
            position: Position {
                line: 0,
                character: 11,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };

    let subpackage_completions = completions(&workspace, &subpackage_snapshot, &subpackage_params);
    let subpackage_labels: Vec<String> = subpackage_completions
        .items
        .iter()
        .map(|item| item.label.clone())
        .collect();

    assert!(
        subpackage_labels.contains(&"io".to_string()),
        "Child namespace 'io' must be suggested under 'blt': {subpackage_labels:?}"
    );

    // 2. User types 'import blt.io.'
    let terminal_code = "import blt.io.";
    let terminal_snapshot = workspace.did_change(&file_uri, 2, terminal_code.to_string());
    let terminal_params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier {
                uri: file_uri.clone(),
            },
            position: Position {
                line: 0,
                character: 14,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };

    let terminal_completions = completions(&workspace, &terminal_snapshot, &terminal_params);
    let terminal_labels: Vec<String> = terminal_completions
        .items
        .iter()
        .map(|item| item.label.clone())
        .collect();

    assert!(
        terminal_labels.contains(&"println".to_string()),
        "'println' must be suggested under 'blt.io': {terminal_labels:?}"
    );
    assert!(
        terminal_labels.contains(&"print".to_string()),
        "'print' must be suggested under 'blt.io': {terminal_labels:?}"
    );
    assert!(
        terminal_labels.contains(&"*".to_string()),
        "Wildcard '*' must be suggested: {terminal_labels:?}"
    );

    let println_item = terminal_completions
        .items
        .iter()
        .find(|item| item.label == "println")
        .expect("println item must exist");
    assert_eq!(
        println_item.insert_text.as_deref(),
        Some("println;"),
        "Terminal symbol insert text must append trailing semicolon when missing"
    );
}

#[test]
fn test_import_completion_semicolon_deduplication() {
    let workspace = Arc::new(WorkspaceManager::new(None, None));
    let file_uri = Url::parse("file:///workspace/test.slx").unwrap();

    // Source already has trailing semicolon: 'import blt.io.;'
    let source_code = "import blt.io.;";
    let snapshot = workspace.did_open(file_uri.clone(), 1, source_code.to_string());

    // Cursor is placed between '.' and ';' (offset 14)
    let completion_params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: file_uri },
            position: Position {
                line: 0,
                character: 14,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };

    let result = completions(&workspace, &snapshot, &completion_params);
    let println_item = result
        .items
        .iter()
        .find(|item| item.label == "println")
        .expect("println item must exist");

    assert_eq!(
        println_item.insert_text.as_deref(),
        Some("println"),
        "Terminal symbol insert text must NOT duplicate semicolon when already present"
    );
}

#[test]
fn test_import_completion_workspace_multi_package_discovery() {
    let workspace = Arc::new(WorkspaceManager::new(None, None));
    let math_uri = Url::parse("file:///workspace/math.slx").unwrap();
    let main_uri = Url::parse("file:///workspace/main.slx").unwrap();

    let math_code = r#"
package my_project.math;

public class Calculator {
    public int32 add(int32 left_operand, int32 right_operand) {
        return left_operand + right_operand;
    }
}
"#;
    let _math_snapshot = workspace.did_open(math_uri, 1, math_code.to_string());

    let main_code = "import my_project.";
    let main_snapshot = workspace.did_open(main_uri.clone(), 1, main_code.to_string());

    let completion_params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: main_uri.clone() },
            position: Position {
                line: 0,
                character: 18,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };

    let result = completions(&workspace, &main_snapshot, &completion_params);
    let item_labels: Vec<String> = result.items.iter().map(|item| item.label.clone()).collect();

    assert!(
        item_labels.contains(&"math".to_string()),
        "Child namespace 'math' must be discovered from workspace snapshots: {item_labels:?}"
    );

    // Now test inner symbol completion: 'import my_project.math.'
    let inner_code = "import my_project.math.";
    let inner_snapshot = workspace.did_change(&main_uri, 2, inner_code.to_string());
    let inner_params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: main_uri },
            position: Position {
                line: 0,
                character: 23,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };

    let inner_result = completions(&workspace, &inner_snapshot, &inner_params);
    let inner_labels: Vec<String> = inner_result
        .items
        .iter()
        .map(|item| item.label.clone())
        .collect();

    assert!(
        inner_labels.contains(&"Calculator".to_string()),
        "Class 'Calculator' must be suggested in namespace 'my_project.math': {inner_labels:?}"
    );
}

#[test]
fn test_codelens_clean_labels_and_no_emoji() {
    let workspace = Arc::new(WorkspaceManager::new(None, None));
    let file_uri = Url::parse("file:///workspace/main.slx").unwrap();

    let source_code = r#"
public class Main {
    public static int32 main() {
        return 0;
    }
}
"#;
    let snapshot = workspace.did_open(file_uri, 1, source_code.to_string());
    let active_lenses = code_lenses(&workspace, &snapshot);

    let lens_titles: Vec<String> = active_lenses
        .iter()
        .filter_map(|lens| lens.command.as_ref().map(|command| command.title.clone()))
        .collect();

    for title in &lens_titles {
        assert!(
            !title.contains('🐞'),
            "CodeLens title must not contain emoji: {title}"
        );
    }

    assert!(
        lens_titles.iter().any(|title| title == "▶ Run Main"),
        "Must contain '▶ Run Main': {lens_titles:?}"
    );
    assert!(
        lens_titles.iter().any(|title| title == "Debug Main"),
        "Must contain clean 'Debug Main': {lens_titles:?}"
    );
}
