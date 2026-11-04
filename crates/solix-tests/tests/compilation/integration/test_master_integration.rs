//! Master end-to-end integration test suites compiling and executing complete Solix fixtures.

#![forbid(unsafe_code)]

use solix_common::bytecode::BinaryProgram;
use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
use solix_compilation::CompilationPipeline;
use solix_runtime::value::model::Value;
use solix_tests::common::run_binary;

fn compile_fixture(file_name: &str, source_code: &str) -> BinaryProgram {
    let options = CompilationOptions::new().with_source(file_name, source_code);
    let mut context = CompilationContext::new(options);

    context.execute().expect("Compilation failed");

    let artifacts = context
        .emission_artifacts
        .as_ref()
        .expect("Bytecode emission artifacts must be present");
    artifacts.program.clone()
}

#[test]
fn test_master_all_statements_emission_and_execution() {
    let source_code = include_str!("../../../fixtures/master_all_statements_emission.slx");
    let program = compile_fixture("master_all_statements_emission.slx", source_code);
    let (res, vm) = run_binary(program, Some("Runner::run_all_checks"));
    assert_eq!(res, Value::I32(358));
    assert!(vm.verify_clean_memory().is_ok());
}

#[test]
fn test_master_all_features_exhaustive_integration() {
    let source_code = include_str!("../../../fixtures/master_all_features_exhaustive.slx");
    let program = compile_fixture("master_all_features_exhaustive.slx", source_code);
    let (res, vm) = run_binary(program, Some("App::main"));
    assert_eq!(res, Value::I32(0), "App::main must return exit code 0");

    let expected_checkpoints = [
        "CHECK 1: OOP AND INHERITANCE OK",
        "CHECK 2: INTERFACES AND VIRTUAL DISPATCH OK",
        "CHECK 3: GENERICS AND MONOMORPHIZATION OK",
        "CHECK 4: OPERATOR OVERLOADING OK",
        "CHECK 5: CONTROL FLOW AND SWITCH OK",
        "CHECK 6: EXCEPTION HANDLING AND FINALLY OK",
        "CHECK 7: SIZEOF AND INSTANCEOF OK",
        "CHECK 8: TUPLES AND TUPLE INSTANCEOF OK",
        "CHECK 9: ARRAYS AND LENGTH OK",
        "CHECK 10: CLOSURES AND LAMBDAS OK",
        "ALL CHECKS PASSED SUCCESSFULLY",
    ];

    for checkpoint in &expected_checkpoints {
        assert!(
            vm.stdout.iter().any(|line| line.contains(checkpoint)),
            "Missing expected checkpoint in stdout: '{}'",
            checkpoint
        );
    }

    assert!(vm.verify_clean_memory().is_ok());
}

#[test]
fn test_master_all_statements_parsing() {
    let source_code = include_str!("../../../fixtures/master_all_statements.slx");
    let options = CompilationOptions::new().with_source("master_all_statements.slx", source_code);
    let mut context = CompilationContext::new(options);

    CompilationPipeline::syntax_only()
        .execute(&mut context)
        .expect("Parsing failed");
    assert!(context.ast[0].is_some(), "AST must be produced");
}
