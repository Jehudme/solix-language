//! Unit tests for function calls, returns, recursion, static @init, and multi-binary linking.

#![forbid(unsafe_code)]

use solix_common::bytecode::{BinaryProgram, CompiledFunction, Instruction, OpCode};
use solix_runtime::{RuntimeContext, RuntimeError, RuntimeOptions, Value, VirtualMachine};

#[test]
fn test_call_direct_passing_arguments_and_returning_value() {
    let mut binary = BinaryProgram::new();

    // Function 0: add(a, b) -> a + b
    let mut add_func = CompiledFunction::new("add", 0, 2, 4);
    // r[2] = r[0] + r[1]
    add_func
        .instructions
        .push(Instruction::format_a(OpCode::AddI32, 0, 2, 0, 1));
    // ReturnValue(r[2])
    add_func
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 2, 0, 0));

    // Function 1: main() -> calls add(15, 27)
    let mut main_func = CompiledFunction::new("main", 0, 0, 8);
    // r[0] = 15
    main_func
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 15));
    // r[1] = 27
    main_func
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 27));
    // r[2] = CallDirect(target_id=0, dst=2, arg_base=0, arg_count=2)
    main_func
        .instructions
        .push(Instruction::format_call(OpCode::CallDirect, 2, 0, 0, 2));
    // ReturnValue(r[2])
    main_func
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 2, 0, 0));

    binary.functions.push(add_func);
    binary.functions.push(main_func);
    binary.entry_point_id = Some(1);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let res = vm.run().expect("Execution failed");
    assert_eq!(res, Value::I32(42));
    assert!(vm.verify_clean_memory().is_ok());
}

#[test]
fn test_recursive_factorial() {
    let mut binary = BinaryProgram::new();

    // Function 0: fact(n) -> if n <= 1 return 1 else return n * fact(n - 1)
    let mut fact_func = CompiledFunction::new("fact", 0, 1, 8);
    // r[0] is n
    // r[1] = 1 (const 1)
    fact_func
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 1));
    // r[2] = r[0] <= 1
    fact_func
        .instructions
        .push(Instruction::format_a(OpCode::LteI32, 0, 2, 0, 1));
    // JumpIfFalse(r[2], offset +2) -> to PC 4 (recursive branch)
    fact_func
        .instructions
        .push(Instruction::format_c(OpCode::JumpIfFalse, 0, 2, 2));
    // Base branch: ReturnValue(1)
    fact_func
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 1, 0, 0));

    // Recursive branch at PC 4:
    // r[3] = r[0] - 1 (n - 1)
    fact_func
        .instructions
        .push(Instruction::format_a(OpCode::SubI32, 0, 3, 0, 1));
    // r[4] = CallDirect(fact, dst=4, arg_base=3, count=1)
    fact_func
        .instructions
        .push(Instruction::format_call(OpCode::CallDirect, 4, 0, 3, 1));
    // r[5] = r[0] * r[4] (n * fact(n - 1))
    fact_func
        .instructions
        .push(Instruction::format_a(OpCode::MulI32, 0, 5, 0, 4));
    // ReturnValue(r[5])
    fact_func
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 5, 0, 0));

    // Function 1: main() -> fact(5)
    let mut main_func = CompiledFunction::new("main", 0, 0, 4);
    // r[0] = 5
    main_func
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 5));
    // r[1] = CallDirect(fact, dst=1, arg_base=0, count=1)
    main_func
        .instructions
        .push(Instruction::format_call(OpCode::CallDirect, 1, 0, 0, 1));
    // ReturnValue(r[1])
    main_func
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 1, 0, 0));

    binary.functions.push(fact_func);
    binary.functions.push(main_func);
    binary.entry_point_id = Some(1);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let res = vm.run().expect("Execution failed");
    assert_eq!(res, Value::I32(120)); // 5! = 120
    assert!(vm.verify_clean_memory().is_ok());
}

#[test]
fn test_stack_overflow_detection() {
    let mut binary = BinaryProgram::new();

    // Infinite recursion: func() calls func()
    let mut inf_func = CompiledFunction::new("inf", 0, 0, 2);
    inf_func
        .instructions
        .push(Instruction::format_call(OpCode::CallDirect, 0, 0, 0, 0));
    inf_func
        .instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));

    binary.functions.push(inf_func);
    binary.entry_point_id = Some(0);

    // Set a tiny max call stack depth of 16
    let options = RuntimeOptions::new()
        .with_binary(binary)
        .with_max_call_stack_depth(16);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let res = vm.run();
    assert!(matches!(res, Err(RuntimeError::StackOverflow { .. })));
}

#[test]
fn test_two_phase_static_initialization() {
    let mut binary = BinaryProgram::new();
    binary.static_space_size = 1;

    // Function 0: @init -> initializes static slot 0 with 777
    let mut init_func = CompiledFunction::new("pkg::@init", 0, 0, 2);
    init_func
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 777));
    init_func
        .instructions
        .push(Instruction::format_b(OpCode::SetStatic, 0, 0, 0));
    init_func
        .instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));

    // Function 1: main -> reads static slot 0 without setting it
    let mut main_func = CompiledFunction::new("main", 0, 0, 2);
    main_func
        .instructions
        .push(Instruction::format_b(OpCode::GetStatic, 0, 0, 0));
    main_func
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 0, 0, 0));

    binary.functions.push(init_func);
    binary.functions.push(main_func);
    binary.entry_point_id = Some(1);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let res = vm.run().expect("Execution failed");
    assert_eq!(res, Value::I32(777));
    assert!(vm.verify_clean_memory().is_ok());
}

#[test]
fn test_multi_binary_cross_module_call_and_relocation() {
    // Binary 1: Library with static variable and exported function
    let mut bin1 = BinaryProgram::new();
    bin1.static_space_size = 1;
    let mut lib_func = CompiledFunction::new("lib::compute", 0, 1, 4);
    // r[1] = 50
    lib_func
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 50));
    // r[2] = r[0] + r[1]
    lib_func
        .instructions
        .push(Instruction::format_a(OpCode::AddI32, 0, 2, 0, 1));
    // ReturnValue(r[2])
    lib_func
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 2, 0, 0));
    bin1.functions.push(lib_func);

    // Binary 2: Executable calling "lib::compute" via CallSymbol
    let mut bin2 = BinaryProgram::new();
    let sym_idx = bin2.constant_pool.add_string("lib::compute").unwrap();
    let mut main_func = CompiledFunction::new("main", 0, 0, 4);
    // r[0] = 25
    main_func
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 25));
    // r[1] = CallSymbol("lib::compute", dst=1, arg_base=0, count=1)
    main_func.instructions.push(Instruction::format_call(
        OpCode::CallSymbol,
        1,
        sym_idx,
        0,
        1,
    ));
    // ReturnValue(r[1])
    main_func
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 1, 0, 0));
    bin2.functions.push(main_func);
    bin2.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(bin1).with_binary(bin2);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let res = vm.run().expect("Execution failed");
    assert_eq!(res, Value::I32(75)); // 25 + 50 = 75
    assert!(vm.verify_clean_memory().is_ok());
}
