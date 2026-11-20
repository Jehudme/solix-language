//! Comprehensive tests for Category 1 superinstructions (0xE0..=0xE5):
//! - GetArrayJumpIfTrue (0xE0)
//! - AddF64Imm (0xE1)
//! - SubF64Imm (0xE2)
//! - MulF64Imm (0xE3)
//! - JumpIfLtF64 (0xE4)
//! - JumpIfLteF64 (0xE5)

#![forbid(unsafe_code)]

use solix_common::bytecode::{BinaryProgram, CompiledFunction, Instruction, OpCode};
use solix_runtime::{RuntimeContext, RuntimeOptions, Value, VirtualMachine};

#[test]
fn test_superinstruction_add_sub_mul_f64_imm() {
    let mut binary = BinaryProgram::new();
    let constant_index_step = binary.constant_pool.add_f64(2.5).expect("add f64 step");
    let constant_index_scale = binary.constant_pool.add_f64(3.0).expect("add f64 scale");

    let mut function = CompiledFunction::new("main", 0, 0, 8);
    // r[0] = LoadF64(constant_index_step) -> 2.5
    function.instructions.push(Instruction::format_b(
        OpCode::LoadF64,
        0,
        0,
        constant_index_step as u32,
    ));
    // r[1] = AddF64Imm(r[0], constant_index_step) -> 2.5 + 2.5 = 5.0
    function.instructions.push(Instruction::format_a(
        OpCode::AddF64Imm,
        0,
        1,
        0,
        constant_index_step,
    ));
    // r[2] = SubF64Imm(r[1], constant_index_step) -> 5.0 - 2.5 = 2.5
    function.instructions.push(Instruction::format_a(
        OpCode::SubF64Imm,
        0,
        2,
        1,
        constant_index_step,
    ));
    // r[3] = MulF64Imm(r[2], constant_index_scale) -> 2.5 * 3.0 = 7.5
    function.instructions.push(Instruction::format_a(
        OpCode::MulF64Imm,
        0,
        3,
        2,
        constant_index_scale,
    ));
    // ReturnValue(r[3])
    function
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 3, 0, 0));

    binary.functions.push(function);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).expect("initialize context");
    let mut virtual_machine = VirtualMachine::new(context);

    let execution_result = virtual_machine.run().expect("execute virtual machine");
    assert_eq!(execution_result, Value::F64(7.5));
}

#[test]
fn test_superinstruction_jump_if_lt_and_lte_f64_branch_taken() {
    let mut binary = BinaryProgram::new();
    let constant_smaller = binary.constant_pool.add_f64(1.25).expect("add smaller");
    let constant_larger = binary.constant_pool.add_f64(4.50).expect("add larger");

    let mut function = CompiledFunction::new("main", 0, 0, 8);
    // r[0] = 1.25, r[1] = 4.50
    function.instructions.push(Instruction::format_b(
        OpCode::LoadF64,
        0,
        0,
        constant_smaller as u32,
    ));
    function.instructions.push(Instruction::format_b(
        OpCode::LoadF64,
        0,
        1,
        constant_larger as u32,
    ));

    // PC 2: JumpIfLtF64 r[0], r[1], offset=+3 (target PC 5)
    function
        .instructions
        .push(Instruction::format_a(OpCode::JumpIfLtF64, 0, 0, 1, 3));

    // PC 3: fallthrough (not taken) -> return 0
    function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 2, 0));
    function
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 2, 0, 0));

    // PC 5: target -> return 42
    function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 3, 42));
    function
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 3, 0, 0));

    binary.functions.push(function);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).expect("initialize context");
    let mut virtual_machine = VirtualMachine::new(context);

    let execution_result = virtual_machine.run().expect("execute virtual machine");
    assert_eq!(execution_result, Value::I32(42));
}

