//! Unit tests for RuntimeOptions and RuntimeContext scaffolding.

#![forbid(unsafe_code)]

use solix_common::bytecode::{BinaryProgram, CompiledFunction};
use solix_runtime::{
    DEFAULT_HEAP_SIZE, DEFAULT_MAX_CALL_STACK_DEPTH, RuntimeContext, RuntimeError, RuntimeOptions,
};
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_default_runtime_options() {
    let options = RuntimeOptions::default();
    assert_eq!(options.entry_point, None);
    assert!(options.binaries.is_empty());
    assert!(options.binary_paths.is_empty());
    assert_eq!(options.heap_size, DEFAULT_HEAP_SIZE);
    assert_eq!(options.max_call_stack_depth, DEFAULT_MAX_CALL_STACK_DEPTH);
    assert!(options.args.is_empty());
    assert!(!options.sandboxed);
}

#[test]
fn test_runtime_options_builder_modifications() {
    let options = RuntimeOptions::new()
        .with_entry_point("custom_entry")
        .with_heap_size(1024 * 1024 * 16)
        .with_max_call_stack_depth(1024)
        .with_args(["--verbose", "--flag"])
        .with_sandboxed(true);

    assert_eq!(options.entry_point.as_deref(), Some("custom_entry"));
    assert_eq!(options.heap_size, 16 * 1024 * 1024);
    assert_eq!(options.max_call_stack_depth, 1024);
    assert_eq!(options.args, vec!["--verbose", "--flag"]);
    assert!(options.sandboxed);
}

#[test]
fn test_load_binary_bytes_corrupted() {
    let invalid_bytes = b"NOT_SLXB_FORMAT";
    let result = RuntimeOptions::new().with_binary_bytes(invalid_bytes);
    assert!(result.is_err());
    match result {
        Err(RuntimeError::BinaryLoadError { reason }) => {
            assert!(
                reason.contains("InvalidMagic"),
                "Expected InvalidMagic in reason, got {reason}"
            );
        }
        _ => panic!("Expected BinaryLoadError"),
    }
}

#[test]
fn test_runtime_context_single_binary_entry_point_id() {
    let mut binary = BinaryProgram::new();
    let func = CompiledFunction::new("entry_func", 0, 0, 4);
    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).expect("Failed to build context");

    assert_eq!(context.entry_point_function_id, Some(0));
    assert_eq!(context.all_functions.len(), 1);
    assert_eq!(
        context.get_function(0).map(|f| f.symbol.as_str()),
        Some("entry_func")
    );
}

#[test]
fn test_runtime_context_entry_point_override() {
    let mut binary = BinaryProgram::new();
    let func1 = CompiledFunction::new("default_entry", 0, 0, 4);
    let func2 = CompiledFunction::new("custom_entry", 0, 0, 4);
    binary.functions.push(func1);
    binary.functions.push(func2);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new()
        .with_binary(binary)
        .with_entry_point("custom_entry");
    let context = RuntimeContext::new(options).expect("Failed to build context");

    assert_eq!(context.entry_point_function_id, Some(1));
}

#[test]
fn test_runtime_context_multiple_binaries_relocation_and_init() {
    // Binary 1: defines 2 static slots, 2 functions (including @init)
    let mut bin1 = BinaryProgram::new();
    bin1.static_space_size = 2;
    bin1.static_symbols_map = vec![("g_count".to_string(), 0), ("g_state".to_string(), 1)];
    let init1 = CompiledFunction::new("pkg1::@init", 0, 0, 2);
    let f1 = CompiledFunction::new("pkg1::run", 0, 0, 4);
    bin1.functions.push(init1);
    bin1.functions.push(f1);

    // Binary 2: defines 3 static slots, 1 function
    let mut bin2 = BinaryProgram::new();
    bin2.static_space_size = 3;
    bin2.static_symbols_map = vec![("pkg2::shared".to_string(), 0)];
    let f2 = CompiledFunction::new("pkg2::compute", 0, 0, 8);
    bin2.functions.push(f2);
    bin2.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(bin1).with_binary(bin2);
    let context = RuntimeContext::new(options).expect("Failed to build context");

    // Total static slots should be 2 + 3 = 5
    assert_eq!(context.total_static_space, 5);

    // Binary 1 static symbols
    assert_eq!(context.resolve_static_slot("g_count"), Some(0));
    assert_eq!(context.resolve_static_slot("g_state"), Some(1));

    // Binary 2 static symbols relocated by bin1.static_space_size (2)
    assert_eq!(context.resolve_static_slot("pkg2::shared"), Some(2));

    // Function relocations
    assert_eq!(context.symbol_to_function_id.get("pkg1::@init"), Some(&0));
    assert_eq!(context.symbol_to_function_id.get("pkg1::run"), Some(&1));
    assert_eq!(context.symbol_to_function_id.get("pkg2::compute"), Some(&2));

    // Function static base offsets relocated
    assert_eq!(context.get_function(0).unwrap().static_base_offset, 0);
    assert_eq!(context.get_function(1).unwrap().static_base_offset, 0);
    assert_eq!(context.get_function(2).unwrap().static_base_offset, 2);

    // Init function detection
    assert_eq!(context.init_function_ids, vec![0]);
}

#[test]
fn test_runtime_context_load_from_file() {
    let mut binary = BinaryProgram::new();
    let func = CompiledFunction::new("main", 0, 0, 4);
    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let bytes = binary.to_bytes();
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    fs::write(temp_file.path(), bytes).expect("Failed to write binary file");

    let options = RuntimeOptions::new().with_binary_path(temp_file.path());
    let context = RuntimeContext::new(options).expect("Failed to build context from file");

    assert_eq!(context.loaded_binaries.len(), 1);
    assert_eq!(context.entry_point_function_id, Some(0));
    assert_eq!(
        context.get_function_by_symbol("main").map(|f| &f.symbol),
        Some(&"main".to_string())
    );
}

#[test]
fn test_runtime_context_missing_file_error() {
    let missing_path = "non_existent_file_path_12345.slxb";
    let options = RuntimeOptions::new().with_binary_path(missing_path);
    let result = RuntimeContext::new(options);

    assert!(result.is_err());
    match result {
        Err(RuntimeError::BinaryLoadError { reason }) => {
            assert!(
                reason.contains("failed to read file"),
                "Expected failed to read file in {reason}"
            );
        }
        _ => panic!("Expected BinaryLoadError"),
    }
}
