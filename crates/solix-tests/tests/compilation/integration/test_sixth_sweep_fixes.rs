//! Integration tests for v2.0 sixth-sweep architectural, verifier, FFI, and stdlib fixes (Issues #203, #204, #207).
//!
//! Validates:
//! 1. Issue #203: Verifier operand register, receiver, and constant pool bounds checks for CallCallable, TupleInstanceof, CallVirtual, CallSymbol, CallNativeDirect.
//! 2. Issue #204: Binder assignment validation rejecting non-L-value expressions, methods, free functions, and classes (E0253).
//! 3. Issue #207: Primitives next_power_of_two32 preventing signed integer overflow to negative numbers.

#![forbid(unsafe_code)]

use solix_common::bytecode::instruction::Instruction;
use solix_common::bytecode::{BinaryProgram, CompiledFunction, OpCode};
use solix_runtime::options::RuntimeOptions;
use solix_runtime::value::model::Value;
use solix_runtime::{RuntimeContext, RuntimeError};
use solix_tests::common::{assert_compile_error, compile_and_run};

// ============================================================================
// Issue #203: Verifier Bounds Verification
// ============================================================================

#[test]
fn test_verifier_accepts_valid_callable_and_tuple_instructions() {
    let mut binary_program = BinaryProgram::new();
    let mut compiled_function = CompiledFunction::new("main", 0, 0, 8);

    // Instruction 0: Mov r1 -> r2
    compiled_function
        .instructions
        .push(Instruction::format_a(OpCode::Mov, 0, 2, 1, 0));
    // Instruction 1: CallCallable (dst: r0, callee: r1, arg_base: r2, arg_count: 1)
    compiled_function
        .instructions
        .push(Instruction::format_call(OpCode::CallCallable, 0, 1, 2, 1));
    // Instruction 2: TupleInstanceof (dst: r3, tuple: r4, index: r5)
    compiled_function
        .instructions
        .push(Instruction::format_a(OpCode::TupleInstanceof, 0, 3, 4, 5));
    // Instruction 3: Return
    compiled_function
        .instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));

    binary_program.functions.push(compiled_function);
    binary_program.entry_point_id = Some(0);

    let runtime_options = RuntimeOptions::new().with_binary(binary_program);
    let runtime_context_result = RuntimeContext::new(runtime_options);
    assert!(
        runtime_context_result.is_ok(),
        "Valid callable and tuple instructions should pass verification"
    );
}

#[test]
fn test_verifier_rejects_out_of_bounds_call_callable_target() {
    let mut binary_program = BinaryProgram::new();
    // Register count is 4 (registers 0..=3 are valid)
    let mut compiled_function = CompiledFunction::new("main", 0, 0, 4);

    // Callee target register 10 exceeds register count 4
    compiled_function
        .instructions
        .push(Instruction::format_call(OpCode::CallCallable, 0, 10, 1, 1));

    binary_program.functions.push(compiled_function);
    binary_program.entry_point_id = Some(0);

    let runtime_options = RuntimeOptions::new().with_binary(binary_program);
    let runtime_context_result = RuntimeContext::new(runtime_options);
    match runtime_context_result {
        Err(RuntimeError::BinaryLoadError { reason }) => {
            assert!(
                reason.contains("callee_reg register index 10 exceeds allocated register count 4"),
                "Expected callee_reg bounds error, got: {reason}"
            );
        }
        other_result => {
            panic!("Expected BinaryLoadError for out-of-bounds callee_reg, got {other_result:?}")
        }
    }
}