#[test]
fn test_superinstruction_jump_if_lt_and_lte_f64_fallthrough() {
    let mut binary = BinaryProgram::new();
    let constant_smaller = binary.constant_pool.add_f64(1.25).expect("add smaller");
    let constant_larger = binary.constant_pool.add_f64(4.50).expect("add larger");

    let mut function = CompiledFunction::new("main", 0, 0, 8);
    // r[0] = 4.50, r[1] = 1.25
    function.instructions.push(Instruction::format_b(
        OpCode::LoadF64,
        0,
        0,
        constant_larger as u32,
    ));
    function.instructions.push(Instruction::format_b(
        OpCode::LoadF64,
        0,
        1,
        constant_smaller as u32,
    ));

    // PC 2: JumpIfLtF64 r[0], r[1], offset=+3 (target PC 5) -> 4.50 < 1.25 is false -> falls through
    function
        .instructions
        .push(Instruction::format_a(OpCode::JumpIfLtF64, 0, 0, 1, 3));

    // PC 3: fallthrough -> return 99
    function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 2, 99));
    function
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 2, 0, 0));

    // PC 5: target
    function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 3, 42));
    function
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 3, 0, 0));

    binary.functions.push(function);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).expect("initialize context");
    let mut virtual_machine = VirtualMachine::new(context);

    let execution_result = virtual_machine.run().expect("execute virtual machine");
    assert_eq!(execution_result, Value::I32(99));
}

#[test]
fn test_superinstruction_jump_if_lte_f64_equal_branch_taken() {
    let mut binary = BinaryProgram::new();
    let constant_value = binary.constant_pool.add_f64(3.25).expect("add f64");

    let mut function = CompiledFunction::new("main", 0, 0, 8);
    // r[0] = 3.25, r[1] = 3.25
    function.instructions.push(Instruction::format_b(
        OpCode::LoadF64,
        0,
        0,
        constant_value as u32,
    ));
    function.instructions.push(Instruction::format_b(
        OpCode::LoadF64,
        0,
        1,
        constant_value as u32,
    ));

    // PC 2: JumpIfLteF64 r[0], r[1], offset=+3 (target PC 5) -> equal, branch taken
    function
        .instructions
        .push(Instruction::format_a(OpCode::JumpIfLteF64, 0, 0, 1, 3));

    // PC 3: fallthrough
    function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 2, 10));
    function
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 2, 0, 0));

    // PC 5: target
    function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 3, 88));
    function
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 3, 0, 0));

    binary.functions.push(function);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).expect("initialize context");
    let mut virtual_machine = VirtualMachine::new(context);

    let execution_result = virtual_machine.run().expect("execute virtual machine");
    assert_eq!(execution_result, Value::I32(88));
}

#[test]
fn test_superinstruction_get_array_jump_if_true() {
    let mut binary = BinaryProgram::new();

    let mut function = CompiledFunction::new("main", 0, 0, 10);
    // r[0] = 2 (array length)
    function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 2));

    // r[1] = AllocArray(dst=1, len_reg=0, elem_size=8)
    function
        .instructions
        .push(Instruction::format_a(OpCode::AllocArray, 0, 1, 0, 8));

    // r[2] = 0 (index 0)
    function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 2, 0));

    // r[3] = 1 (index 1)
    function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 3, 1));

    // r[4] = 42 (non-zero value for index 1)
    function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 4, 42));

    // SetArray r[1][r[3]] = r[4] (arr=r1, idx=r3, val=r4)
    function
        .instructions
        .push(Instruction::format_a(OpCode::SetArray, 0, 1, 3, 4));

    // PC 6: GetArrayJumpIfTrue r[1], r[2] (index 0 is 0/falsy) -> offset=+4 -> falls through
    function.instructions.push(Instruction::format_a(
        OpCode::GetArrayJumpIfTrue,
        0,
        1,
        2,
        4,
    ));

    // PC 7: GetArrayJumpIfTrue r[1], r[3] (index 1 is 42/truthy) -> offset=+3 -> branch taken to PC 10
    function.instructions.push(Instruction::format_a(
        OpCode::GetArrayJumpIfTrue,
        0,
        1,
        3,
        3,
    ));

    // PC 8: fallthrough (unexpected)
    function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 5, 111));
    function
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 5, 0, 0));

    // PC 10: target reached via branch
    function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 6, 777));
    function
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 6, 0, 0));

    binary.functions.push(function);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).expect("initialize context");
    let mut virtual_machine = VirtualMachine::new(context);

    let execution_result = virtual_machine.run().expect("execute virtual machine");
    assert_eq!(execution_result, Value::I32(777));
}

#[test]
fn test_superinstruction_verifier_constant_index_bounds_rejection() {
    let mut binary = BinaryProgram::new();
    // Empty constant pool (len = 0)
    let mut function = CompiledFunction::new("main", 0, 0, 4);

    // PC 0: tries to reference constant pool index 99
    function
        .instructions
        .push(Instruction::format_a(OpCode::AddF64Imm, 0, 0, 0, 99));
    function
        .instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));

    binary.functions.push(function);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context_result = RuntimeContext::new(options);
    assert!(context_result.is_err());
}
