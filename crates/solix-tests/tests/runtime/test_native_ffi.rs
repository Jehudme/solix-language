//! Unit tests for host native FFI callbacks and standard builtins.

#![forbid(unsafe_code)]

use solix_common::bytecode::{BinaryProgram, CompiledFunction, Instruction, OpCode};
use solix_runtime::{RuntimeContext, RuntimeError, RuntimeOptions, Value, VirtualMachine};

fn custom_host_multiply(_vm: &mut VirtualMachine, args: &[Value]) -> Result<Value, RuntimeError> {
    let a = args[0].as_i32()?;
    let b = args[1].as_i32()?;
    Ok(Value::I32(a * b))
}

#[test]
fn test_register_and_invoke_custom_native_host_callback() {
    let mut binary = BinaryProgram::new();
    let sym_idx = binary.constant_pool.add_string("host_mult").unwrap();

    let mut main_func = CompiledFunction::new("main", 0, 0, 8);
    // r[0] = 6
    main_func
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 6));
    // r[1] = 7
    main_func
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 7));
    // r[2] = CallSymbol("host_mult", dst=2, arg_base=0, count=2)
    main_func.instructions.push(Instruction::format_call(
        OpCode::CallSymbol,
        2,
        sym_idx,
        0,
        2,
    ));
    // ReturnValue(r[2])
    main_func
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 2, 0, 0));

    binary.functions.push(main_func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    // Register custom Rust callback
    vm.native_registry
        .register("host_mult", custom_host_multiply);

    let res = vm.run().expect("Execution failed");
    assert_eq!(res, Value::I32(42));
}

#[test]
fn test_builtin_assert_success_and_failure() {
    let mut binary = BinaryProgram::new();
    let sym_assert = binary.constant_pool.add_string("assert").unwrap();

    let mut func = CompiledFunction::new("main", 0, 0, 4);
    // r[0] = false
    func.instructions
        .push(Instruction::format_b(OpCode::LoadBool, 0, 0, 0));
    // CallSymbol("assert", dst=1, arg_base=0, count=1)
    func.instructions.push(Instruction::format_call(
        OpCode::CallSymbol,
        1,
        sym_assert,
        0,
        1,
    ));

    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let res = vm.run();
    assert!(matches!(res, Err(RuntimeError::AssertionFailed { .. })));
}

#[test]
fn test_builtin_clock() {
    let mut binary = BinaryProgram::new();
    let sym_clock = binary.constant_pool.add_string("clock").unwrap();

    let mut func = CompiledFunction::new("main", 0, 0, 4);
    // r[0] = CallSymbol("clock", dst=0, arg_base=0, count=0)
    func.instructions.push(Instruction::format_call(
        OpCode::CallSymbol,
        0,
        sym_clock,
        0,
        0,
    ));
    // ReturnValue(r[0])
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 0, 0, 0));

    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let res = vm.run().expect("Execution failed");
    let timestamp = res.as_i64().expect("Expected i64 from clock");
    assert!(timestamp > 1_000_000_000_000); // Greater than year 2001 in epoch millis
}
