//! Unit tests for arithmetic, bitwise, comparison, and conversion opcodes.

#![forbid(unsafe_code)]

use solix_common::bytecode::{BinaryProgram, CompiledFunction, Instruction, OpCode};
use solix_runtime::{RuntimeContext, RuntimeError, RuntimeOptions, Value, VirtualMachine};

fn run_single_function(func: CompiledFunction) -> Result<Value, RuntimeError> {
    let mut binary = BinaryProgram::new();
    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options)?;
    let mut vm = VirtualMachine::new(context);
    vm.run()
}

#[test]
fn test_i32_arithmetic_and_wrapping() {
    let mut func = CompiledFunction::new("main", 0, 0, 8);
    // r[0] = 100, r[1] = 40
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 100));
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 40));
    // r[2] = r[0] + r[1] (140)
    func.instructions
        .push(Instruction::format_a(OpCode::AddI32, 0, 2, 0, 1));
    // r[3] = r[0] - r[1] (60)
    func.instructions
        .push(Instruction::format_a(OpCode::SubI32, 0, 3, 0, 1));
    // r[4] = r[2] * r[3] (8400)
    func.instructions
        .push(Instruction::format_a(OpCode::MulI32, 0, 4, 2, 3));
    // r[5] = -r[4] (-8400)
    func.instructions
        .push(Instruction::format_a(OpCode::NegI32, 0, 5, 4, 0));
    // ReturnValue(r[5])
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 5, 0, 0));

    let res = run_single_function(func).expect("Execution failed");
    assert_eq!(res, Value::I32(-8400));
}

#[test]
fn test_i32_overflow_wrapping() {
    let mut func = CompiledFunction::new("main", 0, 0, 4);
    // r[0] = i32::MAX (0x7FFFFFFF)
    func.instructions.push(Instruction::format_b_signed(
        OpCode::LoadI32,
        0,
        0,
        i32::MAX,
    ));
    // r[1] = 1
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 1));
    // r[2] = r[0] + r[1] (wrapping into i32::MIN)
    func.instructions
        .push(Instruction::format_a(OpCode::AddI32, 0, 2, 0, 1));
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 2, 0, 0));

    let res = run_single_function(func).expect("Execution failed");
    assert_eq!(res, Value::I32(i32::MIN));
}

#[test]
fn test_division_and_modulo_success() {
    let mut func = CompiledFunction::new("main", 0, 0, 6);
    // r[0] = 53, r[1] = 10
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 53));
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 10));
    // r[2] = r[0] / r[1] (5)
    func.instructions
        .push(Instruction::format_a(OpCode::DivI32, 0, 2, 0, 1));
    // r[3] = r[0] % r[1] (3)
    func.instructions
        .push(Instruction::format_a(OpCode::ModI32, 0, 3, 0, 1));
    // r[4] = r[2] + r[3] (8)
    func.instructions
        .push(Instruction::format_a(OpCode::AddI32, 0, 4, 2, 3));
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 4, 0, 0));

    let res = run_single_function(func).expect("Execution failed");
    assert_eq!(res, Value::I32(8));
}

#[test]
fn test_divide_by_zero_triggers_runtime_error() {
    let mut func = CompiledFunction::new("main", 0, 0, 4);
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 10));
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 0));
    func.instructions
        .push(Instruction::format_a(OpCode::DivI32, 0, 2, 0, 1));
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 2, 0, 0));

    let res = run_single_function(func);
    assert_eq!(res, Err(RuntimeError::DivideByZero));
}

#[test]
fn test_modulo_by_zero_triggers_runtime_error() {
    let mut func = CompiledFunction::new("main", 0, 0, 4);
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI64, 0, 0, 42));
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI64, 0, 1, 0));
    func.instructions
        .push(Instruction::format_a(OpCode::ModI64, 0, 2, 0, 1));
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 2, 0, 0));

    let res = run_single_function(func);
    assert_eq!(res, Err(RuntimeError::DivideByZero));
}

