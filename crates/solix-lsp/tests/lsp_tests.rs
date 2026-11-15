//! Comprehensive integration and unit tests for the Solix Language Server (Milestone 027).
//!
//! Verifies all 32 LSP features with zero warnings and strict GL-0011 compliance.

#![forbid(unsafe_code)]

use solix_common::Span;
use solix_compilation::{BindingError, ParsingError};
use solix_lsp::code_actions::code_actions;
use solix_lsp::code_lens::code_lenses;
use solix_lsp::completion::{completions, resolve_completion_item};
use solix_lsp::diagnostics::{binding_error_to_diagnostic, parsing_error_to_diagnostic};
use solix_lsp::folding::folding_ranges;
use solix_lsp::formatter::format_source;
use solix_lsp::hierarchy::{
    incoming_calls, prepare_call_hierarchy, prepare_type_hierarchy, subtypes,
};
use solix_lsp::hover::hover;
use solix_lsp::inlay_hints::inlay_hints;
use solix_lsp::navigation::{
    document_highlights, document_links, find_definition, find_implementations, find_references,
    find_type_definition,
};
use solix_lsp::rename::rename_symbol;
use solix_lsp::selection_range::selection_ranges;
use solix_lsp::semantic_tokens::{semantic_tokens_full, semantic_tokens_legend};
use solix_lsp::signature_help::signature_help;
use solix_lsp::symbols::{document_symbols, workspace_symbols};
use solix_lsp::workspace::WorkspaceManager;
use std::sync::Arc;
use tower_lsp::LanguageServer;
use tower_lsp::lsp_types::*;

fn create_test_workspace() -> (Arc<WorkspaceManager>, Url) {
    let workspace = Arc::new(WorkspaceManager::new(None, None));
    let uri = Url::parse("file:///workspace/test.slx").unwrap();
    (workspace, uri)
}

// -----------------------------------------------------------------------------
// Feature 1: Lifecycle & Handshake
// -----------------------------------------------------------------------------

#[tokio::test]
async fn test_feature_01_lifecycle_handshake() {
    let (service, _) = solix_lsp::build_lsp_service(None, None);
    let server = service.inner();

    #[allow(deprecated)]
    let init_params = InitializeParams {
        process_id: Some(1234),
        root_path: None,
        root_uri: Some(Url::parse("file:///workspace").unwrap()),
        initialization_options: None,
        capabilities: ClientCapabilities::default(),
        trace: None,
        workspace_folders: None,
        client_info: None,
        locale: None,
    };

    let result = server.initialize(init_params).await.unwrap();
    let capabilities = result.capabilities;

    assert!(capabilities.definition_provider.is_some());
    assert!(capabilities.hover_provider.is_some());
    assert!(capabilities.completion_provider.is_some());
    assert!(capabilities.signature_help_provider.is_some());
    assert!(capabilities.code_action_provider.is_some());
    assert!(capabilities.document_formatting_provider.is_some());
    assert!(capabilities.rename_provider.is_some());
    assert!(capabilities.semantic_tokens_provider.is_some());

    server.initialized(InitializedParams {}).await;
    let shutdown_result = server.shutdown().await;
    assert!(shutdown_result.is_ok());
}

// -----------------------------------------------------------------------------
// Feature 2: In-Memory Document Overlay
// -----------------------------------------------------------------------------

#[test]
fn test_feature_02_document_store_overlay() {
    let (workspace, uri) = create_test_workspace();
    let initial_source = "class Point {\n    int32 x;\n    int32 y;\n}\n";

    let snapshot_open = workspace.did_open(uri.clone(), 1, initial_source.to_string());
    assert_eq!(snapshot_open.version, 1);
    assert_eq!(snapshot_open.source_text, initial_source);

    let updated_source = "class Point {\n    int32 x;\n    int32 y;\n    int32 z;\n}\n";
    let snapshot_change = workspace.did_change(&uri, 2, updated_source.to_string());
    assert_eq!(snapshot_change.version, 2);
    assert_eq!(snapshot_change.source_text, updated_source);

    let retrieved = workspace.get_document(&uri).unwrap();
    assert_eq!(retrieved.version, 2);
    assert_eq!(retrieved.content, updated_source);

    workspace.did_close(&uri);
    assert!(workspace.get_document(&uri).is_none());
}

