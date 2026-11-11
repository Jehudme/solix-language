//! Unit tests for RuntimeErrorCode, entry point signature validation, CLI arguments, and native options.

#![forbid(unsafe_code)]

use solix_common::bytecode::{BinaryProgram, CompiledFunction, Instruction, OpCode};
use solix_runtime::{
    RuntimeContext, RuntimeError, RuntimeErrorCode, RuntimeOptions, Value, VirtualMachine,
};

#[test]
fn test_runtime_error_code_mapping_and_as_i32() {
    let cases: Vec<(RuntimeError, RuntimeErrorCode, i32)> = vec![
        (
            RuntimeError::OutOfMemory {
                requested: 10,
                available: 5,
            },
            RuntimeErrorCode::OutOfMemory,
            1,
        ),
        (
            RuntimeError::StackOverflow {
                current_depth: 100,
                max_depth: 50,
            },
            RuntimeErrorCode::StackOverflow,
            2,
        ),
        (
            RuntimeError::DivideByZero,
            RuntimeErrorCode::DivideByZero,
            3,
        ),
        (
            RuntimeError::IndexOutOfBounds {
                index: 5,
                length: 3,
            },
            RuntimeErrorCode::IndexOutOfBounds,
            4,
        ),
        (
            RuntimeError::NullReferenceAccess,
            RuntimeErrorCode::NullReferenceAccess,
            5,
        ),
        (
            RuntimeError::InvalidCast {
                from: "A".into(),
                to: "B".into(),
            },
            RuntimeErrorCode::InvalidCast,
            6,
        ),
        (
            RuntimeError::FunctionNotFound {
                symbol_or_id: "foo".into(),
            },
            RuntimeErrorCode::FunctionNotFound,
            7,
        ),
        (
            RuntimeError::EntryPointNotFound {
                symbol: "main".into(),
            },
            RuntimeErrorCode::EntryPointNotFound,
            8,
        ),
        (
            RuntimeError::InvalidInstruction { pc: 0, raw: 0 },
            RuntimeErrorCode::InvalidInstruction,
            9,
        ),
        (
            RuntimeError::InvalidOpCode {
                pc: 0,
                opcode: 0xFF,
            },
            RuntimeErrorCode::InvalidOpCode,
            10,
        ),
        (
            RuntimeError::InvalidFieldOffset {
                offset: 100,
                object_size: 16,
            },
            RuntimeErrorCode::InvalidFieldOffset,
            11,
        ),
        (
            RuntimeError::InvalidStaticOffset {
                offset: 50,
                total_static: 10,
            },
            RuntimeErrorCode::InvalidStaticOffset,
            12,
        ),
        (
            RuntimeError::BinaryLoadError {
                reason: "corrupted".into(),
            },
            RuntimeErrorCode::BinaryLoadError,
            13,
        ),
        (
            RuntimeError::UserException {
                message: "fail".into(),
            },
            RuntimeErrorCode::UserException,
            14,
        ),
        (
            RuntimeError::NativeFunctionError {
                name: "f".into(),
                reason: "err".into(),
            },
            RuntimeErrorCode::NativeFunctionError,
            15,
        ),
        (
            RuntimeError::SecurityViolation {
                action: "dlopen".into(),
            },
            RuntimeErrorCode::SecurityViolation,
            16,
        ),
        (
            RuntimeError::TypeMismatch {
                expected: "int".into(),
                found: "ref".into(),
            },
            RuntimeErrorCode::TypeMismatch,
            17,
        ),
        (
            RuntimeError::AssertionFailed {
                message: "false".into(),
            },
            RuntimeErrorCode::AssertionFailed,
            18,
        ),
        (
            RuntimeError::MemoryLeak {
                unallocated_count: 1,
                report: String::new(),
            },
            RuntimeErrorCode::MemoryLeak,
            19,
        ),
        (
            RuntimeError::InvalidEntryPointSignature {
                symbol: "main".into(),
                param_count: 3,
                reason: "invalid parameter count".into(),
            },
            RuntimeErrorCode::InvalidEntryPointSignature,
            20,
        ),
        (
            RuntimeError::DuplicateSymbolDefinition {
                kind: "function".into(),
                symbol: "test".into(),
                first_module: "a".into(),
                second_module: "b".into(),
            },
            RuntimeErrorCode::DuplicateSymbolDefinition,
            24,
        ),
        (
            RuntimeError::Exit { code: 42 },
            RuntimeErrorCode::ExplicitExit,
            21,
        ),
        (RuntimeError::Exit { code: 0 }, RuntimeErrorCode::Success, 0),
    ];

    for (err, expected_code, expected_num) in cases {
        assert_eq!(err.error_code(), expected_code);
        assert_eq!(err.error_code().as_i32(), expected_num);
    }
}

