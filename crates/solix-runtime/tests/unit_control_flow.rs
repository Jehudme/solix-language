//! Unit tests for control flow, branching, loops, and relative jumps.

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
fn test_unconditional_jump() {
    let mut func = CompiledFunction::new("main", 0, 0, 4);
    // PC 0: r[0] = 1
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 1));
    // PC 1: Jump +2 (target PC 3)
    func.instructions
        .push(Instruction::format_c(OpCode::Jump, 0, 0, 2));
    // PC 2: r[0] = 999 (should be skipped!)
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 999));
    // PC 3: ReturnValue(r[0])
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 0, 0, 0));

    let res = run_single_function(func).expect("Execution failed");
    assert_eq!(res, Value::I32(1));
}

#[test]
fn test_conditional_jump_if_true_and_false() {
    // Test JumpIfTrue taking branch
    let mut func = CompiledFunction::new("main", 0, 0, 4);
    func.instructions
        .push(Instruction::format_b(OpCode::LoadBool, 0, 0, 1));
    // PC 1: JumpIfTrue(r[0], offset +2) -> jumps to PC 3
    func.instructions
        .push(Instruction::format_c(OpCode::JumpIfTrue, 0, 0, 2));
    // PC 2: r[1] = 100 (skipped)
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 100));
    // PC 3: r[1] = 200
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 200));
    // PC 4: ReturnValue(r[1])
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 1, 0, 0));

    let res = run_single_function(func).expect("Execution failed");
    assert_eq!(res, Value::I32(200));

    // Test JumpIfFalse not taking branch
    let mut func2 = CompiledFunction::new("main", 0, 0, 4);
    func2
        .instructions
        .push(Instruction::format_b(OpCode::LoadBool, 0, 0, 1)); // true
    // PC 1: JumpIfFalse(r[0], offset +2) -> condition is true, does NOT jump
    func2
        .instructions
        .push(Instruction::format_c(OpCode::JumpIfFalse, 0, 0, 2));
    // PC 2: r[1] = 500 (executed)
    func2
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 500));
    // PC 3: ReturnValue(r[1])
    func2
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 1, 0, 0));

    let res2 = run_single_function(func2).expect("Execution failed");
    assert_eq!(res2, Value::I32(500));
}

#[test]
fn test_while_loop_summation_1_to_10() {
    let mut func = CompiledFunction::new("main", 0, 0, 8);
    // PC 0: r[0] = 0 (sum)
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 0));
    // PC 1: r[1] = 1 (i)
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 1));
    // PC 2: r[2] = 10 (limit)
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 2, 10));
    // PC 3: r[4] = 1 (step)
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 4, 1));

    // PC 4: Header -> r[3] = r[1] <= r[2]
    func.instructions
        .push(Instruction::format_a(OpCode::LteI32, 0, 3, 1, 2));
    // PC 5: JumpIfFalse(r[3], offset +4) -> jumps to PC 9 (Exit)
    func.instructions
        .push(Instruction::format_c(OpCode::JumpIfFalse, 0, 3, 4));
    // PC 6: r[0] = r[0] + r[1] (sum += i)
    func.instructions
        .push(Instruction::format_a(OpCode::AddI32, 0, 0, 0, 1));
    // PC 7: r[1] = r[1] + r[4] (i += 1)
    func.instructions
        .push(Instruction::format_a(OpCode::AddI32, 0, 1, 1, 4));
    // PC 8: Jump(offset -4) -> jumps back to PC 4 (Header: 4 - 8 = -4)
    func.instructions
        .push(Instruction::format_c(OpCode::Jump, 0, 0, -4));

    // PC 9: Exit -> ReturnValue(r[0])
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 0, 0, 0));

    let res = run_single_function(func).expect("Execution failed");
    assert_eq!(res, Value::I32(55));
}

#[test]
fn test_nested_loops_multiplication_table() {
    let mut func = CompiledFunction::new("main", 0, 0, 10);
    // Outer loop: i = 1..3
    // Inner loop: j = 1..4
    // Accumulator: total += i * j
    // Expected total:
    // i=1: 1*1 + 1*2 + 1*3 + 1*4 = 10
    // i=2: 2*1 + 2*2 + 2*3 + 2*4 = 20
    // i=3: 3*1 + 3*2 + 3*3 + 3*4 = 30
    // Total = 60

    // PC 0: r[0] = 0 (total)
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 0));
    // PC 1: r[1] = 1 (i)
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 1));
    // PC 2: r[2] = 3 (i_max)
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 2, 3));
    // PC 3: r[3] = 4 (j_max)
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 3, 4));
    // PC 4: r[9] = 1 (const 1)
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 9, 1));

    // PC 5: Outer Header -> r[5] = r[1] <= r[2]
    func.instructions
        .push(Instruction::format_a(OpCode::LteI32, 0, 5, 1, 2));
    // PC 6: JumpIfFalse(r[5], offset +11) -> jumps to PC 17 (Exit)
    func.instructions
        .push(Instruction::format_c(OpCode::JumpIfFalse, 0, 5, 11));

    // PC 7: r[4] = 1 (j)
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 4, 1));

    // PC 8: Inner Header -> r[6] = r[4] <= r[3]
    func.instructions
        .push(Instruction::format_a(OpCode::LteI32, 0, 6, 4, 3));
    // PC 9: JumpIfFalse(r[6], offset +6) -> jumps to PC 15 (Inner Exit)
    func.instructions
        .push(Instruction::format_c(OpCode::JumpIfFalse, 0, 6, 6));

    // PC 10: r[7] = r[1] * r[4] (i * j)
    func.instructions
        .push(Instruction::format_a(OpCode::MulI32, 0, 7, 1, 4));
    // PC 11: r[0] = r[0] + r[7] (total += prod)
    func.instructions
        .push(Instruction::format_a(OpCode::AddI32, 0, 0, 0, 7));
    // PC 12: r[4] = r[4] + 1 (j += 1)
    func.instructions
        .push(Instruction::format_a(OpCode::AddI32, 0, 4, 4, 9));
    // PC 13: Jump(offset -5) -> back to PC 8 (Inner Header: 8 - 13 = -5)
    func.instructions
        .push(Instruction::format_c(OpCode::Jump, 0, 0, -5));

    // PC 14: Nop
    func.instructions
        .push(Instruction::format_a(OpCode::Nop, 0, 0, 0, 0));

    // PC 15: Inner Exit -> r[1] = r[1] + 1 (i += 1)
    func.instructions
        .push(Instruction::format_a(OpCode::AddI32, 0, 1, 1, 9));
    // PC 16: Jump(offset -11) -> back to PC 5 (Outer Header: 5 - 16 = -11)
    func.instructions
        .push(Instruction::format_c(OpCode::Jump, 0, 0, -11));

    // PC 17: Outer Exit -> ReturnValue(r[0])
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 0, 0, 0));

    let res = run_single_function(func).expect("Execution failed");
    assert_eq!(res, Value::I32(60));
}