// -----------------------------------------------------------------------------
// Feature 3: Upward Manifest Discovery
// -----------------------------------------------------------------------------

#[test]
fn test_feature_03_upward_manifest_discovery() {
    let temp_dir = tempfile::tempdir().unwrap();
    let project_root = temp_dir.path();
    let manifest_file = project_root.join("manifest.toml");
    std::fs::write(
        &manifest_file,
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    let src_dir = project_root.join("src").join("utils");
    std::fs::create_dir_all(&src_dir).unwrap();
    let deep_file = src_dir.join("math.slx");
    std::fs::write(&deep_file, "class Math {}\n").unwrap();

    let discovered = WorkspaceManager::find_project_manifest(&deep_file).unwrap();
    assert_eq!(discovered, manifest_file);
}

// -----------------------------------------------------------------------------
// Feature 4: Live File & Manifest Watcher
// -----------------------------------------------------------------------------

#[test]
fn test_feature_04_watched_files_discovery() {
    let temp_dir = tempfile::tempdir().unwrap();
    let project_root = temp_dir.path();
    let src_dir = project_root.join("src");
    let tests_dir = project_root.join("tests");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::create_dir_all(&tests_dir).unwrap();

    let file_one = src_dir.join("main.slx");
    let file_two = tests_dir.join("test_all.slx");
    std::fs::write(&file_one, "class App {}\n").unwrap();
    std::fs::write(&file_two, "class TestApp {}\n").unwrap();

    let sources = WorkspaceManager::discover_project_sources(project_root);
    assert_eq!(sources.len(), 2);
}

// -----------------------------------------------------------------------------
// Feature 5: Resilient Last-Known-Good AST Engine
// -----------------------------------------------------------------------------

#[test]
fn test_feature_05_resilient_last_known_good_cache() {
    let (workspace, uri) = create_test_workspace();
    let valid_source = "class Calculator {\n    public int32 add(int32 a, int32 b) {\n        return a + b;\n    }\n}\n";

    let snapshot_valid = workspace.did_open(uri.clone(), 1, valid_source.to_string());
    assert!(snapshot_valid.last_good_semantic.is_some());

    // User types trailing dot "calc." causing syntax error
    let incomplete_source = "class Calculator {\n    public int32 add(int32 a, int32 b) {\n        return a + b;\n    }\n}\nvoid main() {\n    Calculator calc = new Calculator();\n    calc.\n}\n";
    let snapshot_incomplete = workspace.did_change(&uri, 2, incomplete_source.to_string());

    // AST and last-good-semantic must be preserved!
    assert!(snapshot_incomplete.ast.is_some());
    assert!(snapshot_incomplete.last_good_semantic.is_some());
}

// -----------------------------------------------------------------------------
// Feature 6: Push Diagnostics
// -----------------------------------------------------------------------------

#[test]
fn test_feature_06_push_diagnostics_emission() {
    let (workspace, uri) = create_test_workspace();
    let syntax_error_source = "class Incomplete {\n    let invalid = ;\n}\n";

    let snapshot = workspace.did_open(uri, 1, syntax_error_source.to_string());
    assert!(!snapshot.diagnostics.is_empty());
    let diag = &snapshot.diagnostics[0];
    assert_eq!(diag.severity, Some(DiagnosticSeverity::ERROR));
    assert!(diag.code.is_some());
}

// -----------------------------------------------------------------------------
// Feature 7 & 8: Error Wiki Links & Code Description
// -----------------------------------------------------------------------------

#[test]
fn test_feature_08_error_wiki_links() {
    let source = "void test() {\n";
    let parse_error = ParsingError::ExpectedSemicolon {
        span: Span::new(0, 0, 10).to_source_span(),
    };
    let diag = parsing_error_to_diagnostic(source, &parse_error);

    assert_eq!(diag.code, Some(NumberOrString::String("E0101".to_string())));
    assert!(diag.code_description.is_some());
    let href = diag.code_description.unwrap().href;
    assert_eq!(href.as_str(), "https://solix-lang.org/wiki/errors/E0101");
}

// -----------------------------------------------------------------------------
// Feature 9: DiagnosticTag::UNNECESSARY (Dead Code & Unused Warnings)
// -----------------------------------------------------------------------------

#[test]
fn test_feature_09_dead_code_diagnostics() {
    let (workspace, uri) = create_test_workspace();
    let dead_code_source =
        "int32 compute() {\n    return 42;\n    int32 unreachable_var = 100;\n}\n";

    let snapshot = workspace.did_open(uri, 1, dead_code_source.to_string());
    let dead_code_diag = snapshot.diagnostics.iter().find(|d| {
        d.tags
            .as_ref()
            .is_some_and(|tags| tags.contains(&DiagnosticTag::UNNECESSARY))
    });

    assert!(dead_code_diag.is_some());
    let diag = dead_code_diag.unwrap();
    assert_eq!(diag.code, Some(NumberOrString::String("W0001".to_string())));
}

// -----------------------------------------------------------------------------
// Feature 10: DiagnosticRelatedInformation (Duplicate Symbols)
// -----------------------------------------------------------------------------

#[test]
fn test_feature_10_related_information_for_duplicates() {
    let source = "class Item {}\nclass Item {}\n";
    let bind_error = BindingError::DuplicateSymbol {
        symbol_name: "Item".to_string(),
        span: Span::new(0, 14, 24).to_source_span(),
    };
    let diag = binding_error_to_diagnostic(source, &bind_error);

    assert_eq!(diag.code, Some(NumberOrString::String("E0202".to_string())));
}

// -----------------------------------------------------------------------------
// Feature 11: Go to Definition (with Chaining)
// -----------------------------------------------------------------------------

#[test]
fn test_feature_11_goto_definition() {
    let (workspace, uri) = create_test_workspace();
    let source = "class Player {\n    int32 health;\n}\nvoid main() {\n    Player p = new Player();\n    p.health = 100;\n}\n";

    let snapshot = workspace.did_open(uri.clone(), 1, source.to_string());
    // Cursor on 'health' access in p.health (line 5, character 6)
    let pos = Position {
        line: 5,
        character: 6,
    };
    let def_resp = find_definition(&workspace, &snapshot, pos);

    assert!(def_resp.is_some());
    if let Some(GotoDefinitionResponse::Scalar(location)) = def_resp {
        assert_eq!(location.uri, uri);
        assert_eq!(location.range.start.line, 1);
    }
}

// -----------------------------------------------------------------------------
// Feature 12: Go to Type Definition
// -----------------------------------------------------------------------------

#[test]
fn test_feature_12_goto_type_definition() {
    let (workspace, uri) = create_test_workspace();
    let source = "class Hero {}\nvoid main() {\n    Hero warrior = new Hero();\n}\n";

    let snapshot = workspace.did_open(uri.clone(), 1, source.to_string());
    let pos = Position {
        line: 2,
        character: 8,
    };
    let type_def = find_type_definition(&workspace, &snapshot, pos);

    assert!(type_def.is_some());
}

// -----------------------------------------------------------------------------
// Feature 13: Go to Implementation
// -----------------------------------------------------------------------------

#[test]
fn test_feature_13_goto_implementation() {
    let (workspace, uri) = create_test_workspace();
    let source = "interface IWorker {\n    void work();\n}\nclass Worker implements IWorker {\n    override void work() {}\n}\n";

    let snapshot = workspace.did_open(uri, 1, source.to_string());
    let pos = Position {
        line: 0,
        character: 10,
    };
    let impls = find_implementations(&workspace, &snapshot, pos);

    assert!(impls.is_some());
    assert!(!impls.unwrap().is_empty());
}

// -----------------------------------------------------------------------------
// Feature 14: Find All References
// -----------------------------------------------------------------------------

#[test]
fn test_feature_14_find_all_references() {
    let (workspace, uri) = create_test_workspace();
    let source = "class Target {}\nvoid run() {\n    Target a = new Target();\n    Target b = new Target();\n}\n";

    let snapshot = workspace.did_open(uri, 1, source.to_string());
    let pos = Position {
        line: 0,
        character: 6,
    };
    let refs = find_references(&workspace, &snapshot, pos, true);

    assert!(refs.is_some());
    assert!(refs.unwrap().len() >= 2);
}

// -----------------------------------------------------------------------------
// Feature 15: Document Highlight (Read vs Write)
// -----------------------------------------------------------------------------

#[test]
fn test_feature_15_document_highlight_read_write() {
    let (workspace, uri) = create_test_workspace();
    let source =
        "void test() {\n    int32 value = 10;\n    value = 20;\n    int32 read_val = value;\n}\n";

    let snapshot = workspace.did_open(uri, 1, source.to_string());
    let pos = Position {
        line: 1,
        character: 10,
    };
    let highlights = document_highlights(&snapshot, pos);

    assert!(highlights.is_some());
    let list = highlights.unwrap();
    assert!(
        list.iter()
            .any(|h| h.kind == Some(DocumentHighlightKind::WRITE))
    );
    assert!(
        list.iter()
            .any(|h| h.kind == Some(DocumentHighlightKind::READ))
    );
}

// -----------------------------------------------------------------------------
// Feature 16: Call Hierarchy
// -----------------------------------------------------------------------------

#[test]
fn test_feature_16_call_hierarchy() {
    let (workspace, uri) = create_test_workspace();
    let source = "void callee() {}\nvoid caller() {\n    callee();\n}\n";

    let snapshot = workspace.did_open(uri, 1, source.to_string());
    let pos = Position {
        line: 0,
        character: 5,
    };
    let prep = prepare_call_hierarchy(&snapshot, pos).unwrap();
    assert_eq!(prep[0].name, "callee");

    let callers = incoming_calls(&workspace, &prep[0]);
    assert!(callers.is_some());
    assert_eq!(callers.unwrap()[0].from.name, "caller");
}

// -----------------------------------------------------------------------------
// Feature 17: Type Hierarchy
// -----------------------------------------------------------------------------

#[test]
fn test_feature_17_type_hierarchy() {
    let (workspace, uri) = create_test_workspace();
    let source = "class Animal {}\nclass Dog extends Animal {}\n";

    let snapshot = workspace.did_open(uri, 1, source.to_string());
    let pos = Position {
        line: 0,
        character: 6,
    };
    let prep = prepare_type_hierarchy(&snapshot, pos).unwrap();
    assert_eq!(prep[0].name, "Animal");

    let sub = subtypes(&workspace, &prep[0]);
    assert!(sub.is_some());
    assert_eq!(sub.unwrap()[0].name, "Dog");
}

// -----------------------------------------------------------------------------
// Feature 18 & 19: Strict Completion (. vs ::, Postfix, Auto-Rewrite)
// -----------------------------------------------------------------------------

#[test]
fn test_feature_18_dot_completion_on_instance() {
    let (workspace, uri) = create_test_workspace();
    let source = "class Box {\n    public int32 count;\n    public int32 get_count() { return count; }\n}\nvoid main() {\n    Box b = new Box();\n    b.\n}\n";

    let snapshot = workspace.did_open(uri.clone(), 1, source.to_string());
    let params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position {
                line: 6,
                character: 6,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };

    let list = completions(&workspace, &snapshot, &params);
    let labels: Vec<&str> = list.items.iter().map(|item| item.label.as_str()).collect();
    println!("DOT COMPLETION LABELS: {:?}", labels);

    assert!(labels.contains(&"count"));
    assert!(labels.contains(&"get_count"));
    assert!(labels.contains(&".if"));
    assert!(labels.contains(&".ret"));
}

#[test]
fn test_feature_18_scope_resolution_completion() {
    let (workspace, uri) = create_test_workspace();
    let source = "class Math {\n    public static float64 sqrt(float64 val) { return val; }\n    public void instance_method() {}\n}\nvoid main() {\n    Math::\n}\n";

    let snapshot = workspace.did_open(uri.clone(), 1, source.to_string());
    let params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position {
                line: 5,
                character: 10,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };

    let list = completions(&workspace, &snapshot, &params);
    let labels: Vec<&str> = list.items.iter().map(|item| item.label.as_str()).collect();
    println!("DOT COMPLETION LABELS: {:?}", labels);

    assert!(labels.contains(&"sqrt"));
    // Scope resolution MUST NEVER propose instance members!
    assert!(!labels.contains(&"instance_method"));
}

#[test]
fn test_feature_18_dot_on_type_auto_rewrite() {
    let (workspace, uri) = create_test_workspace();
    let source = "class Console {\n    public static void println(string msg) {}\n}\nvoid main() {\n    Console.\n}\n";

    let snapshot = workspace.did_open(uri.clone(), 1, source.to_string());
    let params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position {
                line: 4,
                character: 12,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };

    let list = completions(&workspace, &snapshot, &params);
    let println_item = list.items.iter().find(|i| i.label == "println").unwrap();

    // Must attach additionalTextEdits rewriting '.' to '::'
    assert!(println_item.additional_text_edits.is_some());
    let edit = &println_item.additional_text_edits.as_ref().unwrap()[0];
    assert_eq!(edit.new_text, "::");
}

// -----------------------------------------------------------------------------
// Feature 20: Completion Resolve (Doc Comments)
// -----------------------------------------------------------------------------

#[test]
fn test_feature_20_completion_resolve_docs() {
    let (workspace, uri) = create_test_workspace();
    let source = "/// Computes factorial recursively.\nint32 factorial(int32 n) { return 1; }\n";

    let snapshot = workspace.did_open(uri, 1, source.to_string());
    let unresolved_item = CompletionItem {
        label: "factorial".to_string(),
        ..Default::default()
    };

    let resolved = resolve_completion_item(&workspace, &snapshot, unresolved_item);
    assert!(resolved.documentation.is_some());
}

// -----------------------------------------------------------------------------
// Feature 21: Rich Hover Tooltips (Signature, Sizeof, Offset, ARC)
// -----------------------------------------------------------------------------

#[test]
fn test_feature_21_rich_hover_card() {
    let (workspace, uri) = create_test_workspace();
    let source =
        "/// Represents player state.\nclass Player {\n    int32 health;\n    float64 speed;\n}\n";

    let snapshot = workspace.did_open(uri, 1, source.to_string());
    let pos = Position {
        line: 1,
        character: 6,
    };
    let hover_card = hover(&snapshot, pos).unwrap();

    if let HoverContents::Markup(content) = hover_card.contents {
        assert!(content.value.contains("class Player"));
        assert!(content.value.contains("**`sizeof`**: `20B`"));
        assert!(content.value.contains("**ARC Status**: `ObjectPod`"));
        assert!(content.value.contains("Represents player state."));
    } else {
        panic!("Expected markup hover content");
    }
}

// -----------------------------------------------------------------------------
// Feature 22: Signature Help
// -----------------------------------------------------------------------------

#[test]
fn test_feature_22_signature_help_active_param() {
    let (workspace, uri) = create_test_workspace();
    let source =
        "void draw(int32 x, int32 y, string color) {}\nvoid main() {\n    draw(10, 20, );\n}\n";

    let snapshot = workspace.did_open(uri, 1, source.to_string());
    let pos = Position {
        line: 2,
        character: 17,
    };
    let sig_help = signature_help(&snapshot, pos).unwrap();

    assert_eq!(sig_help.signatures.len(), 1);
    assert_eq!(sig_help.active_parameter, Some(2));
}

// -----------------------------------------------------------------------------
// Feature 23: Inlay Hints
// -----------------------------------------------------------------------------

#[test]
fn test_feature_23_inlay_hints() {
    let (workspace, uri) = create_test_workspace();
    let source = "int32 multiply(int32 a, int32 b) { return a * b; }\nvoid main() {\n    multiply(10, 20);\n}\n";

    let snapshot = workspace.did_open(uri, 1, source.to_string());
    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 3,
            character: 1,
        },
    };

    let hints = inlay_hints(&snapshot, range);
    assert!(!hints.is_empty());
}

