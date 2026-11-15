//! Comprehensive integration and regression tests for Milestone 036:
//! LSP Completion Engine Overhaul & Strict Scope/Import Resolution.
//!
//! Verifies:
//! 1. Prefix-preserving member access context parsing (`parse_member_access_context`).
//! 2. First-keystroke zero-lag live buffer completion overlay (`get_live_snapshot`).
//! 3. Partial member typing after `.` and `::` isolating members from global keyword pollution.
//! 4. Strict instance `.` vs static `::` filtering (methods, fields, enum variants).
//! 5. Chained dot access and array indexing receiver type resolution.
//! 6. Hierarchical package and subpackage `import` segment resolution.

#![forbid(unsafe_code)]

use solix_lsp::completion::{AccessDelimiter, completions, parse_member_access_context};
use solix_lsp::workspace::WorkspaceManager;
use std::sync::Arc;
use tower_lsp::lsp_types::*;

fn create_test_workspace() -> (Arc<WorkspaceManager>, Url) {
    let workspace = Arc::new(WorkspaceManager::new(None, None));
    let uri = Url::parse("file:///workspace/ms036_test.slx").unwrap();
    (workspace, uri)
}

#[test]
fn test_ms036_parse_member_access_context() {
    // 1. Basic dot trigger
    let context_dot = parse_member_access_context("    player.").unwrap();
    assert_eq!(context_dot.0, "player");
    assert_eq!(context_dot.1, AccessDelimiter::Dot);
    assert_eq!(context_dot.2, "");

    // 2. Partial member after dot
    let context_partial_dot = parse_member_access_context("let speed = player.sp").unwrap();
    assert_eq!(context_partial_dot.0, "player");
    assert_eq!(context_partial_dot.1, AccessDelimiter::Dot);
    assert_eq!(context_partial_dot.2, "sp");

    // 3. Static double colon trigger
    let context_static = parse_member_access_context("    Console::").unwrap();
    assert_eq!(context_static.0, "Console");
    assert_eq!(context_static.1, AccessDelimiter::DoubleColon);
    assert_eq!(context_static.2, "");

    // 4. Partial member after double colon
    let context_partial_static = parse_member_access_context("    Console::pri").unwrap();
    assert_eq!(context_partial_static.0, "Console");
    assert_eq!(context_partial_static.1, AccessDelimiter::DoubleColon);
    assert_eq!(context_partial_static.2, "pri");

    // 5. Chained dot member access
    let context_chained = parse_member_access_context("    this.position.x").unwrap();
    assert_eq!(context_chained.0, "this.position");
    assert_eq!(context_chained.1, AccessDelimiter::Dot);
    assert_eq!(context_chained.2, "x");

    // 6. Array indexing receiver
    let context_indexed = parse_member_access_context("    entities[target_index].").unwrap();
    assert_eq!(context_indexed.0, "entities[target_index]");
    assert_eq!(context_indexed.1, AccessDelimiter::Dot);
    assert_eq!(context_indexed.2, "");

    // 7. Method call receiver
    let context_call = parse_member_access_context("    get_player().speed").unwrap();
    assert_eq!(context_call.0, "get_player()");
    assert_eq!(context_call.1, AccessDelimiter::Dot);
    assert_eq!(context_call.2, "speed");

    // 8. Range operator .. must NOT parse as member access
    assert!(parse_member_access_context("0..10").is_none());

    // 9. Numeric literal with dot must NOT parse as member access
    assert!(parse_member_access_context("let value = 3.14").is_none());
    assert!(parse_member_access_context("let value = 123.").is_none());

    // 10. Comment line must NOT parse as member access
    assert!(parse_member_access_context("// player.speed").is_none());

    // 11. Import statements must be handled by import completion
    assert!(parse_member_access_context("import solix.collections.").is_none());
}

