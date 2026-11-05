//! Unit tests for runtime dependency tree verification, SemVer checking, and auto-loading.

#![forbid(unsafe_code)]

use solix_common::bytecode::manifest::{
    DeclarationModifier, ExportManifest, ExportedFunction, ExportedParameter,
};
use solix_common::bytecode::opcodes::OpCode;
use solix_common::bytecode::{BinaryProgram, CompiledFunction, Instruction};
use solix_common::solver::{
    DependencyRequirement, InMemoryModuleSolver, PackageVersion, SolvedPackage,
};
use solix_common::{PrimitiveKind, Type};
use solix_runtime::exceptions::RuntimeErrorCode;
use solix_runtime::options::RuntimeOptions;
use solix_runtime::{RuntimeContext, Value, VirtualMachine};
use std::sync::Arc;

fn create_math_binary(package_version: &str) -> BinaryProgram {
    let mut program = BinaryProgram::new();
    program.package_name = "math".to_string();
    program.package_version = package_version.to_string();
    program.export_manifest = ExportManifest {
        package_name: "math".to_string(),
        package_version: package_version.to_string(),
        package_description: "Math Library".to_string(),
        dependencies: Vec::new(),
        exported_classes: Vec::new(),
        exported_functions: vec![ExportedFunction {
            symbol: "math::add".to_string(),
            simple_name: "add".to_string(),
            modifiers: vec![DeclarationModifier::Public],
            parameters: vec![
                ExportedParameter {
                    name: "first".to_string(),
                    param_type: Type::Primitive(PrimitiveKind::Int32),
                },
                ExportedParameter {
                    name: "second".to_string(),
                    param_type: Type::Primitive(PrimitiveKind::Int32),
                },
            ],
            return_type: Type::Primitive(PrimitiveKind::Int32),
            is_constructor: false,
            is_destructor: false,
        }],
        exported_statics: Vec::new(),
        generic_templates: Vec::new(),
    };

    // Compiled function: math::add(r0, r1) -> r2 = r0 + r1; return r2;
    let mut add_fn = CompiledFunction::new("math::add", 0, 2, 3);
    add_fn
        .instructions
        .push(Instruction::format_a(OpCode::AddI32, 0, 2, 0, 1));
    add_fn
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 2, 0, 0));
    program.functions.push(add_fn);

    program
}

fn create_app_binary_requiring(dependency_version: PackageVersion) -> BinaryProgram {
    let mut program = BinaryProgram::new();
    program.package_name = "app".to_string();
    program.package_version = "1.0.0".to_string();
    program.entry_point_id = Some(0);
    program.entry_point_symbol = Some("main".to_string());
    program.export_manifest = ExportManifest {
        package_name: "app".to_string(),
        package_version: "1.0.0".to_string(),
        package_description: "Application".to_string(),
        dependencies: vec![DependencyRequirement {
            name: "math".to_string(),
            version: dependency_version,
        }],
        exported_classes: Vec::new(),
        exported_functions: Vec::new(),
        exported_statics: Vec::new(),
        generic_templates: Vec::new(),
    };

    let sym_idx = program
        .constant_pool
        .add_string("math::add")
        .expect("Adding symbol string must succeed");

    // Compiled main: r0 = 10, r1 = 20, r2 = CallExtern("math::add"), return r2;
    let mut main_fn = CompiledFunction::new("main", 0, 0, 3);
    main_fn
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 10));
    main_fn
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 20));
    main_fn.instructions.push(Instruction::format_call(
        OpCode::CallExtern,
        2,
        sym_idx,
        0,
        2,
    ));
    main_fn
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 2, 0, 0));
    program.functions.push(main_fn);

    program
}

#[test]
fn test_runtime_dependency_missing_without_solver_fails() {
    let app_binary = create_app_binary_requiring(PackageVersion::new(1, 2, 0));
    let options = RuntimeOptions::new().with_binary(app_binary);

    let result = RuntimeContext::new(options);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.error_code(), RuntimeErrorCode::DependencyError);
    assert_eq!(error.error_code().as_i32(), 23);
    assert!(
        error
            .message()
            .contains("Missing required dependency 'math@1.2.0'")
    );
}

#[test]
fn test_runtime_dependency_incompatible_major_version_fails() {
    let app_binary = create_app_binary_requiring(PackageVersion::new(1, 2, 0));
    let math_v200 = create_math_binary("2.0.0");

    let mut solver = InMemoryModuleSolver::new();
    solver.add_package(SolvedPackage::new(
        "math",
        PackageVersion::new(2, 0, 0),
        math_v200,
    ));

    let options = RuntimeOptions::new()
        .with_binary(app_binary)
        .with_solver(Arc::new(solver));

    let result = RuntimeContext::new(options);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.error_code(), RuntimeErrorCode::DependencyError);
}

#[test]
fn test_runtime_dependency_older_incompatible_version_fails() {
    let app_binary = create_app_binary_requiring(PackageVersion::new(1, 2, 0));
    let math_v110 = create_math_binary("1.1.0");

    let mut solver = InMemoryModuleSolver::new();
    solver.add_package(SolvedPackage::new(
        "math",
        PackageVersion::new(1, 1, 0),
        math_v110,
    ));

    let options = RuntimeOptions::new()
        .with_binary(app_binary)
        .with_solver(Arc::new(solver));

    let result = RuntimeContext::new(options);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.error_code(), RuntimeErrorCode::DependencyError);
}

#[test]
fn test_runtime_dependency_auto_loads_compatible_newer_version_and_executes() {
    // App was compiled against 1.2.0
    let app_binary = create_app_binary_requiring(PackageVersion::new(1, 2, 0));
    // Host PC B has 1.4.2 installed
    let math_v142 = create_math_binary("1.4.2");

    let mut solver = InMemoryModuleSolver::new();
    solver.add_package(SolvedPackage::new(
        "math",
        PackageVersion::new(1, 4, 2),
        math_v142,
    ));

    let options = RuntimeOptions::new()
        .with_binary(app_binary)
        .with_solver(Arc::new(solver));

    let context =
        RuntimeContext::new(options).expect("Runtime dependency auto-linking must succeed");
    let mut vm = VirtualMachine::new(context);
    let result = vm.run().expect("VM execution must succeed");
    assert_eq!(result, Value::I32(30));
}
