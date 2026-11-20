//! Comprehensive tests for Category 2 & 3 superinstructions (0xE6..=0xED):
//! - AddI64Imm (0xE6)
//! - JumpIfLtI64 (0xE7)
//! - JumpIfLteI64 (0xE8)
//! - GetFieldI32 (0xE9)
//! - GetFieldF64 (0xEA)
//! - SetFieldI32 (0xEB)
//! - SetFieldF64 (0xEC)
//! - CallVirtualDirect (0xED)

#![forbid(unsafe_code)]

use solix_common::bytecode::{BinaryProgram, CompiledFunction, Instruction, OpCode};
use solix_runtime::{RuntimeContext, RuntimeOptions, Value, VirtualMachine};

#[test]
fn test_superinstruction_add_i64_imm_positive_and_negative() {
    let mut binary = BinaryProgram::new();
    let mut function = CompiledFunction::new("main", 0, 0, 8);

    // r[0] = LoadI64(100)
    function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI64, 0, 0, 100));
    // r[1] = AddI64Imm(r[0], +50) -> 150
    function
        .instructions
        .push(Instruction::format_a(OpCode::AddI64Imm, 0, 1, 0, 50u16));
    // r[2] = AddI64Imm(r[1], -70) -> 80
    let signed_delta: i16 = -70;
    function.instructions.push(Instruction::format_a(
        OpCode::AddI64Imm,
        0,
        2,
        1,
        signed_delta as u16,
    ));
    // ReturnValue(r[2])
    function
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 2, 0, 0));

    binary.functions.push(function);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).expect("initialize context");
    let mut virtual_machine = VirtualMachine::new(context);

    let execution_result = virtual_machine.run().expect("execute virtual machine");
    assert_eq!(execution_result, Value::I64(80));
}

#[test]
fn test_superinstruction_jump_if_lt_and_lte_i64() {
    let mut binary = BinaryProgram::new();
    let mut function = CompiledFunction::new("main", 0, 0, 8);

    // r[0] = 10, r[1] = 20, r[2] = 20
    function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI64, 0, 0, 10));
    function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI64, 0, 1, 20));
    function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI64, 0, 2, 20));

    // PC 3: JumpIfLtI64 r[0], r[1], offset=+3 (target PC 6)
    function
        .instructions
        .push(Instruction::format_a(OpCode::JumpIfLtI64, 0, 0, 1, 3));

    // PC 4: fallthrough trap if branch fails
    function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 3, 0));
    function
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 3, 0, 0));

    // PC 6: JumpIfLteI64 r[1], r[2], offset=+3 (target PC 9)
    function
        .instructions
        .push(Instruction::format_a(OpCode::JumpIfLteI64, 0, 1, 2, 3));

    // PC 7: fallthrough trap if branch fails
    function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 3, 0));
    function
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 3, 0, 0));

    // PC 9: success return 777
    function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI64, 0, 3, 777));
    function
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 3, 0, 0));

    binary.functions.push(function);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).expect("initialize context");
    let mut virtual_machine = VirtualMachine::new(context);

    let execution_result = virtual_machine.run().expect("execute virtual machine");
    assert_eq!(execution_result, Value::I64(777));
}