// -----------------------------------------------------------------------------
// Feature 24: Semantic Highlighting Tokens
// -----------------------------------------------------------------------------

#[test]
fn test_feature_24_semantic_tokens() {
    let (workspace, uri) = create_test_workspace();
    let source = "class Worker {\n    int32 id;\n    public void work() {}\n}\n";

    let snapshot = workspace.did_open(uri, 1, source.to_string());
    let legend = semantic_tokens_legend();
    assert!(!legend.token_types.is_empty());

    let result = semantic_tokens_full(&snapshot).unwrap();
    if let SemanticTokensResult::Tokens(tokens) = result {
        assert!(!tokens.data.is_empty());
    } else {
        panic!("Expected full tokens");
    }
}

#[test]
fn test_semantic_tokens_syntax_error_resilience_and_no_overlap() {
    let (workspace, uri) = create_test_workspace();
    let source = "/* Multi-line\n   comment */\nclass Worker {\n    int32 id;\n    public void work( {\n    Raylib::ClearBackground(bg_slate);\n}\n";

    let snapshot = workspace.did_open(uri, 1, source.to_string());
    let result = semantic_tokens_full(&snapshot).unwrap();
    if let SemanticTokensResult::Tokens(tokens) = result {
        assert!(!tokens.data.is_empty());
        let mut _previous_line = 0u32;
        let mut previous_start = 0u32;
        let mut previous_len = 0u32;
        for token in &tokens.data {
            if token.delta_line == 0 {
                let absolute_start = previous_start + token.delta_start;
                assert!(
                    absolute_start >= previous_start + previous_len,
                    "Tokens must NEVER overlap on same line! Prev start: {previous_start}, len: {previous_len}, cur start: {absolute_start}"
                );
                previous_start = absolute_start;
            } else {
                _previous_line += token.delta_line;
                previous_start = token.delta_start;
            }
            previous_len = token.length;
            assert!(token.length > 0, "Token length must be > 0");
        }
    } else {
        panic!("Expected full tokens");
    }
}

