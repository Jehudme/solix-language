//! Integration tests for Milestone 023 Pillar 3: Dual-Mode Linking and Static DCE Reachability.

#![forbid(unsafe_code)]

use solix_common::bytecode::{BinaryProgram, CompiledFunction, Instruction, OpCode};
use solix_compilation::CompilationContext::{
    BuildProfile, CompilationContext, CompilationOptions, LinkMode,
};
use solix_compilation::CompilationPipeline;
use solix_runtime::context::RuntimeContext;
use solix_runtime::value::Value;
use solix_runtime::{RuntimeOptions, VirtualMachine};

#[test]
fn test_static_linking_relocates_external_function_and_rewrites_to_call_direct() {
    // 1. Build a dependency binary with a reachable function and an unreachable dead function
    let mut dependency_binary = BinaryProgram::new();
    dependency_binary.package_name = "math_lib".to_string();

    // Reachable function: math_lib::add(a, b) -> a + b
    let mut add_function = CompiledFunction::new("math_lib::add", 2, 0, 4);
    // arg0: r[0], arg1: r[1], result: r[0] = r[0] + r[1]; return r[0]
    add_function
        .instructions
        .push(Instruction::format_a(OpCode::AddI32, 0, 0, 0, 1));
    add_function
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 0, 0, 0));
    dependency_binary.functions.push(add_function);

    // Unreachable function: math_lib::dead_function()
    let mut dead_function = CompiledFunction::new("math_lib::dead_function", 0, 0, 2);
    dead_function
        .instructions
        .push(Instruction::format_b(OpCode::LoadI32, 0, 0, 42));
    dead_function
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 0, 0, 0));
    dependency_binary.functions.push(dead_function);

    // 2. Build consumer code that calls math_lib::add
    let consumer_source = r#"
package app;

extern public int32 math_lib::add(int32 first, int32 second);

public int32 main() {
    return math_lib::add(15, 27);
}
"#;

    let mut compilation_options = CompilationOptions::new()
        .with_profile(BuildProfile::Debug)
        .with_link_mode(LinkMode::Static)
        .with_source("consumer.slx", consumer_source);
    compilation_options
        .dependency_binaries
        .push(dependency_binary);

    let mut context = CompilationContext::new(compilation_options);
    let mut pipeline = CompilationPipeline::standard();
    let compilation_result = pipeline.execute(&mut context);
    assert!(
        compilation_result.is_ok(),
        "Compilation failed: {:?}",
        compilation_result.err()
    );

    let artifacts = context.emit_context().expect("emit_context failed");

    let binary = &artifacts.program;

    // Verify unreachable function was pruned by Dead Code Elimination
    assert!(
        !binary
            .functions
            .iter()
            .any(|function| function.symbol.contains("dead_function")),
        "Dead function should have been eliminated by reachability DCE"
    );

    // Verify math_lib::add was merged into the binary
    assert!(
        binary
            .functions
            .iter()
            .any(|function| function.symbol.contains("math_lib::add")),
        "Reachable function math_lib::add must be statically linked into the binary"
    );

    // Verify main's call instruction was rewritten from CallExtern to CallDirect
    let main_function = binary
        .functions
        .iter()
        .find(|function| function.symbol == "main" || function.symbol.ends_with("::main"))
        .expect("main function must exist in binary");

    let has_call_direct = main_function
        .instructions
        .iter()
        .any(|instruction| instruction.raw_opcode() == OpCode::CallDirect as u8);

    assert!(
        has_call_direct,
        "Static linking must rewrite CallExtern to CallDirect"
    );

    let has_call_extern = main_function
        .instructions
        .iter()
        .any(|instruction| instruction.raw_opcode() == OpCode::CallExtern as u8);

    assert!(
        !has_call_extern,
        "Static binary must not retain CallExtern for resolved static dependency"
    );

    // Execute standalone without external dependency loaded into VM
    let runtime_options = RuntimeOptions::new().with_binary(binary.clone());
    let runtime_context = RuntimeContext::new(runtime_options)
        .expect("Failed to create runtime context for statically linked binary");
    let mut virtual_machine = VirtualMachine::new(runtime_context);
    let execution_result = virtual_machine.run().expect("Execution failed");
    assert_eq!(
        execution_result,
        Value::I32(42),
        "Statically linked execution should return 15 + 27 = 42"
    );
}

#[test]
fn test_dynamic_linking_mode_retains_call_extern() {
    let consumer_source = r#"
package app;

extern public int32 external_service::compute(int32 input);

public int32 main() {
    return external_service::compute(10);
}
"#;

    let compilation_options = CompilationOptions::new()
        .with_profile(BuildProfile::Debug)
        .with_link_mode(LinkMode::Dynamic)
        .with_source("consumer_dyn.slx", consumer_source);

    let mut context = CompilationContext::new(compilation_options);
    let mut pipeline = CompilationPipeline::standard();
    let compilation_result = pipeline.execute(&mut context);
    assert!(
        compilation_result.is_ok(),
        "Compilation failed: {:?}",
        compilation_result.err()
    );

    let artifacts = context.emit_context().expect("emit_context failed");

    let binary = &artifacts.program;

    let main_function = binary
        .functions
        .iter()
        .find(|function| function.symbol == "main" || function.symbol.ends_with("::main"))
        .expect("main function must exist in binary");

    let has_call_extern = main_function
        .instructions
        .iter()
        .any(|instruction| instruction.raw_opcode() == OpCode::CallExtern as u8);

    assert!(
        has_call_extern,
        "Dynamic linking mode must preserve CallExtern for external symbols"
    );
}
