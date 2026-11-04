//! Unit tests for zero-cost exception handling, Throw, and stack unwinding.

#![forbid(unsafe_code)]

use solix_common::bytecode::{
    BinaryProgram, CompiledFunction, ExceptionHandlerEntry, Instruction, OpCode,
};
use solix_runtime::{RuntimeContext, RuntimeError, RuntimeOptions, Value, VirtualMachine};

#[test]
fn test_zero_cost_exception_caught_locally() {
    let mut binary = BinaryProgram::new();
    let mut func = CompiledFunction::new("main", 0, 0, 8);

    // Try block: PC 0..2
    // PC 0: r[0] = 500 (exception value)
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 500));
    // PC 1: Throw(r[0])
    func.instructions
        .push(Instruction::format_a(OpCode::Throw, 0, 0, 0, 0));

    // Normal return if not thrown (PC 2)
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 0));
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 1, 0, 0));

    // Catch handler at PC 4:
    // Handler puts caught exception into r[2]
    // r[3] = r[2] + 1 (501)
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 4, 1));
    func.instructions
        .push(Instruction::format_a(OpCode::AddI32, 0, 3, 2, 4));
    // ReturnValue(r[3])
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 3, 0, 0));

    func.exception_handlers.push(ExceptionHandlerEntry {
        try_start_pc: 0,
        try_end_pc: 2,
        catch_type_symbol: None, // catch-all
        catch_reg: 2,
        handler_pc: 4,
        finally_pc: None,
    });

    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let res = vm.run().expect("Execution failed");
    assert_eq!(res, Value::I32(501));
}

#[test]
fn test_cross_frame_exception_unwinding() {
    let mut binary = BinaryProgram::new();

    // Function 0: callee() -> throws 404
    let mut callee = CompiledFunction::new("callee", 0, 0, 4);
    callee
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 404));
    callee
        .instructions
        .push(Instruction::format_a(OpCode::Throw, 0, 0, 0, 0));

    // Function 1: caller() -> calls callee() inside a try block
    let mut caller = CompiledFunction::new("caller", 0, 0, 8);
    // PC 0: CallDirect(callee=0, dst=1, arg_base=0, count=0)
    caller
        .instructions
        .push(Instruction::format_call(OpCode::CallDirect, 1, 0, 0, 0));
    // PC 1: Return (unreachable if thrown)
    caller
        .instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));

    // Handler at PC 2: catch_reg = 2
    caller
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 2, 0, 0));

    caller.exception_handlers.push(ExceptionHandlerEntry {
        try_start_pc: 0,
        try_end_pc: 2,
        catch_type_symbol: None,
        catch_reg: 2,
        handler_pc: 2,
        finally_pc: None,
    });

    binary.functions.push(callee);
    binary.functions.push(caller);
    binary.entry_point_id = Some(1);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let res = vm.run().expect("Execution failed");
    assert_eq!(res, Value::I32(404));
}

#[test]
fn test_unhandled_exception_halts_with_error() {
    let mut binary = BinaryProgram::new();

    let mut func = CompiledFunction::new("main", 0, 0, 4);
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 999));
    func.instructions
        .push(Instruction::format_a(OpCode::Throw, 0, 0, 0, 0));

    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let res = vm.run();
    assert!(matches!(res, Err(RuntimeError::UserException { .. })));
}