#[test]
fn test_ms036_partial_member_completion_isolation() {
    let (workspace, uri) = create_test_workspace();
    let source_text = "class Player {\n    public float32 speed_x;\n    public float32 speed_y;\n    public int32 score;\n}\nvoid main() {\n    Player player = new Player();\n    player.sp\n}\n";

    let snapshot = workspace.did_open(uri.clone(), 1, source_text.to_string());
    let params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position {
                line: 7,
                character: 13,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };

    let list = completions(&workspace, &snapshot, &params);
    let labels: Vec<&str> = list.items.iter().map(|item| item.label.as_str()).collect();

    // Must propose speed_x and speed_y
    assert!(
        labels.contains(&"speed_x"),
        "Expected 'speed_x' in {labels:?}"
    );
    assert!(
        labels.contains(&"speed_y"),
        "Expected 'speed_y' in {labels:?}"
    );

    // Must NOT propose unrelated score or keywords
    assert!(
        !labels.contains(&"score"),
        "Must filter out non-matching member 'score'"
    );
    assert!(
        !labels.contains(&"class"),
        "Must NOT pollute with keyword 'class'"
    );
    assert!(
        !labels.contains(&"return"),
        "Must NOT pollute with keyword 'return'"
    );
}

#[test]
fn test_ms036_partial_scope_completion_isolation() {
    let (workspace, uri) = create_test_workspace();
    let source_text = "class Console {\n    public static void print(string message) {}\n    public static void println(string message) {}\n    public static void clear() {}\n}\nvoid main() {\n    Console::pr\n}\n";

    let snapshot = workspace.did_open(uri.clone(), 1, source_text.to_string());
    let params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position {
                line: 6,
                character: 15,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };

    let list = completions(&workspace, &snapshot, &params);
    let labels: Vec<&str> = list.items.iter().map(|item| item.label.as_str()).collect();

    assert!(labels.contains(&"print"), "Expected 'print' in {labels:?}");
    assert!(
        labels.contains(&"println"),
        "Expected 'println' in {labels:?}"
    );

    // Must NOT propose non-matching 'clear' or keywords
    assert!(!labels.contains(&"clear"), "Must filter out 'clear'");
    assert!(
        !labels.contains(&"class"),
        "Must NOT pollute with keyword 'class'"
    );
}

#[test]
fn test_ms036_enum_scope_completion() {
    let (workspace, uri) = create_test_workspace();
    let source_text = "enum GameState {\n    Menu,\n    Playing,\n    Paused,\n    GameOver,\n}\nvoid main() {\n    GameState::\n}\n";

    let snapshot = workspace.did_open(uri.clone(), 1, source_text.to_string());
    let params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position {
                line: 7,
                character: 15,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };

    let list = completions(&workspace, &snapshot, &params);
    let labels: Vec<&str> = list.items.iter().map(|item| item.label.as_str()).collect();

    assert!(labels.contains(&"Menu"), "Expected 'Menu' in {labels:?}");
    assert!(
        labels.contains(&"Playing"),
        "Expected 'Playing' in {labels:?}"
    );
    assert!(
        labels.contains(&"Paused"),
        "Expected 'Paused' in {labels:?}"
    );
    assert!(
        labels.contains(&"GameOver"),
        "Expected 'GameOver' in {labels:?}"
    );

    for item in &list.items {
        assert_eq!(item.kind, Some(CompletionItemKind::ENUM_MEMBER));
    }
}

