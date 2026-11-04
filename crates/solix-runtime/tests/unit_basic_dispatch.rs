//! Unit tests for Value system, CallFrame stack, and basic opcode dispatch.

#![forbid(unsafe_code)]

use solix_common::bytecode::{BinaryProgram, CompiledFunction, Instruction, OpCode};
use solix_runtime::{RuntimeContext, RuntimeOptions, Value, VirtualMachine};

#[test]
fn test_value_truthiness_and_casts() {
    assert!(!Value::Null.is_truthy());
    assert!(!Value::Bool(false).is_truthy());
    assert!(Value::Bool(true).is_truthy());
    assert!(!Value::I32(0).is_truthy());
    assert!(Value::I32(42).is_truthy());
    assert!(!Value::I64(0).is_truthy());
    assert!(Value::I64(-1).is_truthy());
    assert!(!Value::F64(0.0).is_truthy());
    assert!(!Value::F64(f64::NAN).is_truthy());
    assert!(Value::F64(2.75).is_truthy());
    assert!(!Value::Ref(0).is_truthy());
    assert!(Value::Ref(0x1000).is_truthy());
    assert!(Value::FnPtr(0).is_truthy());

    // Type casts
    assert_eq!(Value::I32(10).as_i32().unwrap(), 10);
    assert_eq!(Value::I64(20).as_i64().unwrap(), 20);
    assert!((Value::F64(1.5).as_f64().unwrap() - 1.5).abs() < f64::EPSILON);
    assert!(Value::Bool(true).as_bool().unwrap());
    assert_eq!(Value::Ref(50).as_ref().unwrap(), 50);

    // Errors
    assert!(Value::Null.as_bool().is_err());
    assert!(Value::Bool(true).as_i32().is_err());
}

#[test]
fn test_value_display_formatting() {
    assert_eq!(format!("{}", Value::Null), "null");
    assert_eq!(format!("{}", Value::Bool(true)), "true");
    assert_eq!(format!("{}", Value::I32(42)), "42");
    assert_eq!(format!("{}", Value::I64(-100)), "-100");
    assert_eq!(format!("{}", Value::F64(5.0)), "5.0");
    assert_eq!(format!("{}", Value::F64(12.34)), "12.34");
    assert_eq!(format!("{}", Value::Ref(0x10)), "<ref 0x00000010>");
    assert_eq!(format!("{}", Value::FnPtr(3)), "<fn_ptr #3>");
}

#[test]
fn test_dispatch_constants_and_moves() {
    let mut binary = BinaryProgram::new();
    let mut func = CompiledFunction::new("main", 0, 0, 8);

    // r[0] = LoadI32(42)
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 42));
    // r[1] = Mov(r[0])
    func.instructions
        .push(Instruction::format_a(OpCode::Mov, 0, 1, 0, 0));
    // r[2] = LoadBool(true)
    func.instructions
        .push(Instruction::format_b(OpCode::LoadBool, 0, 2, 1));
    // r[3] = LoadNull
    func.instructions
        .push(Instruction::format_b(OpCode::LoadNull, 0, 3, 0));
    // ReturnValue(r[1])
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 1, 0, 0));

    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let result = vm.run().expect("VM run failed");
    assert_eq!(result, Value::I32(42));
    assert!(vm.verify_clean_memory().is_ok());
}

#[test]
fn test_dispatch_exit_opcode() {
    let mut binary = BinaryProgram::new();
    let mut func = CompiledFunction::new("main", 0, 0, 4);

    // r[0] = LoadI32(7)
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 7));
    // Exit(r[0])
    func.instructions
        .push(Instruction::format_a(OpCode::Exit, 0, 0, 0, 0));

    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let result = vm.run().expect("VM run failed");
    assert_eq!(result, Value::I32(7));
    assert_eq!(vm.exit_code, Some(7));
    assert!(vm.halted);
    assert!(vm.verify_clean_memory().is_ok());
}

#[test]
fn test_dispatch_static_variables_read_and_write() {
    let mut binary = BinaryProgram::new();
    binary.static_space_size = 2;
    let mut func = CompiledFunction::new("main", 0, 0, 4);

    // r[0] = LoadI64(1000)
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI64, 0, 0, 1000));
    // SetStatic(slot 0, r[0])
    func.instructions
        .push(Instruction::format_b(OpCode::SetStatic, 0, 0, 0));
    // r[1] = GetStatic(slot 0)
    func.instructions
        .push(Instruction::format_b(OpCode::GetStatic, 0, 1, 0));
    // ReturnValue(r[1])
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 1, 0, 0));

    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let result = vm.run().expect("VM run failed");
    assert_eq!(result, Value::I64(1000));
    assert_eq!(vm.get_static(0).unwrap(), Value::I64(1000));
    assert_eq!(vm.heap.read_static_u64(0).unwrap(), 1000);
    assert!(vm.verify_clean_memory().is_ok());
}

#[test]
fn test_dispatch_load_const_float_and_string() {
    let mut binary = BinaryProgram::new();
    let idx_f64 = binary.constant_pool.add_f64(9.876).unwrap();
    let idx_str = binary.constant_pool.add_string("hello solix").unwrap();

    let mut func = CompiledFunction::new("main", 0, 0, 4);
    // r[0] = LoadF64(idx_f64)
    func.instructions
        .push(Instruction::format_b(OpCode::LoadF64, 0, 0, idx_f64 as u32));
    // r[1] = LoadConst(idx_str)
    func.instructions.push(Instruction::format_b(
        OpCode::LoadConst,
        0,
        1,
        idx_str as u32,
    ));
    // ReturnValue(r[1])
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 1, 0, 0));

    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let result = vm.run().expect("VM run failed");
    let str_ref = result.as_ref().unwrap();
    assert!(str_ref > 0);
    let bytes = vm.heap.read_bytes(str_ref, 11).unwrap();
    assert_eq!(std::str::from_utf8(bytes).unwrap(), "hello solix");

    // Deallocate the string to verify clean memory
    vm.heap.deallocate(str_ref).unwrap();
    assert!(vm.verify_clean_memory().is_ok());
}
