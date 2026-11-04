//! Unit tests for closures, captures, lambdas, and function pointers.

#![forbid(unsafe_code)]

use solix_common::bytecode::{BinaryProgram, CompiledFunction, Instruction, OpCode};
use solix_runtime::{RuntimeContext, RuntimeOptions, Value, VirtualMachine};

#[test]
fn test_call_callable_with_function_pointer() {
    let mut binary = BinaryProgram::new();

    // Function 0: double(x) -> x * 2
    let mut double_func = CompiledFunction::new("double", 0, 1, 4);
    // r[1] = 2
    double_func
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 2));
    // r[2] = r[0] * r[1]
    double_func
        .instructions
        .push(Instruction::format_a(OpCode::MulI32, 0, 2, 0, 1));
    // ReturnValue(r[2])
    double_func
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 2, 0, 0));

    // Function 1: main() -> calls FnPtr(0) with arg 21
    let mut main_func = CompiledFunction::new("main", 0, 0, 8);
    // r[0] will be initialized to FnPtr(0) externally; PC 0 is Nop
    main_func
        .instructions
        .push(Instruction::format_a(OpCode::Nop, 0, 0, 0, 0));
    // r[1] = 21 (argument)
    main_func
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 21));
    // r[2] = CallCallable(dst=2, r_callee=0, arg_base=1, count=1)
    main_func
        .instructions
        .push(Instruction::format_call(OpCode::CallCallable, 2, 0, 1, 1));
    // ReturnValue(r[2])
    main_func
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 2, 0, 0));

    binary.functions.push(double_func);
    binary.functions.push(main_func);
    binary.entry_point_id = Some(1);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    // Set r[0] to Value::FnPtr(0)
    // We execute with a pre-set function pointer or wrapper
    vm.push_frame(1, 0, &[]).unwrap();
    vm.set_reg(0, Value::FnPtr(0)).unwrap();

    while !vm.halted && !vm.call_frames.is_empty() {
        vm.step().unwrap();
    }

    assert_eq!(vm.last_return_value, Value::I32(42));
}

#[test]
fn test_pack_closure_and_unpack_captures() {
    let mut binary = BinaryProgram::new();

    // Function 0: lambda(x) -> base + x (base is captured)
    let mut lambda_func = CompiledFunction::new("lambda_func", 0, 1, 8);
    // PC 0: UnpackCaptures(capture_base=2, count=1) -> puts captured `base` into r[2]
    lambda_func
        .instructions
        .push(Instruction::format_call(OpCode::UnpackCaptures, 0, 0, 2, 1));
    // PC 1: r[3] = r[0] + r[2] (x + base)
    lambda_func
        .instructions
        .push(Instruction::format_a(OpCode::AddI32, 0, 3, 0, 2));
    // PC 2: ReturnValue(r[3])
    lambda_func
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 3, 0, 0));

    // Function 1: main() -> creates closure capturing base=100, calls closure(23)
    let mut main_func = CompiledFunction::new("main", 0, 0, 8);
    // PC 0: r[0] = 100 (base to capture)
    main_func
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 100));
    // PC 1: r[1] = PackClosure(target_lambda=0, cap_base=0, count=1)
    main_func
        .instructions
        .push(Instruction::format_call(OpCode::PackClosure, 1, 0, 0, 1));
    // PC 2: r[2] = 23 (argument to pass)
    main_func
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 2, 23));
    // PC 3: r[3] = CallCallable(dst=3, r_callee=1, arg_base=2, count=1)
    main_func
        .instructions
        .push(Instruction::format_call(OpCode::CallCallable, 3, 1, 2, 1));
    // PC 4: ReturnValue(r[3])
    main_func
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 3, 0, 0));

    binary.functions.push(lambda_func);
    binary.functions.push(main_func);
    binary.entry_point_id = Some(1);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let res = vm.run().expect("Execution failed");
    assert_eq!(res, Value::I32(123)); // 100 + 23 = 123
}