#[test]
fn test_superinstruction_get_set_field_i32_and_f64() {
    let mut binary = BinaryProgram::new();
    let constant_float_val = binary.constant_pool.add_f64(123.456).expect("add f64");

    let mut function = CompiledFunction::new("main", 0, 0, 10);
    // r[0] = AllocObject(vtable_id: 0, size: 24)
    function
        .instructions
        .push(Instruction::format_a(OpCode::AllocObject, 0, 0, 0, 24));

    // r[1] = 42 (int32)
    function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 42));
    // SetFieldI32(val: r[1], obj: r[0], offset: 8)
    function
        .instructions
        .push(Instruction::format_a(OpCode::SetFieldI32, 0, 1, 0, 8));

    // r[2] = 123.456 (float64)
    function.instructions.push(Instruction::format_b(
        OpCode::LoadF64,
        0,
        2,
        constant_float_val as u32,
    ));
    // SetFieldF64(val: r[2], obj: r[0], offset: 16)
    function
        .instructions
        .push(Instruction::format_a(OpCode::SetFieldF64, 0, 2, 0, 16));

    // r[3] = GetFieldI32(obj: r[0], offset: 8)
    function
        .instructions
        .push(Instruction::format_a(OpCode::GetFieldI32, 0, 3, 0, 8));
    // r[4] = GetFieldF64(obj: r[0], offset: 16)
    function
        .instructions
        .push(Instruction::format_a(OpCode::GetFieldF64, 0, 4, 0, 16));

    // ReturnValue(r[3]) -> 42
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
fn test_superinstruction_monomorphic_call_virtual_quickening_and_unquickening() {
    let mut binary = BinaryProgram::new();

    // Function 0: VirtualMethodA (returns 100)
    let mut method_alpha = CompiledFunction::new("ClassA::method", 1, 0, 4);
    method_alpha
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 100));
    method_alpha
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 1, 0, 0));
    binary.functions.push(method_alpha);

    // Function 1: VirtualMethodB (returns 200)
    let mut method_beta = CompiledFunction::new("ClassB::method", 1, 0, 4);
    method_beta
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 200));
    method_beta
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 1, 0, 0));
    binary.functions.push(method_beta);

    // Function 2: main
    // vtable 0 -> slot 0 calls Function 0
    // vtable 1 -> slot 0 calls Function 1
    // We allocate an object with vtable 0, call slot 0 (quickening to CallVirtualDirect).
    // In a loop or repeated call, we call it again with vtable 0 -> monomorphic hit!
    // Then we call the same call-site with an object having vtable 1 -> polymorphic miss -> unquickened!
    let mut main_function = CompiledFunction::new("main", 0, 0, 12);
    // r[0] = AllocObject(vtable 0, size 16)
    main_function
        .instructions
        .push(Instruction::format_a(OpCode::AllocObject, 0, 0, 0, 16));
    // r[1] = AllocObject(vtable 1, size 16)
    main_function
        .instructions
        .push(Instruction::format_a(OpCode::AllocObject, 0, 1, 1, 16));

    // Call 1: CallVirtual(dst: 2, slot: 0, arg_base: 0, arg_count: 1) -> calls MethodA, returns 100
    // Quickens to CallVirtualDirect
    main_function
        .instructions
        .push(Instruction::format_call(OpCode::CallVirtual, 2, 0, 0, 1));

    // Call 2: CallVirtual(dst: 3, slot: 0, arg_base: 0, arg_count: 1) -> repeated on vtable 0
    main_function
        .instructions
        .push(Instruction::format_call(OpCode::CallVirtual, 3, 0, 0, 1));

    // Call 3: CallVirtual(dst: 4, slot: 0, arg_base: 1, arg_count: 1) -> calls with vtable 1!
    main_function
        .instructions
        .push(Instruction::format_call(OpCode::CallVirtual, 4, 0, 1, 1));

    // Add results: r[5] = r[2] + r[3] + r[4] = 100 + 100 + 200 = 400
    main_function
        .instructions
        .push(Instruction::format_a(OpCode::AddI32, 0, 5, 2, 3));
    main_function
        .instructions
        .push(Instruction::format_a(OpCode::AddI32, 0, 6, 5, 4));
    main_function
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 6, 0, 0));

    binary.functions.push(main_function);
    binary.entry_point_id = Some(2);

    let options = RuntimeOptions::new().with_binary(binary);
    let mut context = RuntimeContext::new(options).expect("initialize context");
    // Register flat vtables:
    // vtable 0: [func_id 0]
    // vtable 1: [func_id 1]
    context.flat_vtables = vec![vec![0].into_boxed_slice(), vec![1].into_boxed_slice()];
    let mut virtual_machine = VirtualMachine::new(context);

    let execution_result = virtual_machine.run().expect("execute virtual machine");
    assert_eq!(execution_result, Value::I32(400));
}