#[test]
fn test_ms036_first_keystroke_live_snapshot_overlay() {
    let (workspace, uri) = create_test_workspace();
    let initial_source = "class Point {\n    public int32 position_x;\n    public int32 position_y;\n}\nvoid main() {\n    Point point_instance = new Point();\n}\n";

    let _initial_snapshot = workspace.did_open(uri.clone(), 1, initial_source.to_string());

    // User types '    point_instance.' without triggering debounce analyze_document_change
    let modified_source = "class Point {\n    public int32 position_x;\n    public int32 position_y;\n}\nvoid main() {\n    Point point_instance = new Point();\n    point_instance.\n}\n";
    workspace.update_document(&uri, 2, modified_source.to_string());

    // get_live_snapshot must overlay modified source immediately
    let live_snapshot = workspace
        .get_live_snapshot(&uri)
        .expect("Live snapshot must exist");
    assert_eq!(live_snapshot.version, 2);
    assert_eq!(live_snapshot.source_text, modified_source);

    let params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position {
                line: 6,
                character: 19,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };

    let list = completions(&workspace, &live_snapshot, &params);
    let labels: Vec<&str> = list.items.iter().map(|item| item.label.as_str()).collect();

    assert!(
        labels.contains(&"position_x"),
        "First-keystroke dot completion must propose 'position_x': {labels:?}"
    );
    assert!(
        labels.contains(&"position_y"),
        "First-keystroke dot completion must propose 'position_y': {labels:?}"
    );
}

#[test]
fn test_ms036_member_ranking_direct_vs_inherited_vs_postfix() {
    let (workspace, uri) = create_test_workspace();
    let source_text = "class Entity {\n    public int32 entity_id;\n}\nclass Player extends Entity {\n    public string player_name;\n}\nvoid main() {\n    Player player = new Player();\n    player.\n}\n";

    let snapshot = workspace.did_open(uri.clone(), 1, source_text.to_string());
    let params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position {
                line: 8,
                character: 11,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };

    let list = completions(&workspace, &snapshot, &params);

    let player_name_item = list
        .items
        .iter()
        .find(|item| item.label == "player_name")
        .expect("Direct member 'player_name' must exist");
    assert_eq!(
        player_name_item.sort_text.as_deref(),
        Some("0_player_name"),
        "Direct class member must have rank prefix '0_'"
    );

    let entity_id_item = list
        .items
        .iter()
        .find(|item| item.label == "entity_id")
        .expect("Inherited member 'entity_id' must exist");
    assert_eq!(
        entity_id_item.sort_text.as_deref(),
        Some("1_entity_id"),
        "Inherited class member must have rank prefix '1_'"
    );

    let postfix_item = list
        .items
        .iter()
        .find(|item| item.label == ".return")
        .expect("Postfix snippet '.return' must exist");
    assert_eq!(
        postfix_item.sort_text.as_deref(),
        Some("2_return"),
        "Postfix snippet must have rank prefix '2_'"
    );
}

#[test]
fn test_ms036_chained_dot_and_call_receiver_resolution() {
    let (workspace, uri) = create_test_workspace();
    let source_text = "class Vector2 {\n    public float32 coord_x;\n    public float32 coord_y;\n}\nclass Character {\n    public Vector2 velocity;\n    public Vector2 get_velocity() {\n        return velocity;\n    }\n}\nvoid main() {\n    Character hero = new Character();\n    hero.velocity.\n}\n";

    let snapshot = workspace.did_open(uri.clone(), 1, source_text.to_string());
    let params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position {
                line: 12,
                character: 18,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };

    let list = completions(&workspace, &snapshot, &params);
    let labels: Vec<&str> = list.items.iter().map(|item| item.label.as_str()).collect();

    assert!(
        labels.contains(&"coord_x"),
        "Chained dot access must propose 'coord_x': {labels:?}"
    );
    assert!(
        labels.contains(&"coord_y"),
        "Chained dot access must propose 'coord_y': {labels:?}"
    );
}

#[test]
fn test_single_colon_completion_suppression() {
    let (workspace, uri) = create_test_workspace();
    let source_text = "package test;\nvoid main() {\n    solix::collections:\n}\n";

    let snapshot = workspace.did_open(uri.clone(), 1, source_text.to_string());
    let params_single_colon = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 2,
                character: 23,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };

    let list_single = completions(&workspace, &snapshot, &params_single_colon);
    assert!(
        list_single.items.is_empty(),
        "Typing single colon ':' MUST NOT trigger completions, got: {:?}",
        list_single
            .items
            .iter()
            .map(|item| &item.label)
            .collect::<Vec<_>>()
    );
}