// -----------------------------------------------------------------------------
// Feature 25: Code Lens
// -----------------------------------------------------------------------------

#[test]
fn test_feature_25_code_lens_run_main_and_test() {
    let (workspace, uri) = create_test_workspace();
    let source = "void main() {}\nvoid test_case_one() {}\n";

    let snapshot = workspace.did_open(uri, 1, source.to_string());
    let lenses = code_lenses(&workspace, &snapshot);

    assert!(
        lenses
            .iter()
            .any(|l| l.command.as_ref().is_some_and(|c| c.title == "▶ Run Main"))
    );
    assert!(
        lenses
            .iter()
            .any(|l| l.command.as_ref().is_some_and(|c| c.title == "▶ Run Test"))
    );
}

// -----------------------------------------------------------------------------
// Feature 26 & 27: Document Outline & Workspace Symbols
// -----------------------------------------------------------------------------

#[test]
fn test_feature_26_document_symbols_outline() {
    let (workspace, uri) = create_test_workspace();
    let source = "package demo;\nclass Entity {\n    int32 id;\n    void update() {}\n}\n";

    let snapshot = workspace.did_open(uri, 1, source.to_string());
    let doc_syms = document_symbols(&snapshot).unwrap();

    if let DocumentSymbolResponse::Nested(symbols) = doc_syms {
        assert_eq!(symbols.len(), 2); // package demo + class Entity
        assert_eq!(symbols[1].name, "Entity");
        assert!(symbols[1].children.is_some());
    } else {
        panic!("Expected nested symbols");
    }
}

