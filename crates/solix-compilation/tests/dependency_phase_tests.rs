//! Comprehensive unit and integration tests for `DependencyResolutionPhase`, `extern import`,
//! and root facade binding.

#![forbid(unsafe_code)]

use solix_common::bytecode::BinaryProgram;
use solix_common::bytecode::manifest::{
    DeclarationModifier, ExportManifest, ExportedFunction, ExportedParameter,
};
use solix_common::solver::{
    DependencyRequirement, InMemoryModuleSolver, PackageVersion, SolvedPackage,
};
use solix_common::{PrimitiveKind, Type};
use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
use std::sync::Arc;

fn create_dummy_library_binary(
    package_name: &str,
    package_version: &str,
    dependencies: Vec<DependencyRequirement>,
    exported_function_name: &str,
) -> BinaryProgram {
    let mut program = BinaryProgram::new();
    program.package_name = package_name.to_string();
    program.package_version = package_version.to_string();
    program.package_description = format!("Test package {package_name}");
    program.export_manifest = ExportManifest {
        entry_point: None,
        package_name: package_name.to_string(),
        package_version: package_version.to_string(),
        package_description: format!("Test package {package_name}"),
        build_profile: "debug".to_string(),
        native_library_checksums: std::collections::HashMap::new(),
        dependencies,
        exported_classes: Vec::new(),
        exported_enums: Vec::new(),
        exported_functions: vec![ExportedFunction {
            symbol: format!("{package_name}::{exported_function_name}"),
            simple_name: exported_function_name.to_string(),
            modifiers: vec![DeclarationModifier::Public],
            parameters: vec![
                ExportedParameter {
                    name: "first_value".to_string(),
                    param_type: Type::Primitive(PrimitiveKind::Int32),
                },
                ExportedParameter {
                    name: "second_value".to_string(),
                    param_type: Type::Primitive(PrimitiveKind::Int32),
                },
            ],
            return_type: Type::Primitive(PrimitiveKind::Int32),
            is_constructor: false,
            is_destructor: false,
            is_operator: false,
        }],
        exported_statics: Vec::new(),
        generic_templates: Vec::new(),
        source_files: Vec::new(),
        api_header: String::new(),
        exported_ast_bytes: Vec::new(),
    };
    program
}

#[test]
fn test_dependency_resolution_phase_transitive_tree_resolution() {
    let math_v110 = create_dummy_library_binary("math", "1.1.0", Vec::new(), "add");
    let math_v130 = create_dummy_library_binary("math", "1.3.0", Vec::new(), "add");

    let engine_req = DependencyRequirement::new("math", PackageVersion::new(1, 1, 0));
    let engine_v100 =
        create_dummy_library_binary("engine", "1.0.0", vec![engine_req], "init_engine");

    let mut solver = InMemoryModuleSolver::new();
    solver.add_package(SolvedPackage::new(
        "math",
        PackageVersion::new(1, 1, 0),
        math_v110,
    ));
    solver.add_package(SolvedPackage::new(
        "math",
        PackageVersion::new(1, 3, 0),
        math_v130,
    ));
    solver.add_package(SolvedPackage::new(
        "engine",
        PackageVersion::new(1, 0, 0),
        engine_v100,
    ));

    let consumer_code = r#"
        import engine.*;
        import math.*;

        class Main {
            public static int32 main() {
                int32 initial = init_engine(1, 2);
                int32 total = add(initial, 10);
                return total;
            }
        }
    "#;

    let app_dependency = DependencyRequirement::new("engine", PackageVersion::new(1, 0, 0));

    let options = CompilationOptions::new()
        .with_source("main.slx", consumer_code)
        .with_dependency_spec(app_dependency)
        .with_solver(Arc::new(solver));

    let mut context = CompilationContext::new(options);
    context
        .execute()
        .expect("Compilation with transitive dependency resolution must succeed");

    assert!(context.dependencies_map.contains_key("engine"));
    assert!(context.dependencies_map.contains_key("math"));
    // Math should be resolved to the highest compatible version: 1.3.0
    assert_eq!(
        context.dependencies_map["math"].version,
        PackageVersion::new(1, 3, 0)
    );

    let artifacts = context
        .emission_artifacts
        .expect("Emission artifacts must be present");
    let recorded_dependencies = &artifacts.program.export_manifest.dependencies;
    assert_eq!(recorded_dependencies.len(), 2);
    assert!(
        recorded_dependencies
            .iter()
            .any(|dep| dep.name == "engine" && dep.version == PackageVersion::new(1, 0, 0))
    );
    assert!(
        recorded_dependencies
            .iter()
            .any(|dep| dep.name == "math" && dep.version == PackageVersion::new(1, 3, 0))
    );
}