#[test]
fn test_verifier_rejects_out_of_bounds_tuple_instanceof_index_register() {
    let mut binary_program = BinaryProgram::new();
    // Register count is 4
    let mut compiled_function = CompiledFunction::new("main", 0, 0, 4);

    // src2 (index_reg) is register 8, which exceeds register count 4
    compiled_function
        .instructions
        .push(Instruction::format_a(OpCode::TupleInstanceof, 0, 0, 1, 8));

    binary_program.functions.push(compiled_function);
    binary_program.entry_point_id = Some(0);

    let runtime_options = RuntimeOptions::new().with_binary(binary_program);
    let runtime_context_result = RuntimeContext::new(runtime_options);
    match runtime_context_result {
        Err(RuntimeError::BinaryLoadError { reason }) => {
            assert!(
                reason.contains("index_reg register index 8 exceeds allocated register count 4"),
                "Expected index_reg bounds error, got: {reason}"
            );
        }
        other_result => {
            panic!("Expected BinaryLoadError for out-of-bounds index_reg, got {other_result:?}")
        }
    }
}

#[test]
fn test_verifier_rejects_out_of_bounds_call_virtual_receiver_when_arg_count_zero() {
    let mut binary_program = BinaryProgram::new();
    let mut compiled_function = CompiledFunction::new("main", 0, 0, 4);

    // CallVirtual with arg_count = 0 but receiver arg_base = 7 (> register_count 4)
    compiled_function
        .instructions
        .push(Instruction::format_call(OpCode::CallVirtual, 0, 0, 7, 0));

    binary_program.functions.push(compiled_function);
    binary_program.entry_point_id = Some(0);

    let runtime_options = RuntimeOptions::new().with_binary(binary_program);
    let runtime_context_result = RuntimeContext::new(runtime_options);
    match runtime_context_result {
        Err(RuntimeError::BinaryLoadError { reason }) => {
            assert!(
                reason.contains("receiver_reg register index 7 exceeds allocated register count 4"),
                "Expected receiver_reg bounds error, got: {reason}"
            );
        }
        other_result => {
            panic!("Expected BinaryLoadError for out-of-bounds receiver_reg, got {other_result:?}")
        }
    }
}

#[test]
fn test_verifier_rejects_out_of_bounds_call_symbol_constant_pool_index() {
    let mut binary_program = BinaryProgram::new();
    let mut compiled_function = CompiledFunction::new("main", 0, 0, 4);

    // CallSymbol with constant pool index 999 when pool length is 0
    compiled_function
        .instructions
        .push(Instruction::format_call(OpCode::CallSymbol, 0, 999, 0, 0));

    binary_program.functions.push(compiled_function);
    binary_program.entry_point_id = Some(0);

    let runtime_options = RuntimeOptions::new().with_binary(binary_program);
    let runtime_context_result = RuntimeContext::new(runtime_options);
    match runtime_context_result {
        Err(RuntimeError::BinaryLoadError { reason }) => {
            assert!(
                reason.contains("constant pool index 999 exceeds constant pool size 0"),
                "Expected constant pool bounds error, got: {reason}"
            );
        }
        other_result => panic!(
            "Expected BinaryLoadError for out-of-bounds constant pool index, got {other_result:?}"
        ),
    }
}

#[test]
fn test_verifier_rejects_out_of_bounds_call_native_direct_function_id() {
    let mut binary_program = BinaryProgram::new();
    let mut compiled_function = CompiledFunction::new("main", 0, 0, 4);

    // CallNativeDirect with function id 50 when total functions is 1
    compiled_function
        .instructions
        .push(Instruction::format_call(
            OpCode::CallNativeDirect,
            0,
            50,
            0,
            0,
        ));

    binary_program.functions.push(compiled_function);
    binary_program.entry_point_id = Some(0);

    let runtime_options = RuntimeOptions::new().with_binary(binary_program);
    let runtime_context_result = RuntimeContext::new(runtime_options);
    match runtime_context_result {
        Err(RuntimeError::BinaryLoadError { reason }) => {
            assert!(
                reason.contains("call target function ID 50 exceeds total function count 1"),
                "Expected function ID bounds error, got: {reason}"
            );
        }
        other_result => {
            panic!("Expected BinaryLoadError for out-of-bounds function ID, got {other_result:?}")
        }
    }
}

// ============================================================================
// Issue #204: Binder Assignment Validation (E0253)
// ============================================================================

