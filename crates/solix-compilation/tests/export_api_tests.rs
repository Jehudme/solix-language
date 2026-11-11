//! Integration tests verifying Milestone 023 Pillar 2: Pre-Generated API Headers,
//! Pruned `ExportedAst` serialization, and Local-Only dependency isolation.

#![forbid(unsafe_code)]

use solix_common::bytecode::BinaryProgram;
use solix_common::bytecode::manifest::ExportManifest;
use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
use solix_compilation::CompilationPipeline;
use solix_compilation::phases::export_api::generate_api_header_and_ast;
use solix_compilation::phases::parser::statements::ProgramAst;

#[test]
fn test_export_api_header_qualified_names_and_field_layout_preservation() {
    let source_code = r#"
package solix.io;

public class Console {
    private int32 internal_descriptor;
    protected int32 buffer_size;
    public string device_name;

    public Console(int32 descriptor) {
        this.internal_descriptor = descriptor;
    }

    public static void print(string message) {
    }

    public static void println(string message) {
    }

    private void internal_flush() {
    }

    ~Console() {
    }
}
"#;

    let options = CompilationOptions::new().with_source("Console.slx", source_code);
    let mut context = CompilationContext::new(options);
    let mut pipeline = CompilationPipeline::standard();
    let compilation_result = pipeline.execute(&mut context);
    assert!(
        compilation_result.is_ok(),
        "Compilation of Console must succeed: {compilation_result:?}"
    );

    let (api_header, exported_ast_bytes) = generate_api_header_and_ast(&context);

    // 1. Check qualified class name
    assert!(
        api_header.contains("extern public class solix::io::Console"),
        "API header must contain namespace-qualified class declaration: {api_header}"
    );

    // 2. Check that instance fields (private, protected, public) are preserved for sizeof/layout
    assert!(
        api_header.contains("private int32 internal_descriptor;"),
        "Private instance field must be retained for memory layout: {api_header}"
    );
    assert!(
        api_header.contains("protected int32 buffer_size;"),
        "Protected instance field must be retained for memory layout: {api_header}"
    );
    assert!(
        api_header.contains("public string device_name;"),
        "Public instance field must be retained for memory layout: {api_header}"
    );

    // 3. Check public methods and constructors are emitted as bodiless extern signatures
    assert!(
        api_header.contains("public Console(int32 descriptor);"),
        "Public constructor signature must be emitted: {api_header}"
    );
    assert!(
        api_header.contains("public static void print(string message);"),
        "Public static method must be emitted: {api_header}"
    );

    // 4. Check that private methods and destructors are stripped
    assert!(
        !api_header.contains("internal_flush"),
        "Private method internal_flush must NOT be in API header: {api_header}"
    );
    assert!(
        !api_header.contains("~Console"),
        "Destructor ~Console must NOT be in API header: {api_header}"
    );

    // 5. Check pre-parsed AST serialization
    assert!(
        !exported_ast_bytes.is_empty(),
        "Exported AST bytes must be populated"
    );
    let deserialized_ast = bincode::deserialize::<ProgramAst>(&exported_ast_bytes);
    assert!(
        deserialized_ast.is_ok(),
        "Exported AST bytes must deserialize cleanly into ProgramAst via bincode"
    );
}

#[test]
fn test_export_manifest_local_only_invariant() {
    let mut dependency_program = BinaryProgram::new();
    dependency_program.package_name = "dep_lib".to_string();
    dependency_program.package_version = "1.0.0".to_string();
    dependency_program.export_manifest = ExportManifest {
        package_name: "dep_lib".to_string(),
        package_version: "1.0.0".to_string(),
        package_description: "External Dependency".to_string(),
        build_profile: "debug".to_string(),
        native_library_checksums: std::collections::HashMap::new(),
        dependencies: Vec::new(),
        exported_classes: Vec::new(),
        exported_enums: Vec::new(),
        exported_functions: Vec::new(),
        exported_statics: Vec::new(),
        generic_templates: Vec::new(),
        source_files: Vec::new(),
        api_header: "extern public class dep_lib::DepClass { public void dep_method(); }\n"
            .to_string(),
        exported_ast_bytes: Vec::new(),
    };

    let user_source = r#"
package user_app;

public class App {
    public static int32 run() {
        return 42;
    }
}
"#;

    let options = CompilationOptions::new()
        .with_source("App.slx", user_source)
        .with_dependency_binary(dependency_program);

    let mut context = CompilationContext::new(options);
    let mut pipeline = CompilationPipeline::standard();
    let compilation_result = pipeline.execute(&mut context);
    assert!(
        compilation_result.is_ok(),
        "Compilation with dependency must succeed: {compilation_result:?}"
    );

    let emission_artifacts = context
        .emission_artifacts
        .expect("Emission artifacts must be produced");

    // The output binary program must ONLY export user_app::App, NOT dep_lib::DepClass!
    let exported_class_names: Vec<&str> = emission_artifacts
        .program
        .export_manifest
        .exported_classes
        .iter()
        .map(|exported_class| exported_class.name.as_str())
        .collect();

    assert!(
        exported_class_names.contains(&"App"),
        "Local class 'App' must be exported: {exported_class_names:?}"
    );
    assert!(
        !exported_class_names.contains(&"DepClass"),
        "External dependency class 'DepClass' must NOT be re-exported (local-only invariant): {exported_class_names:?}"
    );

    let output_api_header = &emission_artifacts.program.export_manifest.api_header;
    assert!(
        output_api_header.contains("user_app::App"),
        "API header must contain user_app::App: {output_api_header}"
    );
    assert!(
        !output_api_header.contains("DepClass"),
        "API header must NOT contain external DepClass: {output_api_header}"
    );
}

#[test]
fn test_dependency_pre_parsed_ast_injection_and_resolution() {
    let dep_source = r#"
package math.core;

public class Calculator {
    public static int32 add(int32 left_operand, int32 right_operand) {
        return left_operand + right_operand;
    }
}
"#;

    let dep_options = CompilationOptions::new()
        .with_package_metadata("math", "1.0.0", "Math Library")
        .with_source("Calculator.slx", dep_source);
    let mut dep_context = CompilationContext::new(dep_options);
    let mut dep_pipeline = CompilationPipeline::standard();
    let dep_result = dep_pipeline.execute(&mut dep_context);
    assert!(dep_result.is_ok(), "Library compilation must succeed");

    let dep_program = dep_context
        .emission_artifacts
        .expect("Emission artifacts must exist")
        .program;

    assert!(
        !dep_program.export_manifest.exported_ast_bytes.is_empty(),
        "Exported AST bytes must be present in library container"
    );

    let consumer_source = r#"
package client;

extern import math.core.Calculator;

public class Main {
    public static int32 main() {
        return Calculator::add(10, 20);
    }
}
"#;

    let consumer_options = CompilationOptions::new()
        .with_source("Main.slx", consumer_source)
        .with_dependency_binary(dep_program);

    let mut consumer_context = CompilationContext::new(consumer_options);
    let mut consumer_pipeline = CompilationPipeline::standard();
    let consumer_result = consumer_pipeline.execute(&mut consumer_context);
    assert!(
        consumer_result.is_ok(),
        "Consumer compiling against pre-parsed AST dependency must succeed: {consumer_result:?}"
    );

    // Verify external source was marked as external and didn't re-lex or re-parse from scratch
    let dep_source_id = consumer_context
        .get_source_id("math@1.0.0.slx")
        .expect("Dependency synthetic source must be registered");
    assert!(
        consumer_context.is_external_source(dep_source_id),
        "Dependency source must be marked as external source"
    );
}
