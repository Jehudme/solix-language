//! Unit tests for dynamic arrays, heterogeneous tuples, and tuple instanceof.

#![forbid(unsafe_code)]

use solix_common::bytecode::{BinaryProgram, CompiledFunction, Instruction, OpCode};
use solix_runtime::{RuntimeContext, RuntimeError, RuntimeOptions, Value, VirtualMachine};

#[test]
fn test_alloc_array_get_set_and_length() {
    let mut binary = BinaryProgram::new();
    let mut func = CompiledFunction::new("main", 0, 0, 12);

    // r[0] = 5 (len)
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 5));
    // r[1] = AllocArray(r_size=0, elem_size=8)
    func.instructions
        .push(Instruction::format_a(OpCode::AllocArray, 0, 1, 0, 8));
    // r[2] = ArrayLength(r[1])
    func.instructions
        .push(Instruction::format_a(OpCode::ArrayLength, 0, 2, 1, 0));

    // r[3] = index 2
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 3, 2));
    // r[4] = val 1234
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 4, 1234));
    // SetArray(arr=1, idx=3, val=4)
    func.instructions
        .push(Instruction::format_a(OpCode::SetArray, 0, 1, 3, 4));

    // r[5] = GetArray(arr=1, idx=3)
    func.instructions
        .push(Instruction::format_a(OpCode::GetArray, 0, 5, 1, 3));
    // r[6] = r[5] + r[2] (1234 + 5 = 1239)
    func.instructions
        .push(Instruction::format_a(OpCode::AddI32, 0, 6, 5, 2));
    // ReturnValue(r[6])
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 6, 0, 0));

    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let res = vm.run().expect("Execution failed");
    assert_eq!(res, Value::I32(1239));
}

#[test]
fn test_array_bounds_check_failures() {
    let mut binary = BinaryProgram::new();
    let mut func = CompiledFunction::new("main", 0, 0, 6);

    // r[0] = 3
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 3));
    // r[1] = AllocArray(3, 8)
    func.instructions
        .push(Instruction::format_a(OpCode::AllocArray, 0, 1, 0, 8));
    // r[2] = index 5 (out of bounds)
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 2, 5));
    // r[3] = GetArray(r[1], r[2]) -> throws IndexOutOfBounds
    func.instructions
        .push(Instruction::format_a(OpCode::GetArray, 0, 3, 1, 2));

    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let res = vm.run();
    assert!(matches!(res, Err(RuntimeError::IndexOutOfBounds { .. })));
}

#[test]
fn test_alloc_tuple_and_get_tuple() {
    let mut binary = BinaryProgram::new();
    let mut func = CompiledFunction::new("main", 0, 0, 12);

    // r[0] = AllocTuple(count=3)
    func.instructions
        .push(Instruction::format_a(OpCode::AllocTuple, 0, 0, 3, 0));

    // r[1] = idx 0, r[2] = 100
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 0));
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 2, 100));
    func.instructions
        .push(Instruction::format_a(OpCode::SetArray, 0, 0, 1, 2));

    // r[3] = idx 1, r[4] = true
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 3, 1));
    func.instructions
        .push(Instruction::format_b(OpCode::LoadBool, 0, 4, 1));
    func.instructions
        .push(Instruction::format_a(OpCode::SetArray, 0, 0, 3, 4));

    // r[5] = GetTuple(r[0], slot 0) (100)
    func.instructions
        .push(Instruction::format_a(OpCode::GetTuple, 0, 5, 0, 0));
    // r[6] = GetTuple(r[0], slot 1) (true)
    func.instructions
        .push(Instruction::format_a(OpCode::GetTuple, 0, 6, 0, 1));
    // ReturnValue(r[6])
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 6, 0, 0));

    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let res = vm.run().expect("Execution failed");
    assert_eq!(res, Value::Bool(true));
}

#[test]
fn test_tuple_instanceof_type_tag_inspection() {
    let mut binary = BinaryProgram::new();
    let mut func = CompiledFunction::new("main", 0, 0, 12);

    // r[0] = AllocTuple(2)
    func.instructions
        .push(Instruction::format_a(OpCode::AllocTuple, 0, 0, 2, 0));

    // slot 0: int32 (42)
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 0));
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 2, 42));
    func.instructions
        .push(Instruction::format_a(OpCode::SetArray, 0, 0, 1, 2));

    // slot 1: bool (true)
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 3, 1));
    func.instructions
        .push(Instruction::format_b(OpCode::LoadBool, 0, 4, 1));
    func.instructions
        .push(Instruction::format_a(OpCode::SetArray, 0, 0, 3, 4));

    // r[5] = TupleInstanceof(r[0], idx=r[1](0), type_tag=3 (int32)) -> true
    func.instructions
        .push(Instruction::format_a(OpCode::TupleInstanceof, 3, 5, 0, 1));
    // r[6] = TupleInstanceof(r[0], idx=r[1](0), type_tag=11 (bool)) -> false
    func.instructions
        .push(Instruction::format_a(OpCode::TupleInstanceof, 11, 6, 0, 1));
    // r[7] = TupleInstanceof(r[0], idx=r[3](1), type_tag=11 (bool)) -> true
    func.instructions
        .push(Instruction::format_a(OpCode::TupleInstanceof, 11, 7, 0, 3));

    // Combine: all_ok = r[5] && !r[6] && r[7]
    // r[8] = !r[6] (true)
    func.instructions
        .push(Instruction::format_a(OpCode::Not, 0, 8, 6, 0));
    // r[9] = r[5] == r[8] (true == true -> true)
    func.instructions
        .push(Instruction::format_a(OpCode::EqBool, 0, 9, 5, 8));
    // r[10] = r[9] == r[7] (true == true -> true)
    func.instructions
        .push(Instruction::format_a(OpCode::EqBool, 0, 10, 9, 7));
    // ReturnValue(r[10])
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 10, 0, 0));

    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let res = vm.run().expect("Execution failed");
    assert_eq!(res, Value::Bool(true));
}
