//! Comprehensive unit tests for the Ahead-of-Time (AOT) Bytecode Verifier.

#![forbid(unsafe_code)]

use solix_common::bytecode::{BinaryProgram, CompiledFunction, Instruction, OpCode};
use solix_runtime::{RuntimeContext, RuntimeError, RuntimeOptions};

#[test]
fn test_verifier_valid_program_passes() {
    let mut binary = BinaryProgram::new();
    let mut func = CompiledFunction::new("main", 0, 0, 4);

    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 42));
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 0, 0, 0));

    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context_result = RuntimeContext::new(options);
    assert!(context_result.is_ok());
}

#[test]
fn test_verifier_rejects_destination_register_out_of_bounds() {
    let mut binary = BinaryProgram::new();
    // Function only allocates 2 registers: r[0], r[1]
    let mut func = CompiledFunction::new("main", 0, 0, 2);

    // PC 0: tries to write to r[5] (exceeds num_registers = 2)
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 5, 42));
    func.instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));

    binary.functions.push(func);

    let options = RuntimeOptions::new().with_binary(binary);
    let context_result = RuntimeContext::new(options);

    match context_result {
        Err(RuntimeError::BinaryLoadError { reason }) => {
            assert!(reason.contains("register index 5 exceeds allocated register count 2"));
        }
        other => panic!("Expected BinaryLoadError for out-of-bounds register, got: {other:?}"),
    }
}

#[test]
fn test_verifier_rejects_source_register_out_of_bounds() {
    let mut binary = BinaryProgram::new();
    let mut func = CompiledFunction::new("main", 0, 0, 3);

    // PC 0: r[0] = r[1] + r[9] (r[9] exceeds num_registers = 3)
    func.instructions
        .push(Instruction::format_a(OpCode::AddI32, 0, 0, 1, 9));
    func.instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));

    binary.functions.push(func);

    let options = RuntimeOptions::new().with_binary(binary);
    let context_result = RuntimeContext::new(options);

    match context_result {
        Err(RuntimeError::BinaryLoadError { reason }) => {
            assert!(reason.contains("register index 9 exceeds allocated register count 3"));
        }
        other => panic!("Expected BinaryLoadError, got: {other:?}"),
    }
}

#[test]
fn test_verifier_rejects_backward_jump_before_instruction_zero() {
    let mut binary = BinaryProgram::new();
    let mut func = CompiledFunction::new("main", 0, 0, 2);

    // PC 0: Jump backwards by -5 instructions (target = -5 < 0)
    func.instructions
        .push(Instruction::format_c(OpCode::Jump, 0, 0, -5));

    binary.functions.push(func);

    let options = RuntimeOptions::new().with_binary(binary);
    let context_result = RuntimeContext::new(options);

    match context_result {
        Err(RuntimeError::BinaryLoadError { reason }) => {
            assert!(reason.contains("branch target -5 is out of bounds"));
        }
        other => panic!("Expected BinaryLoadError for negative branch target, got: {other:?}"),
    }
}

#[test]
fn test_verifier_rejects_forward_jump_past_end_of_function() {
    let mut binary = BinaryProgram::new();
    let mut func = CompiledFunction::new("main", 0, 0, 2);

    // Function only has 2 instructions.
    // PC 0: Jump forward by +10 (target = 10 > 2)
    func.instructions
        .push(Instruction::format_c(OpCode::Jump, 0, 0, 10));
    func.instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));

    binary.functions.push(func);

    let options = RuntimeOptions::new().with_binary(binary);
    let context_result = RuntimeContext::new(options);

    match context_result {
        Err(RuntimeError::BinaryLoadError { reason }) => {
            assert!(reason.contains("branch target 10 is out of bounds"));
        }
        other => panic!("Expected BinaryLoadError for forward jump past end, got: {other:?}"),
    }
}

#[test]
fn test_verifier_rejects_superinstruction_jump_out_of_bounds() {
    let mut binary = BinaryProgram::new();
    let mut func = CompiledFunction::new("main", 0, 0, 2);

    // PC 0: JumpIfLtI32 r[0], r[1], offset=+50 (target 50 > 2)
    func.instructions
        .push(Instruction::format_a(OpCode::JumpIfLtI32, 0, 0, 1, 50));
    func.instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));

    binary.functions.push(func);

    let options = RuntimeOptions::new().with_binary(binary);
    let context_result = RuntimeContext::new(options);

    match context_result {
        Err(RuntimeError::BinaryLoadError { reason }) => {
            assert!(reason.contains("branch target 50 is out of bounds"));
        }
        other => panic!("Expected BinaryLoadError, got: {other:?}"),
    }
}

#[test]
fn test_verifier_rejects_call_argument_window_overflow() {
    let mut binary = BinaryProgram::new();
    // Function has 4 registers (0..=3)
    let mut func = CompiledFunction::new("main", 0, 0, 4);

    // CallDirect: dst=0, target_id=0, arg_base=2, arg_count=4 -> window [2..6] exceeds 4!
    func.instructions
        .push(Instruction::format_call(OpCode::CallDirect, 0, 0, 2, 4));
    func.instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));

    binary.functions.push(func);

    let options = RuntimeOptions::new().with_binary(binary);
    let context_result = RuntimeContext::new(options);

    match context_result {
        Err(RuntimeError::BinaryLoadError { reason }) => {
            assert!(
                reason.contains("call argument window [2..6] exceeds allocated register count 4")
            );
        }
        other => panic!("Expected BinaryLoadError for call argument window, got: {other:?}"),
    }
}

#[test]
fn test_verifier_rejects_static_slot_out_of_bounds() {
    let mut binary = BinaryProgram::new();
    binary.static_space_size = 2; // only slots 0 and 1
    let mut func = CompiledFunction::new("main", 0, 0, 2);

    // GetStaticDirect accessing slot 10 (exceeds total static space 2)
    func.instructions
        .push(Instruction::format_b(OpCode::GetStaticDirect, 0, 0, 10));
    func.instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));

    binary.functions.push(func);

    let options = RuntimeOptions::new().with_binary(binary);
    let context_result = RuntimeContext::new(options);

    match context_result {
        Err(RuntimeError::BinaryLoadError { reason }) => {
            assert!(reason.contains("static slot 10 exceeds total static space 2"));
        }
        other => panic!("Expected BinaryLoadError for static slot out of bounds, got: {other:?}"),
    }
}