#[test]
fn test_abrupt_termination_captures_error_code() {
    let mut binary = BinaryProgram::new();
    let mut func = CompiledFunction::new("main", 0, 0, 4);

    // PC 0: r[0] = 10
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 10));
    // PC 1: r[1] = 0
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 0));
    // PC 2: r[2] = r[0] / r[1] -> triggers DivideByZero!
    func.instructions
        .push(Instruction::format_a(OpCode::DivI32, 0, 2, 0, 1));
    // PC 3: Return
    func.instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));

    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let exit_status = vm.execute();
    assert_eq!(exit_status, RuntimeErrorCode::DivideByZero.as_i32());
    assert_eq!(vm.exit_code, Some(3));
    assert!(vm.halted);
}

#[test]
fn test_entry_point_void_returns_zero_automatically() {
    let mut binary = BinaryProgram::new();
    let mut func = CompiledFunction::new("main", 0, 0, 2);

    // void main() ends with Return (producing Value::Null)
    func.instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));

    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let result = vm.run().expect("Execution succeeded");
    assert_eq!(result, Value::I32(0));
    assert_eq!(vm.execute(), 0);
    assert_eq!(vm.exit_code, Some(0));
}

#[test]
fn test_entry_point_rejects_invalid_signature_more_than_one_param() {
    let mut binary = BinaryProgram::new();
    // main with 2 parameters -> invalid!
    let mut func = CompiledFunction::new("main", 0, 2, 4);
    func.instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));

    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let res = vm.run();
    assert!(matches!(
        res,
        Err(RuntimeError::InvalidEntryPointSignature { param_count: 2, .. })
    ));
    assert_eq!(
        vm.exit_code,
        Some(RuntimeErrorCode::InvalidEntryPointSignature.as_i32())
    );
    assert_eq!(
        vm.execute(),
        RuntimeErrorCode::InvalidEntryPointSignature.as_i32()
    );
}

#[test]
fn test_entry_point_receives_cli_arguments_array() {
    let mut binary = BinaryProgram::new();
    // main(char[][] args) -> param_count = 1
    // r[0] holds args array reference
    // r[1] = ArrayLength(r[0])
    // ReturnValue(r[1])
    let mut func = CompiledFunction::new("main", 0, 1, 4);
    func.instructions
        .push(Instruction::format_a(OpCode::ArrayLength, 0, 1, 0, 0));
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 1, 0, 0));

    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary).with_args(vec![
        "foo".to_string(),
        "bar".to_string(),
        "baz".to_string(),
    ]);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let result = vm.run().expect("Execution succeeded");
    assert_eq!(result, Value::I32(3));
    assert_eq!(vm.execute(), 3);
}

#[test]
fn test_runtime_options_native_function_registration_and_stdout() {
    let mut binary = BinaryProgram::new();
    let const_idx = binary.constant_pool.add_string("custom_ping").unwrap();

    let mut func = CompiledFunction::new("main", 0, 0, 4);

    // Call native "custom_ping"
    func.instructions.push(Instruction::format_call(
        OpCode::CallSymbol,
        1,
        const_idx,
        0,
        0,
    ));
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 1, 0, 0));

    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let custom_callback = |_vm: &mut VirtualMachine,
                           _args: &[Value]|
     -> Result<Value, RuntimeError> { Ok(Value::I32(777)) };

    let options = RuntimeOptions::new()
        .with_binary(binary)
        .with_native_function("custom_ping", custom_callback);

    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let result = vm.run().expect("Execution succeeded");
    assert_eq!(result, Value::I32(777));
}
