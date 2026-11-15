//! Conformance tests for multi-entrypoint VirtualMachine lifecycle and duplicate symbol rejection.

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
    let mut binary = BinaryProgram::new();
    binary.package_name = "game_engine".to_string();
    binary.package_version = "1.0.0".to_string();
    binary.static_space_size = 2;
    binary
        .static_symbols_map
        .push(("game_engine::score".to_string(), 0));
    binary
        .static_symbols_map
        .push(("game_engine::ticks".to_string(), 1));

    // @init: sets static 0 to 100
    let mut init_func = CompiledFunction::new("game_engine::@init", 0, 0, 2);
    init_func.is_public = false;
    init_func
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 100));
    init_func
        .instructions
        .push(Instruction::format_b(OpCode::SetStaticDirect, 0, 0, 0));
    init_func
        .instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));
    binary.functions.push(init_func);

    // Entry 1: on_init() -> returns static 0 (score)
    let mut on_init_func = CompiledFunction::new("game_engine::on_init", 0, 0, 2);
    on_init_func.is_public = true;
    on_init_func.return_type = Type::Primitive(PrimitiveKind::Int32);
    on_init_func
        .instructions
        .push(Instruction::format_b(OpCode::GetStaticDirect, 0, 0, 0));
    on_init_func
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 0, 0, 0));
    binary.functions.push(on_init_func);

    // Entry 2: add_score(int32 points) -> score += points, returns new score
    let mut add_score_func = CompiledFunction::new("game_engine::add_score", 0, 1, 3);
    add_score_func.is_public = true;
    add_score_func.return_type = Type::Primitive(PrimitiveKind::Int32);
    add_score_func.parameter_types = vec![Type::Primitive(PrimitiveKind::Int32)];
    // r0 = param (points)
    // r1 = get_static 0
    add_score_func
        .instructions
        .push(Instruction::format_b(OpCode::GetStaticDirect, 0, 1, 0));
    // r1 = r1 + r0
    add_score_func
        .instructions
        .push(Instruction::format_a(OpCode::AddI32, 0, 1, 1, 0));
    // set_static 0 = r1
    add_score_func
        .instructions
        .push(Instruction::format_b(OpCode::SetStaticDirect, 0, 1, 0));
    // return r1
    add_score_func
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 1, 0, 0));
    binary.functions.push(add_score_func);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).expect("Context construction succeeds");
    let mut virtual_machine =
        VirtualMachine::new_initialized(context).expect("VM initialization succeeds");

    // Invoke entry 1: on_init()
    let init_result = virtual_machine
        .invoke_entry("game_engine::on_init", &[])
        .expect("on_init invocation succeeds");
    assert_eq!(init_result, Value::I32(100));

    // Invoke entry 2: add_score(50) -> should see score 100 + 50 = 150
    let add_first_result = virtual_machine
        .invoke_entry("game_engine::add_score", &[Value::I32(50)])
        .expect("add_score invocation succeeds");
    assert_eq!(add_first_result, Value::I32(150));

    // Invoke entry 2 again: add_score(25) -> should see score 150 + 25 = 175 (shared state preserved!)
    let add_second_result = virtual_machine
        .invoke_entry("game_engine::add_score", &[Value::I32(25)])
        .expect("second add_score invocation succeeds");
    assert_eq!(add_second_result, Value::I32(175));

    // Shutdown cleans up static state
    virtual_machine.shutdown().expect("Shutdown succeeds");
    assert!(virtual_machine.is_shut_down);
}

#[test]
fn test_non_public_or_non_primitive_entrypoint_rejected() {
    let mut binary = BinaryProgram::new();
    binary.package_name = "test_pkg".to_string();

    // Private entrypoint
    let mut private_func = CompiledFunction::new("test_pkg::private_entry", 0, 0, 1);
    private_func.is_public = false;
    private_func
        .instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));
    binary.functions.push(private_func);

    // Non-primitive parameter entrypoint
    let mut object_param_func = CompiledFunction::new("test_pkg::object_entry", 0, 1, 2);
    object_param_func.is_public = true;
    object_param_func.parameter_types = vec![Type::Class {
        symbol_name: "custom::Widget".to_string(),
        type_arguments: Vec::new(),
    }];
    object_param_func
        .instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));
    binary.functions.push(object_param_func);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).expect("Context construction succeeds");
    let mut virtual_machine = VirtualMachine::new(context);

    // Rejects private entrypoint
    let private_res = virtual_machine.invoke_entry("test_pkg::private_entry", &[]);
    assert!(matches!(
        private_res,
        Err(RuntimeError::InvalidEntryPointSignature { .. })
    ));

    // Rejects non-primitive parameter entrypoint
    let object_res =
        virtual_machine.invoke_entry("test_pkg::object_entry", &[Value::Ref(1234)]);
    assert!(matches!(
        object_res,
        Err(RuntimeError::InvalidEntryPointSignature { .. })
    ));
}

#[test]
fn test_load_time_duplicate_symbol_definition_rejected() {
    let mut binary_one = BinaryProgram::new();
    binary_one.package_name = "pkg_alpha".to_string();
    binary_one.package_version = "1.0.0".to_string();
    let mut func_one = CompiledFunction::new("collision::duplicate_fn", 0, 0, 1);
    func_one.is_public = true;
    func_one
        .instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));
    binary_one.functions.push(func_one);

    let mut binary_two = BinaryProgram::new();
    binary_two.package_name = "pkg_beta".to_string();
    binary_two.package_version = "2.0.0".to_string();
    let mut func_two = CompiledFunction::new("collision::duplicate_fn", 0, 0, 1);
    func_two.is_public = true;
    func_two
        .instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));
    binary_two.functions.push(func_two);

    let options = RuntimeOptions::new()
        .with_binary(binary_one)
        .with_binary(binary_two);

    let context_result = RuntimeContext::new(options);
    assert!(matches!(
        context_result,
        Err(RuntimeError::DuplicateSymbolDefinition {
            kind,
            symbol,
            first_module,
            second_module,
        }) if kind == "function"
            && symbol == "collision::duplicate_fn"
            && first_module == "pkg_alpha@1.0.0"
            && second_module == "pkg_beta@2.0.0"
    ));
}