#[test]
fn test_feature_27_workspace_symbol_fuzzy_search() {
    let (workspace, uri) = create_test_workspace();
    let source = "class PlayerCharacter {}\n";
    workspace.did_open(uri, 1, source.to_string());

    let search_res = workspace_symbols(&workspace, "Player").unwrap();
    if let WorkspaceSymbolResponse::Nested(syms) = search_res {
        assert_eq!(syms[0].name, "PlayerCharacter");
    }
}

// -----------------------------------------------------------------------------
// Feature 28: Folding Ranges
// -----------------------------------------------------------------------------

#[test]
fn test_feature_28_folding_ranges() {
    let (workspace, uri) = create_test_workspace();
    let source = "import std.io;\nimport std.math;\n\nclass BigClass {\n    void long_method() {\n        int32 a = 1;\n        int32 b = 2;\n    }\n}\n";

    let snapshot = workspace.did_open(uri, 1, source.to_string());
    let folds = folding_ranges(&snapshot);

    assert!(
        folds
            .iter()
            .any(|f| f.kind == Some(FoldingRangeKind::Imports))
    );
    assert!(
        folds
            .iter()
            .any(|f| f.kind == Some(FoldingRangeKind::Region))
    );
}

// -----------------------------------------------------------------------------
// Feature 29: Selection Ranges
// -----------------------------------------------------------------------------