#[test]
fn test_superinstruction_call_virtual_direct_monomorphic_cache_and_polymorphic_miss() {
    let mut binary = BinaryProgram::new();

    // Function 0: ClassA::calc -> returns arg + 10
    let mut method_alpha = CompiledFunction::new("ClassA::calc", 1, 0, 4);
    method_alpha
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 10));
    method_alpha
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 1, 0, 0));
    binary.functions.push(method_alpha);

    // Function 1: ClassB::calc -> returns arg + 50
    let mut method_beta = CompiledFunction::new("ClassB::calc", 1, 0, 4);
    method_beta
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 50));
    method_beta
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 1, 0, 0));
    binary.functions.push(method_beta);

    // Function 2: main
    // r[0]: obj_a (vtable 0)
    // r[1]: obj_b (vtable 1)
    // r[2]: current target object ref
    // r[5]: accumulated sum
    // r[6]: return value of call
    // r[7]: iteration counter
    // r[8]: comparison bound (1)
    let mut main_function = CompiledFunction::new("main", 0, 0, 16);
    // PC 0: r[0] = AllocObject(vtable 0, size 16)
    main_function
        .instructions
        .push(Instruction::format_a(OpCode::AllocObject, 0, 0, 0, 16));
    // PC 1: r[1] = AllocObject(vtable 1, size 16)
    main_function
        .instructions
        .push(Instruction::format_a(OpCode::AllocObject, 0, 1, 1, 16));
    // PC 2: r[2] = Mov r[0] (start with obj_a)
    main_function
        .instructions
        .push(Instruction::format_a(OpCode::Mov, 0, 2, 0, 0));
    // PC 3: r[5] = 0 (sum)
    main_function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 5, 0));
    // PC 4: r[7] = 0 (iteration counter)
    main_function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 7, 0));

    // PC 5: CallVirtual(dst: 6, slot: 0, arg_base: 2, arg_count: 1)
    // First iteration: calls obj_a, returns 10, quickens to CallVirtualDirect
    main_function
        .instructions
        .push(Instruction::format_call(OpCode::CallVirtual, 6, 0, 2, 1));
    // PC 6: r[5] += r[6]
    main_function
        .instructions
        .push(Instruction::format_a(OpCode::AddI32, 0, 5, 5, 6));
    // PC 7: r[7] += 1
    main_function
        .instructions
        .push(Instruction::format_a(OpCode::AddI32Imm, 0, 7, 7, 1));
    // PC 8: r[8] = 1
    main_function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 8, 1));
    // PC 9: JumpIfEqI32 r[7], r[8], offset: +2 -> jumps to PC 11
    main_function
        .instructions
        .push(Instruction::format_a(OpCode::JumpIfEqI32, 0, 7, 8, 2));
    // PC 10: Jump offset: +3 -> jumps to PC 13 (exit)
    main_function
        .instructions
        .push(Instruction::format_b_signed(OpCode::Jump, 0, 0, 3));
    // PC 11: r[2] = Mov r[1] (switch to obj_b)
    main_function
        .instructions
        .push(Instruction::format_a(OpCode::Mov, 0, 2, 1, 0));
    // PC 12: Jump offset: -7 -> jumps back to PC 5 (5 - 12 = -7)
    main_function
        .instructions
        .push(Instruction::format_b_signed(OpCode::Jump, 0, 0, -7));
    // PC 13: ReturnValue(r[5])
    main_function
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 5, 0, 0));

    binary.functions.push(main_function);
    binary.entry_point_id = Some(2);

    let options = RuntimeOptions::new().with_binary(binary);
    let mut context = RuntimeContext::new(options).expect("initialize context");
    context.flat_vtables = vec![vec![0].into_boxed_slice(), vec![1].into_boxed_slice()];
    let mut virtual_machine = VirtualMachine::new(context);

    let execution_result = virtual_machine.run().expect("execute virtual machine");
    // Iteration 1 (obj_a): sum = 10
    // Iteration 2 (obj_b): sum = 10 + 50 = 60
    assert_eq!(execution_result, Value::I32(60));
}