#[test]
fn test_dependency_resolution_phase_diamond_incompatible_conflict_fails() {
    let core_v100 = create_dummy_library_binary("core", "1.0.0", Vec::new(), "core_init");
    let core_v200 = create_dummy_library_binary("core", "2.0.0", Vec::new(), "core_init");

    let lib_a_req = DependencyRequirement::new("core", PackageVersion::new(1, 0, 0));
    let lib_a = create_dummy_library_binary("lib_a", "1.0.0", vec![lib_a_req], "do_a");

    let lib_b_req = DependencyRequirement::new("core", PackageVersion::new(2, 0, 0));
    let lib_b = create_dummy_library_binary("lib_b", "1.0.0", vec![lib_b_req], "do_b");

    let mut solver = InMemoryModuleSolver::new();
    solver.add_package(SolvedPackage::new(
        "core",
        PackageVersion::new(1, 0, 0),
        core_v100,
    ));
    solver.add_package(SolvedPackage::new(
        "core",
        PackageVersion::new(2, 0, 0),
        core_v200,
    ));
    solver.add_package(SolvedPackage::new(
        "lib_a",
        PackageVersion::new(1, 0, 0),
        lib_a,
    ));
    solver.add_package(SolvedPackage::new(
        "lib_b",
        PackageVersion::new(1, 0, 0),
        lib_b,
    ));

    let consumer_code = r#"
        class Main {
            public static int32 main() {
                return 0;
            }
        }
    "#;

    let options = CompilationOptions::new()
        .with_source("main.slx", consumer_code)
        .with_dependency_spec(DependencyRequirement::new("lib_a", PackageVersion::new(1, 0, 0)))
        .with_dependency_spec(DependencyRequirement::new("lib_b", PackageVersion::new(1, 0, 0)))
        .with_solver(Arc::new(solver));

    let mut context = CompilationContext::new(options);
    let result = context.execute();
    assert!(
        result.is_err(),
        "Diamond dependency major version conflict must fail compilation"
    );
    let diagnostics = result.unwrap_err();
    let has_incompatible_error = diagnostics.iter().any(|diagnostic| {
        diagnostic.to_string().contains("E0289")
            || diagnostic.to_string().contains("Conflicting major version")
    });
    assert!(
        has_incompatible_error,
        "Must report E0289 incompatible dependency version"
    );
}

#[test]
fn test_dependency_resolution_phase_missing_package_fails() {
    let consumer_code = r#"
        class Main {
            public static int32 main() {
                return 0;
            }
        }
    "#;

    let solver = InMemoryModuleSolver::new();

    let options = CompilationOptions::new()
        .with_source("main.slx", consumer_code)
        .with_dependency_spec(DependencyRequirement::new(
            "non_existent_package",
            PackageVersion::new(1, 0, 0),
        ))
        .with_solver(Arc::new(solver));

    let mut context = CompilationContext::new(options);
    let result = context.execute();
    assert!(result.is_err(), "Missing package must fail compilation");
    let diagnostics = result.unwrap_err();
    let has_not_found_error = diagnostics.iter().any(|diagnostic| {
        diagnostic.to_string().contains("E0290")
            || diagnostic
                .to_string()
                .contains("not found in package store")
    });
    assert!(
        has_not_found_error,
        "Must report E0290 external package not found"
    );
}

#[test]
fn test_extern_import_on_the_fly_resolution_and_root_facade() {
    let raylib_binary = create_dummy_library_binary("raylib", "1.2.0", Vec::new(), "initWindow");

    let mut solver = InMemoryModuleSolver::new();
    solver.add_package(SolvedPackage::new(
        "raylib",
        PackageVersion::new(1, 2, 0),
        raylib_binary,
    ));

    // Notice: no --deps option passed! The file uses 'extern import raylib.*;'
    let consumer_code = r#"
        extern import raylib.*;

        class Main {
            public static int32 main() {
                int32 handle_a = initWindow(800, 600);
                int32 handle_b = raylib.initWindow(1024, 768);
                return handle_a + handle_b;
            }
        }
    "#;

    let options = CompilationOptions::new()
        .with_source("app.slx", consumer_code)
        .with_solver(Arc::new(solver));

    let mut context = CompilationContext::new(options);
    context
        .execute()
        .expect("Compilation with extern import must resolve and compile cleanly");

    assert!(context.dependencies_map.contains_key("raylib"));
    assert_eq!(
        context.dependencies_map["raylib"].version,
        PackageVersion::new(1, 2, 0)
    );

    let artifacts = context
        .emission_artifacts
        .expect("Emission artifacts must be present");
    let dependencies = &artifacts.program.export_manifest.dependencies;
    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "raylib");
    assert_eq!(dependencies[0].version, PackageVersion::new(1, 2, 0));
}

#[test]
fn test_extern_import_specific_symbol_and_root_facade() {
    let math_binary = create_dummy_library_binary("math", "2.0.0", Vec::new(), "compute_sum");

    let mut solver = InMemoryModuleSolver::new();
    solver.add_package(SolvedPackage::new(
        "math",
        PackageVersion::new(2, 0, 0),
        math_binary,
    ));

    let consumer_code = r#"
        extern import math.compute_sum;

        class Main {
            public static int32 main() {
                int32 result_unqualified = compute_sum(4, 5);
                int32 result_qualified = math.compute_sum(10, 20);
                return result_unqualified + result_qualified;
            }
        }
    "#;

    let options = CompilationOptions::new()
        .with_source("calc.slx", consumer_code)
        .with_solver(Arc::new(solver));

    let mut context = CompilationContext::new(options);
    context
        .execute()
        .expect("Compilation with extern import specific symbol must succeed");

    assert_eq!(
        context.dependencies_map["math"].version,
        PackageVersion::new(2, 0, 0)
    );
}
