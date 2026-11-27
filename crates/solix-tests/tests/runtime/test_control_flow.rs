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

#[test]
fn test_for_loop_i32_lte_i32_max_terminates_without_wrap() {
    // Regression test (#300.6): inclusive (<=) ForLoopI32 ending at i32::MAX must not wrap to
    // i32::MIN and loop forever — both on the scalar-quickened unit-stride path and on main dispatch.
    fn build_max_loop(step_value: i32, start_offset_from_max: i32) -> (RuntimeContext, Value) {
        let mut func = CompiledFunction::new("main", 0, 0, 8);
        // PC 0: r[1] = 1
        func.instructions
            .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 1));
        // PC 1: r[3] = i32::MAX (bound)
        func.instructions.push(Instruction::format_b_signed(
            OpCode::LoadI32,
            0,
            3,
            i32::MAX,
        ));
        // PC 2: r[0] = i32::MAX - start_offset_from_max (idx)
        func.instructions.push(Instruction::format_b_signed(
            OpCode::LoadI32,
            0,
            0,
            i32::MAX - start_offset_from_max,
        ));
        // PC 3: r[5] = 0 (iteration counter)
        func.instructions
            .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 5, 0));
        // PC 4: r[6] = step_value
        func.instructions.push(Instruction::format_b_signed(
            OpCode::LoadI32,
            0,
            6,
            step_value,
        ));
        // PC 5: body (r[5] += 1)
        func.instructions
            .push(Instruction::format_a(OpCode::AddI32, 0, 5, 5, 1));
        // PC 6: ForLoopI32(dst=0, src1=bound r[3], offset=-1 -> PC 5)
        let loop_flags = if step_value == 1 {
            0x40
        } else {
            0x80 | 0x40 | 6
        };
        func.instructions.push(Instruction::format_a(
            OpCode::ForLoopI32,
            loop_flags,
            0,
            3,
            (-1_i16) as u16,
        ));
        // PC 7: ReturnValue(r[5])
        func.instructions
            .push(Instruction::format_a(OpCode::ReturnValue, 0, 5, 0, 0));

        let mut binary = BinaryProgram::new();
        binary.functions.push(func);
        binary.entry_point_id = Some(0);
        let context = RuntimeContext::new(RuntimeOptions::new().with_binary(binary))
            .expect("create runtime context");
        let mut vm = VirtualMachine::new(context.clone());
        let value = vm.run().expect("loop at i32::MAX must terminate");
        (context, value)
    }

    // Unit-stride (step=1): iterations MAX-3, MAX-2, MAX-1, MAX => 4 iterations, tagged 0x10.
    let (ctx_unit, val_unit) = build_max_loop(1, 3);
    assert_eq!(val_unit, Value::I32(4));
    assert_ne!(
        ctx_unit.get_function(0).unwrap().instructions[6].flags() & 0x10,
        0,
        "Pure i32 unit-stride loop should carry scalar quick-tag 0x10"
    );

    // Custom stride (step=2): iterations MAX-4, MAX-2, MAX => 3 iterations.
    let (_ctx_step, val_step) = build_max_loop(2, 4);
    assert_eq!(val_step, Value::I32(3));
}

#[test]
fn test_mark_scalar_for_loops_skips_f64_and_i64_compare_jumps() {
    // Regression test (#292.4): loops containing JumpIfLtF64 or JumpIfLtI64 must NOT be tagged
    // with scalar-i32 bit 0x10 in RuntimeContext::mark_scalar_for_loops.
    let mut binary = BinaryProgram::new();
    let small_f64 = binary.constant_pool.add_f64(1.0).expect("add f64");
    let big_f64 = binary.constant_pool.add_f64(2.0).expect("add f64");

    let mut func = CompiledFunction::new("main", 0, 0, 8);
    // r[0] = 0 (idx), r[1] = 2 (bound), r[2] = 0 (count), r[3] = 1.0, r[4] = 2.0, r[5] = 1
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 0));
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 2));
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 2, 0));
    func.instructions.push(Instruction::format_b(
        OpCode::LoadF64,
        0,
        3,
        small_f64 as u32,
    ));
    func.instructions
        .push(Instruction::format_b(OpCode::LoadF64, 0, 4, big_f64 as u32));
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 5, 1));
    // PC 6 (loop start): if r[3] < r[4] (1.0 < 2.0, true!) jump +2 -> PC 8
    func.instructions
        .push(Instruction::format_a(OpCode::JumpIfLtF64, 0, 3, 4, 2));
    // PC 7: skipped
    func.instructions
        .push(Instruction::format_a(OpCode::AddI32, 0, 2, 2, 5));
    // PC 8: r[2] += 1
    func.instructions
        .push(Instruction::format_a(OpCode::AddI32, 0, 2, 2, 5));
    // PC 9: ForLoopI32 dst=0, bound=1, offset=-3 -> PC 6 ((10 - 1) - 3 = 6)
    func.instructions.push(Instruction::format_a(
        OpCode::ForLoopI32,
        0,
        0,
        1,
        (-3_i16) as u16,
    ));
    // PC 10: ReturnValue(r[2])
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 2, 0, 0));

    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let context = RuntimeContext::new(RuntimeOptions::new().with_binary(binary))
        .expect("create runtime context");
    assert_eq!(
        context.get_function(0).unwrap().instructions[9].flags() & 0x10,
        0,
        "JumpIfLtF64 loop must not receive i32 scalar fast-path bit 0x10"
    );

    let mut vm = VirtualMachine::new(context);
    assert_eq!(vm.run().expect("run f64 branch loop"), Value::I32(2));
}
