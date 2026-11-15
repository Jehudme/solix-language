//! Multi-entrypoint runtime and duplicate symbol collision tests.

#![forbid(unsafe_code)]

use solix_common::bytecode::{
    BinaryProgram, CompiledFunction, Instruction, OpCode,
};
use solix_common::types::{PrimitiveKind, Type};
use solix_runtime::context::RuntimeContext;
use solix_runtime::exceptions::RuntimeError;
use solix_runtime::options::RuntimeOptions;
use solix_runtime::value::Value;
use solix_runtime::vm::VirtualMachine;

#[test]
fn test_multiple_entrypoints_share_static_state_and_raii_shutdown() {
    let mut binary_program = BinaryProgram::new();
    binary_program.package_name = "engine".to_string();
    binary_program.package_version = "1.0.0".to_string();
    binary_program.static_space_size = 1;
    binary_program
        .static_symbols_map
        .push(("engine::global_counter".to_string(), 0));

    // 1. @init: writes 100 into static slot 0
    let mut init_function = CompiledFunction::new("@init", 0, 0, 2);
    init_function.is_public = false;
    init_function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 100));
    init_function
        .instructions
        .push(Instruction::format_b(OpCode::SetStatic, 0, 0, 0));
    init_function
        .instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));
    binary_program.functions.push(init_function);

    // 2. add_to_counter(int32 delta) -> returns new counter
    let mut add_function = CompiledFunction::new("engine::add_to_counter", 0, 1, 4);
    add_function.is_public = true;
    add_function.return_type = Type::Primitive(PrimitiveKind::Int32);
    add_function.parameter_types = vec![Type::Primitive(PrimitiveKind::Int32)];
    // r0 has delta
    // r1 = GetStatic(0)
    add_function
        .instructions
        .push(Instruction::format_b(OpCode::GetStatic, 0, 1, 0));
    // r2 = r1 + r0
    add_function
        .instructions
        .push(Instruction::format_a(OpCode::AddI32, 0, 2, 1, 0));
    // SetStatic(0, r2)
    add_function
        .instructions
        .push(Instruction::format_b(OpCode::SetStatic, 0, 2, 0));
    // ReturnValue(r2)
    add_function
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 2, 0, 0));
    binary_program.functions.push(add_function);

    // 3. get_counter() -> returns counter
    let mut get_function = CompiledFunction::new("engine::get_counter", 0, 0, 2);
    get_function.is_public = true;
    get_function.return_type = Type::Primitive(PrimitiveKind::Int32);
    // r0 = GetStatic(0)
    get_function
        .instructions
        .push(Instruction::format_b(OpCode::GetStatic, 0, 0, 0));
    // ReturnValue(r0)
    get_function
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 0, 0, 0));
    binary_program.functions.push(get_function);

    let runtime_options = RuntimeOptions::new().with_binary(binary_program);
    let runtime_context = RuntimeContext::new(runtime_options).expect("Context construction must succeed");
    let mut virtual_machine =
        VirtualMachine::new_initialized(runtime_context).expect("VM initialization must succeed");

    // First invocation: add 25 to 100 -> 125
    let first_result = virtual_machine
        .invoke_entry("engine::add_to_counter", &[Value::I32(25)])
        .expect("First entry invocation must succeed");
    assert_eq!(first_result, Value::I32(125));

    // Second invocation: add 75 to 125 -> 200
    let second_result = virtual_machine
        .invoke_entry("engine::add_to_counter", &[Value::I32(75)])
        .expect("Second entry invocation must succeed");
    assert_eq!(second_result, Value::I32(200));

    // Third invocation: read counter without arguments
    let read_result = virtual_machine
        .invoke_entry("engine::get_counter", &[])
        .expect("Third entry invocation must succeed");
    assert_eq!(read_result, Value::I32(200));

    // Shutdown should succeed cleanly
    assert!(virtual_machine.shutdown().is_ok());
}

#[test]
fn test_non_public_or_non_primitive_entrypoint_rejected() {
    let mut binary_program = BinaryProgram::new();

    // 1. Private function
    let mut private_function = CompiledFunction::new("secret_function", 0, 0, 2);
    private_function.is_public = false;
    private_function.return_type = Type::Primitive(PrimitiveKind::Void);
    private_function
        .instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));
    binary_program.functions.push(private_function);

    // 2. Public function with custom class parameter
    let mut complex_function = CompiledFunction::new("complex_param_function", 0, 1, 2);
    complex_function.is_public = true;
    complex_function.return_type = Type::Primitive(PrimitiveKind::Void);
    complex_function.parameter_types = vec![Type::Class {
        symbol_name: "CustomWidget".to_string(),
        type_arguments: Vec::new(),
    }];
    complex_function
        .instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));
    binary_program.functions.push(complex_function);

    let runtime_options = RuntimeOptions::new().with_binary(binary_program);
    let runtime_context = RuntimeContext::new(runtime_options).expect("Context construction must succeed");
    let mut virtual_machine =
        VirtualMachine::new_initialized(runtime_context).expect("VM initialization must succeed");

    // Invoking private entry point must fail
    let private_invocation_result = virtual_machine.invoke_entry("secret_function", &[]);
    assert!(matches!(
        private_invocation_result,
        Err(RuntimeError::InvalidEntryPointSignature { .. })
    ));

    // Invoking complex class parameter entry point must fail
    let complex_invocation_result =
        virtual_machine.invoke_entry("complex_param_function", &[Value::Null]);
    assert!(matches!(
        complex_invocation_result,
        Err(RuntimeError::InvalidEntryPointSignature { .. })
    ));
}

#[test]
fn test_load_time_duplicate_symbol_definition_rejected() {
    let mut primary_binary = BinaryProgram::new();
    primary_binary.package_name = "app".to_string();
    primary_binary.package_version = "1.0.0".to_string();

    let mut primary_function = CompiledFunction::new("shared_helper", 0, 0, 2);
    primary_function.is_public = true;
    primary_function
        .instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));
    primary_binary.functions.push(primary_function);

    let mut conflicting_binary = BinaryProgram::new();
    conflicting_binary.package_name = "plugin".to_string();
    conflicting_binary.package_version = "2.0.0".to_string();

    let mut conflicting_function = CompiledFunction::new("shared_helper", 0, 0, 2);
    conflicting_function.is_public = true;
    conflicting_function
        .instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));
    conflicting_binary.functions.push(conflicting_function);

    let runtime_options = RuntimeOptions::new()
        .with_binary(primary_binary)
        .with_binary(conflicting_binary);

    let context_result = RuntimeContext::new(runtime_options);
    match context_result {
        Err(RuntimeError::DuplicateSymbolDefinition {
            kind,
            symbol,
            first_module,
            second_module,
        }) => {
            assert_eq!(kind, "function");
            assert_eq!(symbol, "shared_helper");
            assert_eq!(first_module, "app@1.0.0");
            assert_eq!(second_module, "plugin@2.0.0");
        }
        other_result => panic!("Expected DuplicateSymbolDefinition, got: {other_result:?}"),
    }
}
