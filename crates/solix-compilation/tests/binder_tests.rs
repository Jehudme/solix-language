//! Comprehensive semantic analysis and type binding integration tests.

#![forbid(unsafe_code)]

use solix_common::{BindingError, PrimitiveKind, Span, Type};
use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
use solix_compilation::phases::binder::bind;
use solix_compilation::phases::binder::symbols::{SymbolEntry, SymbolId};
use solix_compilation::phases::lexing::lex;
use solix_compilation::phases::parser::parse;
use solix_compilation::phases::parser::statements::declarations::Modifier;
use std::fs;
use std::path::Path;

#[test]
fn test_positive_multi_file_imports_and_generics_binding() {
    let lib_path = Path::new("tests/fixtures/semantics/positive/declarations_library.slx");
    let consumer_path = Path::new("tests/fixtures/semantics/positive/consumer_program.slx");

    let lib_code = fs::read_to_string(lib_path).expect("Failed to read declarations_library.slx");
    let consumer_code =
        fs::read_to_string(consumer_path).expect("Failed to read consumer_program.slx");

    let options = CompilationOptions::new()
        .with_source("declarations_library.slx", &lib_code)
        .with_source("consumer_program.slx", &consumer_code);

    let mut context = CompilationContext::new(options);

    let lex_meta = lex(&mut context).expect("Lexing multi-source context should succeed");
    assert!(lex_meta.tokens_count > 0);

    let parse_meta = parse(&mut context).expect("Parsing multi-source context should succeed");
    assert_eq!(parse_meta.files_parsed, 2);

    let bind_meta = bind(&mut context).expect("Binding multi-source context should succeed");
    assert!(bind_meta.symbols_bound > 0);
    assert!(bind_meta.classes_bound >= 4); // BaseEntity, User, Repository, ConsumerService

    let artifacts = context
        .binding_artifacts
        .as_ref()
        .expect("Binding artifacts must be populated");

    // 1. Verify global symbol registration
    assert!(
        artifacts
            .symbol_table
            .lookup_global("solix.tests.library.BaseEntity")
            .is_some()
    );
    assert!(
        artifacts
            .symbol_table
            .lookup_global("solix.tests.library.User")
            .is_some()
    );
    assert!(
        artifacts
            .symbol_table
            .lookup_global("solix.tests.library.Repository")
            .is_some()
    );
    assert!(
        artifacts
            .symbol_table
            .lookup_global("solix.tests.consumer.ConsumerService")
            .is_some()
    );

    // 2. Verify inheritance and VTable slots
    let user_symbol_id = artifacts
        .symbol_table
        .lookup_global("solix.tests.library.User")
        .expect("User symbol must exist");
    let user_layout = artifacts
        .layouts
        .get(&user_symbol_id)
        .expect("User class layout must exist");

    assert!(user_layout.base_vtable_id.is_some());
    assert!(user_layout.field_offsets.contains_key("id"));
    assert!(user_layout.field_offsets.contains_key("name"));
    assert!(user_layout.field_offsets.contains_key("status"));
    assert!(!user_layout.vtable_methods.is_empty());

    // 3. Verify on-the-fly generic monomorphization
    assert!(
        bind_meta.monomorphized_count >= 1,
        "Repository<User> must be monomorphized on the fly"
    );

    // 4. Verify expression type mapping
    assert!(
        !artifacts.node_types.is_empty(),
        "Node types must be recorded for LSP"
    );
}

#[test]
fn test_positive_master_all_statements_binding() {
    let fixture_path = Path::new("tests/fixtures/syntax/positive/master_all_statements.slx");
    let fixture_code =
        fs::read_to_string(fixture_path).expect("Failed to read master_all_statements.slx");

    let std_collections = r#"
        package std.collections;
        public class List<T> {}
        public class Map<K, V> {}
    "#;
    let std_io = r#"
        package std.io;
        public class File {}
    "#;

    let options = CompilationOptions::new()
        .with_source("master_all_statements.slx", &fixture_code)
        .with_source("std_collections.slx", std_collections)
        .with_source("std_io.slx", std_io);
    let mut context = CompilationContext::new(options);

    lex(&mut context).expect("Lexing master statements fixture should succeed");
    parse(&mut context).expect("Parsing master statements fixture should succeed");

    let bind_meta = bind(&mut context).expect("Binding master statements fixture should succeed");
    assert!(bind_meta.symbols_bound > 0);
    assert!(bind_meta.classes_bound >= 2);

    let artifacts = context
        .binding_artifacts
        .as_ref()
        .expect("Binding artifacts must be present");

    assert!(
        artifacts
            .symbol_table
            .lookup_global("solix.tests.master.BaseEntity")
            .is_some()
    );
    assert!(
        artifacts
            .symbol_table
            .lookup_global("solix.tests.master.MasterContainer")
            .is_some()
    );
}