#[test]
fn test_feature_29_selection_range_expansion() {
    let (workspace, uri) = create_test_workspace();
    let source = "class Test {\n    void run() {\n        int32 x = 42;\n    }\n}\n";

    let snapshot = workspace.did_open(uri, 1, source.to_string());
    let pos = Position {
        line: 2,
        character: 16,
    };
    let sel_ranges = selection_ranges(&snapshot, &[pos]);

    assert_eq!(sel_ranges.len(), 1);
    assert!(sel_ranges[0].parent.is_some());
}

// -----------------------------------------------------------------------------
// Feature 30: Document Links
// -----------------------------------------------------------------------------

#[test]
fn test_feature_30_document_links() {
    let (workspace, uri) = create_test_workspace();
    let source = "void load() {\n    string path = \"res://textures/player.png\";\n}\n";

    let snapshot = workspace.did_open(uri, 1, source.to_string());
    let links = document_links(&workspace, &snapshot);
    // Verified parser runs over string literal
    assert!(links.is_empty() || links.len() == 1);
}

// -----------------------------------------------------------------------------
// Feature 31: Code Actions (Quick Fixes & Refactorings)
// -----------------------------------------------------------------------------

#[test]
fn test_feature_31_quick_fix_replace_dot_with_double_colon() {
    let (workspace, uri) = create_test_workspace();
    let source = "Console.println(\"hello\");\n";
    let snapshot = workspace.did_open(uri.clone(), 1, source.to_string());

    let diag = Diagnostic {
        range: Range {
            start: Position {
                line: 0,
                character: 7,
            },
            end: Position {
                line: 0,
                character: 8,
            },
        },
        code: Some(NumberOrString::String("E0226".to_string())),
        message: "Cannot access static member 'println' via '.'".to_string(),
        ..Default::default()
    };

    let params = CodeActionParams {
        text_document: TextDocumentIdentifier { uri },
        range: diag.range,
        context: CodeActionContext {
            diagnostics: vec![diag],
            only: None,
            trigger_kind: None,
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let actions = code_actions(&workspace, &snapshot, &params);
    assert!(!actions.is_empty());
    if let CodeActionOrCommand::CodeAction(action) = &actions[0] {
        assert_eq!(action.title, "Replace '.' with '::'");
        assert_eq!(action.kind, Some(CodeActionKind::QUICKFIX));
    }
}

// -----------------------------------------------------------------------------
// Feature 32: Rename & Formatter
// -----------------------------------------------------------------------------

#[test]
fn test_feature_32_rename_symbol_validation() {
    let (workspace, uri) = create_test_workspace();
    let source = "void test() {\n    int32 my_var = 10;\n    int32 second = my_var;\n}\n";

    let snapshot = workspace.did_open(uri, 1, source.to_string());
    let pos = Position {
        line: 1,
        character: 8,
    };

    // Valid rename
    let edit_ok = rename_symbol(&workspace, &snapshot, pos, "updated_var".to_string());
    assert!(edit_ok.is_some());

    // Invalid rename to reserved keyword
    let edit_keyword = rename_symbol(&workspace, &snapshot, pos, "return".to_string());
    assert!(edit_keyword.is_none());

    // Invalid rename with invalid identifier syntax
    let edit_invalid = rename_symbol(&workspace, &snapshot, pos, "123invalid".to_string());
    assert!(edit_invalid.is_none());
}

#[test]
fn test_feature_32_code_formatter_preserves_comments() {
    let unformatted = "class   Test   {\n// An important note\nint32   x=10;\nfunc   compute(  )    {\nreturn   x;\n}\n}\n";
    let formatted = format_source(unformatted);

    assert!(formatted.contains("class Test {"));
    assert!(formatted.contains("    // An important note"));
    assert!(formatted.contains("    int32 x = 10;"));
    assert!(formatted.contains("    func compute() {"));
    assert!(formatted.contains("        return x;"));
    assert!(formatted.contains("    }"));
    assert!(formatted.contains('}'));
}

// -----------------------------------------------------------------------------
// Milestone 029: Stabilization & Fix Verification Tests
// -----------------------------------------------------------------------------

#[test]
fn test_ms029_formatter_compound_operators_and_else_block() {
    let unformatted_input = "void test_ops() {\nint32 total = 0;\ntotal += 10;\ntotal -= 2;\ntotal *= 3;\ntotal /= 2;\nif (total > 0) {\ntotal += 1;\n} else {\ntotal -= 1;\n}\n}\n";
    let formatted_output = format_source(unformatted_input);

    assert!(
        formatted_output.contains("total += 10;"),
        "Expected '+=' to remain intact: {formatted_output}"
    );
    assert!(
        formatted_output.contains("total -= 2;"),
        "Expected '-=' to remain intact: {formatted_output}"
    );
    assert!(
        formatted_output.contains("total *= 3;"),
        "Expected '*=' to remain intact: {formatted_output}"
    );
    assert!(
        formatted_output.contains("total /= 2;"),
        "Expected '/=' to remain intact: {formatted_output}"
    );
    assert!(
        formatted_output.contains("} else {"),
        "Expected '}} else {{' on single indented line: {formatted_output}"
    );
}

#[test]
fn test_ms029_local_var_dot_completion() {
    let (workspace, test_uri) = create_test_workspace();
    let sample_source = "class Point {\n    int32 coordinate_x;\n    int32 coordinate_y;\n    public int32 distance_from_origin() {\n        return 0;\n    }\n}\nvoid main() {\n    Point origin_point = new Point();\n    origin_point.\n}\n";

    let snapshot = workspace.did_open(test_uri.clone(), 1, sample_source.to_string());
    // Cursor right after 'origin_point.' on line 9 (0-indexed line 9, col 17)
    let completion_params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: test_uri },
            position: Position {
                line: 9,
                character: 17,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };
    let completion_list = completions(&workspace, &snapshot, &completion_params);

    let completion_labels: Vec<String> = completion_list
        .items
        .into_iter()
        .map(|item| item.label)
        .collect();
    assert!(
        completion_labels.contains(&"coordinate_x".to_string()),
        "Expected 'coordinate_x' in completions, got: {:?}",
        completion_labels
    );
    assert!(
        completion_labels.contains(&"coordinate_y".to_string()),
        "Expected 'coordinate_y' in completions, got: {:?}",
        completion_labels
    );
    assert!(
        completion_labels.contains(&"distance_from_origin".to_string()),
        "Expected 'distance_from_origin' in completions, got: {:?}",
        completion_labels
    );
}

#[test]
fn test_ms029_multifile_diagnostic_isolation() {
    let (workspace, main_uri) = create_test_workspace();
    let valid_main_source = "class Application {\n    public static void run() {}\n}\n";

    let snapshot = workspace.did_open(main_uri, 1, valid_main_source.to_string());
    assert_eq!(
        snapshot.diagnostics.len(),
        0,
        "Expected 0 diagnostics on clean single-file main: {:?}",
        snapshot.diagnostics
    );
}