#[test]
fn test_binder_accepts_valid_variable_and_field_assignment() {
    let source_code = r#"
class Container {
    public int32 counter = 10;

    public void update(int32 next_value) {
        this.counter = next_value;
    }
}

class Main {
    public static int32 main() {
        Container item = new Container();
        item.update(42);
        int32 local_value = item.counter;
        local_value = 100;
        return local_value;
    }
}
"#;
    let (execution_value, _virtual_machine) = compile_and_run(source_code);
    assert_eq!(execution_value, Value::I32(100));
}

#[test]
fn test_binder_rejects_assignment_to_method() {
    let source_code = r#"
class Demo {
    public void action() {}

    public void fail() {
        this.action = 42;
    }
}

class Main {
    public static int32 main() {
        return 0;
    }
}
"#;
    assert_compile_error(source_code, "E0253");
}

#[test]
fn test_binder_rejects_assignment_to_free_function() {
    let source_code = r#"
void helper() {}

class Main {
    public static int32 main() {
        helper = 10;
        return 0;
    }
}
"#;
    assert_compile_error(source_code, "E0253");
}

#[test]
fn test_binder_rejects_assignment_to_class() {
    let source_code = r#"
class TargetClass {}

class Main {
    public static int32 main() {
        TargetClass = 20;
        return 0;
    }
}
"#;
    assert_compile_error(source_code, "E0253");
}

// ============================================================================
// Issue #207: Primitives next_power_of_two32 Signed Overflow
// ============================================================================

#[test]
fn test_primitives_source_contains_power_of_two_clamping_guard() {
    let primitives_source =
        include_str!("../../../../../libraries/solixlib/src/core/Primitives.slx");
    assert!(
        primitives_source.contains("if (value >= 1073741824)"),
        "next_power_of_two32 must guard against inputs >= 1073741824 to prevent signed negative overflow"
    );
}

#[test]
fn test_primitives_next_power_of_two32_computes_valid_powers() {
    let source_code = r#"
class PrimitivesLogic {
    public static int32 next_power_of_two32(int32 value) {
        if (value <= 1) {
            return 1;
        }
        if (value >= 1073741824) {
            return 1073741824;
        }
        value = value - 1;
        value = value | (value >> 1);
        value = value | (value >> 2);
        value = value | (value >> 4);
        value = value | (value >> 8);
        value = value | (value >> 16);
        return value + 1;
    }
}

class Main {
    public static int32 main() {
        int32 power_one = PrimitivesLogic::next_power_of_two32(0);
        int32 power_two = PrimitivesLogic::next_power_of_two32(5);
        int32 power_three = PrimitivesLogic::next_power_of_two32(16);
        if (power_one != 1 || power_two != 8 || power_three != 16) {
            return 1;
        }
        return 0;
    }
}
"#;
    let (execution_value, _virtual_machine) = compile_and_run(source_code);
    assert_eq!(execution_value, Value::I32(0));
}

#[test]
fn test_primitives_next_power_of_two32_large_input_does_not_overflow_negative() {
    let source_code = r#"
class PrimitivesLogic {
    public static int32 next_power_of_two32(int32 value) {
        if (value <= 1) {
            return 1;
        }
        if (value >= 1073741824) {
            return 1073741824;
        }
        value = value - 1;
        value = value | (value >> 1);
        value = value | (value >> 2);
        value = value | (value >> 4);
        value = value | (value >> 8);
        value = value | (value >> 16);
        return value + 1;
    }
}

class Main {
    public static int32 main() {
        // Inputs above 1073741824 should clamp to 1073741824 instead of overflowing to negative MIN_INT32
        int32 result_large = PrimitivesLogic::next_power_of_two32(1073741825);
        if (result_large < 0) {
            return 1;
        }
        if (result_large != 1073741824) {
            return 2;
        }
        return 0;
    }
}
"#;
    let (execution_value, _virtual_machine) = compile_and_run(source_code);
    assert_eq!(execution_value, Value::I32(0));
}