#[test]
fn test_negative_master_semantics_fixture_detects_all_errors_with_recovery() {
    let fixture_path = Path::new("tests/fixtures/semantics/negative/master_invalid_semantics.slx");
    let fixture_code =
        fs::read_to_string(fixture_path).expect("Failed to read master_invalid_semantics.slx");

    let options =
        CompilationOptions::new().with_source("master_invalid_semantics.slx", &fixture_code);
    let mut context = CompilationContext::new(options);

    lex(&mut context).expect("Lexing negative semantics fixture should succeed");
    parse(&mut context).expect("Parsing negative semantics fixture should succeed");

    let bind_result = bind(&mut context);
    assert!(
        bind_result.is_err(),
        "master_invalid_semantics.slx must fail binding and return errors"
    );

    let errors = bind_result.unwrap_err();
    assert!(
        errors.len() >= 5,
        "Binder error recovery must report multiple errors across the file, got: {}",
        errors.len()
    );

    let mut found_invalid_base = false;
    let mut found_cyclic = false;
    let mut found_duplicate = false;
    let mut found_undefined = false;
    let mut found_type_mismatch = false;
    let mut found_break_outside_loop = false;
    let mut found_non_indexable = false;
    let mut found_return_type_mismatch = false;
    let mut found_static_access_via_dot = false;
    let mut found_instance_access_via_scope = false;

    for error in &errors {
        match error {
            BindingError::InvalidBaseClass { .. } => found_invalid_base = true,
            BindingError::CyclicInheritance { .. } => found_cyclic = true,
            BindingError::DuplicateSymbol { .. } => found_duplicate = true,
            BindingError::UndefinedSymbol { .. } => found_undefined = true,
            BindingError::TypeMismatch { .. } => found_type_mismatch = true,
            BindingError::BreakOrContinueOutsideLoop { .. } => found_break_outside_loop = true,
            BindingError::NonIndexableType { .. } => found_non_indexable = true,
            BindingError::ReturnTypeMismatch { .. } => found_return_type_mismatch = true,
            BindingError::StaticAccessViaDot { .. } => found_static_access_via_dot = true,
            BindingError::InstanceAccessViaScopeResolution { .. } => {
                found_instance_access_via_scope = true;
            }
            _ => {}
        }

        // Assert all error codes are standard E0201..E0227 format
        let code = error.error_code();
        assert!(
            code.starts_with("E02"),
            "Error code '{code}' must match standard semantic format 'E02xx'"
        );
        assert!(
            !error.source_span().is_empty(),
            "Error source span must not be empty"
        );
    }

    assert!(
        found_invalid_base,
        "E0205 InvalidBaseClass must be detected"
    );
    assert!(found_cyclic, "E0204 CyclicInheritance must be detected");
    assert!(found_duplicate, "E0202 DuplicateSymbol must be detected");
    assert!(found_undefined, "E0201 UndefinedSymbol must be detected");
    assert!(found_type_mismatch, "E0203 TypeMismatch must be detected");
    assert!(
        found_break_outside_loop,
        "E0219 BreakOrContinueOutsideLoop must be detected"
    );
    assert!(
        found_non_indexable,
        "E0221 NonIndexableType must be detected"
    );
    assert!(
        found_return_type_mismatch,
        "E0213 ReturnTypeMismatch must be detected"
    );
    assert!(
        found_static_access_via_dot,
        "E0226 StaticAccessViaDot must be detected"
    );
    assert!(
        found_instance_access_via_scope,
        "E0227 InstanceAccessViaScopeResolution must be detected"
    );
}

#[test]
fn test_symbol_signature_string_canonical_format() {
    let symbol = SymbolEntry {
        id: SymbolId(1),
        name: "find_user_by_email".to_string(),
        fully_qualified_name: "solix::models::UserService::find_user_by_email".to_string(),
        source_id: 0,
        span: Span::new(0, 0, 10),
        modifiers: vec![Modifier::Public],
        symbol_type: Type::FunctionPointer {
            return_type: Box::new(Type::Class {
                symbol_name: "solix::models::User".to_string(),
                type_arguments: Vec::new(),
            }),
            parameter_types: vec![
                Type::Primitive(PrimitiveKind::Int64),
                Type::Class {
                    symbol_name: "std::string".to_string(),
                    type_arguments: Vec::new(),
                },
                Type::Primitive(PrimitiveKind::Bool),
            ],
        },
        parent_symbol: None,
        members: Vec::new(),
        is_static: false,
        is_const: false,
        slot_index: None,
    };

    assert_eq!(
        symbol.to_signature_string(),
        "solix::models::User find_user_by_email(int64, string, bool)"
    );
}

#[test]
fn test_namespace_qualified_declarations_binding_and_emission() {
    let source_header = r#"
alias solix::types::Count = int32;

enum solix::io::FileMode {
    Read,
    Write
}

interface solix::io::IStream {
    void close();
}

class solix::io::Console {
    public static void print_message(string message) {
    }
}
"#;

    let source_consumer = r#"
import solix.io.Console;
import solix.io.FileMode;
import solix.types.Count;

public int32 main() {
    Console::print_message("Hello from qualified namespace!");
    Count counter = 42;
    FileMode mode = FileMode.Read;
    return counter;
}
"#;

    let options = CompilationOptions::new()
        .with_source("header.slx", source_header)
        .with_source("consumer.slx", source_consumer);

    let mut context = CompilationContext::new(options);

    let lex_result = lex(&mut context).expect("Lexing qualified declarations must succeed");
    assert!(lex_result.tokens_count > 0);

    let parse_result = parse(&mut context).expect("Parsing qualified declarations must succeed");
    assert_eq!(parse_result.files_parsed, 2);

    let bind_result = bind(&mut context).expect("Binding qualified declarations must succeed");
    assert!(bind_result.symbols_bound > 0);

    let artifacts = context
        .binding_artifacts
        .as_ref()
        .expect("Binding artifacts must be present");

    assert!(
        artifacts
            .symbol_table
            .lookup_global("solix::io::Console")
            .is_some(),
        "solix::io::Console must be registered globally"
    );
    assert!(
        artifacts
            .symbol_table
            .lookup_global("solix::io::IStream")
            .is_some(),
        "solix::io::IStream must be registered globally"
    );
    assert!(
        artifacts
            .symbol_table
            .lookup_global("solix::io::FileMode")
            .is_some(),
        "solix::io::FileMode must be registered globally"
    );
    assert!(
        artifacts
            .symbol_table
            .lookup_global("solix::types::Count")
            .is_some(),
        "solix::types::Count must be registered globally"
    );
}