#[test]
fn test_bitwise_operations() {
    let mut func = CompiledFunction::new("main", 0, 0, 8);
    // r[0] = 0b1100 (12), r[1] = 0b1010 (10)
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI64, 0, 0, 12));
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI64, 0, 1, 10));
    // r[2] = r[0] & r[1] (0b1000 = 8)
    func.instructions
        .push(Instruction::format_a(OpCode::Band, 0, 2, 0, 1));
    // r[3] = r[0] | r[1] (0b1110 = 14)
    func.instructions
        .push(Instruction::format_a(OpCode::Bor, 0, 3, 0, 1));
    // r[4] = r[0] ^ r[1] (0b0110 = 6)
    func.instructions
        .push(Instruction::format_a(OpCode::Bxor, 0, 4, 0, 1));
    // r[5] = r[0] << 2 (48)
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI64, 0, 6, 2));
    func.instructions
        .push(Instruction::format_a(OpCode::Shl, 0, 5, 0, 6));
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 5, 0, 0));

    let res = run_single_function(func).expect("Execution failed");
    assert_eq!(res, Value::I64(48));
}

#[test]
fn test_comparisons_and_boolean_logic() {
    let mut func = CompiledFunction::new("main", 0, 0, 8);
    // r[0] = 10, r[1] = 20
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 10));
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 20));
    // r[2] = r[0] < r[1] (true)
    func.instructions
        .push(Instruction::format_a(OpCode::LtI32, 0, 2, 0, 1));
    // r[3] = r[0] >= r[1] (false)
    func.instructions
        .push(Instruction::format_a(OpCode::GteI32, 0, 3, 0, 1));
    // r[4] = !r[3] (true)
    func.instructions
        .push(Instruction::format_a(OpCode::Not, 0, 4, 3, 0));
    // r[5] = r[2] == r[4] (true == true -> true)
    func.instructions
        .push(Instruction::format_a(OpCode::EqBool, 0, 5, 2, 4));
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 5, 0, 0));

    let res = run_single_function(func).expect("Execution failed");
    assert_eq!(res, Value::Bool(true));
}

#[test]
fn test_numeric_type_conversions() {
    let mut func = CompiledFunction::new("main", 0, 0, 4);
    // r[0] = 50
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI64, 0, 0, 50));
    // r[1] = ConvIToF(r[0]) -> 50.0
    func.instructions
        .push(Instruction::format_a(OpCode::ConvIToF, 0, 1, 0, 0));
    // r[2] = ConvFToI(r[1]) -> 50
    func.instructions
        .push(Instruction::format_a(OpCode::ConvFToI, 0, 2, 1, 0));
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 2, 0, 0));

    let res = run_single_function(func).expect("Execution failed");
    assert_eq!(res, Value::I64(50));
}

#[test]
fn test_eq_string_heap_comparison() {
    let mut binary = BinaryProgram::new();
    let idx1 = binary.constant_pool.add_string("foobar").unwrap();
    let idx2 = binary.constant_pool.add_string("foobar").unwrap();
    let idx3 = binary.constant_pool.add_string("baz").unwrap();

    let mut func = CompiledFunction::new("main", 0, 0, 6);
    // r[0] = LoadConst(idx1)
    func.instructions
        .push(Instruction::format_b(OpCode::LoadConst, 0, 0, idx1 as u32));
    // r[1] = LoadConst(idx2)
    func.instructions
        .push(Instruction::format_b(OpCode::LoadConst, 0, 1, idx2 as u32));
    // r[2] = LoadConst(idx3)
    func.instructions
        .push(Instruction::format_b(OpCode::LoadConst, 0, 2, idx3 as u32));
    // r[3] = EqString(r[0], r[1]) -> true
    func.instructions
        .push(Instruction::format_a(OpCode::EqString, 0, 3, 0, 1));
    // r[4] = NeqString(r[0], r[2]) -> true
    func.instructions
        .push(Instruction::format_a(OpCode::NeqString, 0, 4, 0, 2));
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 3, 0, 0));

    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let res = vm.run().expect("Execution failed");
    assert_eq!(res, Value::Bool(true));
}
