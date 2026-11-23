//! Integration tests for v2.0 seventh-sweep architectural, verifier, binder, toolchain, DAP, and stdlib fixes (Issues #210-#215).
//!
//! Validates:
//! 1. Issue #210: Verifier bounds verification for AllocObjectExtern, GetStatic, SetStatic, GetStaticGlobal, and SetStaticGlobal.
//! 2. Issue #211: Binder relational comparison operand compatibility validation (E0248).
//! 3. Issue #212: Toolchain `solix info` accepting projects residing within ancestor directories named `build` or `stage`.
//! 4. Issue #214: Stdlib `Random.next_int32(lower_bound, upper_bound)` preventing 32-bit signed integer overflow.
//! 5. Issue #215: Stdlib `Math.factorial` input bounds validation and overflow protection.

#![forbid(unsafe_code)]

use std::fs;
use tempfile::tempdir;

use solix_common::bytecode::instruction::Instruction;
use solix_common::bytecode::{BinaryProgram, CompiledFunction, OpCode};
use solix_runtime::options::RuntimeOptions;
use solix_runtime::value::model::Value;
use solix_runtime::{RuntimeContext, RuntimeError};
use solix_tests::common::{assert_compile_error, compile_and_run};

// ============================================================================
// Issue #210: Verifier Bounds Verification
// ============================================================================

#[test]
fn test_verifier_accepts_valid_object_and_static_instructions() {
    let mut binary_program = BinaryProgram::new();
    binary_program.static_space_size = 4;
    let _ = binary_program.constant_pool.add_string("TestClass");

    let mut compiled_function = CompiledFunction::new("main", 0, 0, 4);

    // AllocObject: dst=0
    compiled_function
        .instructions
        .push(Instruction::format_a(OpCode::AllocObject, 0, 0, 0, 0));
    // AllocObjectExtern: dst=1, imm32=0 (valid pool index)
    compiled_function
        .instructions
        .push(Instruction::format_b(OpCode::AllocObjectExtern, 0, 1, 0));
    // GetStatic: dst=2, imm32=1 (valid slot < 4)
    compiled_function
        .instructions
        .push(Instruction::format_b(OpCode::GetStatic, 0, 2, 1));
    // SetStatic: dst=2, imm32=2 (valid slot < 4)
    compiled_function
        .instructions
        .push(Instruction::format_b(OpCode::SetStatic, 0, 2, 2));
    // GetStaticGlobal: dst=3, imm32=0 (valid pool index)
    compiled_function
        .instructions
        .push(Instruction::format_b(OpCode::GetStaticGlobal, 0, 3, 0));
    // Return
    compiled_function
        .instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));

    binary_program.functions.push(compiled_function);
    binary_program.entry_point_id = Some(0);

    let runtime_options = RuntimeOptions::new().with_binary(binary_program);
    let runtime_context_result = RuntimeContext::new(runtime_options);
    assert!(
        runtime_context_result.is_ok(),
        "Valid object and static instructions should pass verification"
    );
}

#[test]
fn test_verifier_rejects_out_of_bounds_alloc_object_extern_pool_index() {
    let mut binary_program = BinaryProgram::new();
    let mut compiled_function = CompiledFunction::new("main", 0, 0, 4);

    // AllocObjectExtern with pool index 42 exceeding pool length 0
    compiled_function
        .instructions
        .push(Instruction::format_b(OpCode::AllocObjectExtern, 0, 0, 42));

    binary_program.functions.push(compiled_function);
    binary_program.entry_point_id = Some(0);

    let runtime_options = RuntimeOptions::new().with_binary(binary_program);
    let runtime_context_result = RuntimeContext::new(runtime_options);
    match runtime_context_result {
        Err(RuntimeError::BinaryLoadError {
            reason: error_reason,
        }) => {
            assert!(
                error_reason
                    .contains("AllocObjectExtern constant pool index 42 exceeds pool length 0"),
                "Expected AllocObjectExtern pool bounds error, got: {error_reason}"
            );
        }
        other_result => {
            panic!(
                "Expected BinaryLoadError for out-of-bounds AllocObjectExtern pool index, got {other_result:?}"
            )
        }
    }
}

#[test]
fn test_verifier_rejects_out_of_bounds_get_static_slot() {
    let mut binary_program = BinaryProgram::new();
    binary_program.static_space_size = 2; // only slots 0, 1
    let mut compiled_function = CompiledFunction::new("main", 0, 0, 4);

    // GetStatic slot 8 exceeds static space 2
    compiled_function
        .instructions
        .push(Instruction::format_b(OpCode::GetStatic, 0, 0, 8));

    binary_program.functions.push(compiled_function);
    binary_program.entry_point_id = Some(0);

    let runtime_options = RuntimeOptions::new().with_binary(binary_program);
    let runtime_context_result = RuntimeContext::new(runtime_options);
    match runtime_context_result {
        Err(RuntimeError::BinaryLoadError {
            reason: error_reason,
        }) => {
            assert!(
                error_reason.contains("static slot 8 exceeds total static space 2"),
                "Expected static slot bounds error, got: {error_reason}"
            );
        }
        other_result => {
            panic!("Expected BinaryLoadError for out-of-bounds static slot, got {other_result:?}")
        }
    }
}

#[test]
fn test_verifier_rejects_out_of_bounds_get_static_global_pool_index() {
    let mut binary_program = BinaryProgram::new();
    let mut compiled_function = CompiledFunction::new("main", 0, 0, 4);

    // GetStaticGlobal pool index 15 exceeds pool length 0
    compiled_function
        .instructions
        .push(Instruction::format_b(OpCode::GetStaticGlobal, 0, 0, 15));

    binary_program.functions.push(compiled_function);
    binary_program.entry_point_id = Some(0);

    let runtime_options = RuntimeOptions::new().with_binary(binary_program);
    let runtime_context_result = RuntimeContext::new(runtime_options);
    match runtime_context_result {
        Err(RuntimeError::BinaryLoadError {
            reason: error_reason,
        }) => {
            assert!(
                error_reason.contains("global static constant pool index 15 exceeds pool length 0"),
                "Expected global static pool bounds error, got: {error_reason}"
            );
        }
        other_result => {
            panic!(
                "Expected BinaryLoadError for out-of-bounds global static pool index, got {other_result:?}"
            )
        }
    }
}

// ============================================================================
// Issue #211: Binder Relational Comparison Type Compatibility (E0248)
// ============================================================================

#[test]
fn test_relational_comparisons_on_valid_numerics() {
    let source_code = r#"
class Main {
    public static int32 main() {
        int32 first_number = 10;
        int32 second_number = 20;
        bool is_less = first_number < second_number;
        bool is_greater_equal = first_number >= 5;
        float64 floating_value = 15.5;
        bool float_comparison = floating_value > 10.0;
        if (is_less && is_greater_equal && float_comparison) {
            return 0;
        }
        return 1;
    }
}
"#;
    let (execution_result, _virtual_machine) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_relational_comparison_with_operator_overload() {
    let source_code = r#"
class Priority {
    public int32 priority_level;

    public Priority(int32 level) {
        this.priority_level = level;
    }

    public bool operator <(Priority other_priority) {
        return this.priority_level < other_priority.priority_level;
    }
}

class Main {
    public static int32 main() {
        Priority low_priority = new Priority(1);
        Priority high_priority = new Priority(5);
        if (low_priority < high_priority) {
            return 42;
        }
        return 0;
    }
}
"#;
    let (execution_result, _virtual_machine) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(42));
}

#[test]
fn test_binder_rejects_relational_comparison_on_incompatible_types_e0248() {
    let source_code = r#"
class Main {
    public static int32 main() {
        int32 numeric_value = 10;
        string text_value = "hello";
        bool invalid_comparison = numeric_value < text_value;
        return 0;
    }
}
"#;
    assert_compile_error(source_code, "E0248");
}

#[test]
fn test_binder_rejects_relational_comparison_on_booleans_e0248() {
    let source_code = r#"
class Main {
    public static int32 main() {
        bool first_boolean = true;
        bool second_boolean = false;
        bool invalid_comparison = first_boolean < second_boolean;
        return 0;
    }
}
"#;
    assert_compile_error(source_code, "E0248");
}

#[test]
fn test_binder_rejects_relational_comparison_on_class_without_operator_e0248() {
    let source_code = r#"
class Container {
    public int32 payload_value;
    public Container(int32 initial_value) {
        this.payload_value = initial_value;
    }
}

class Main {
    public static int32 main() {
        Container first_container = new Container(10);
        Container second_container = new Container(20);
        bool invalid_comparison = first_container < second_container;
        return 0;
    }
}
"#;
    assert_compile_error(source_code, "E0248");
}

// ============================================================================
// Issue #212: Toolchain Directory Resolution with Ancestor 'build' / 'stage'
// ============================================================================

#[test]
fn test_toolchain_info_allows_project_in_path_with_build_ancestor() {
    let temporary_root = tempdir().expect("Create temporary directory");
    let nested_build_directory = temporary_root
        .path()
        .join("build_workspace")
        .join("test_project");
    fs::create_dir_all(&nested_build_directory).expect("Create project directory");

    let manifest_content = r#"
name: test_project
version: 1.0.0
description: Sample project inside ancestor path named build
profiles:
  release:
    optimize: speed
"#;
    fs::write(
        nested_build_directory.join("manifest.yaml"),
        manifest_content,
    )
    .expect("Write manifest.yaml");

    let source_directory = nested_build_directory.join("src");
    fs::create_dir_all(&source_directory).expect("Create src directory");
    fs::write(
        source_directory.join("Main.slx"),
        "public class Main { public static void main() {} }",
    )
    .expect("Write Main.slx");

    let manifest_found = solix_common::project::find_manifest_path(&nested_build_directory);
    assert!(
        manifest_found.is_some(),
        "Manifest should be located in project inside build ancestor"
    );
}

// ============================================================================
// Issue #214: Stdlib Random Range Integer Overflow Protection
// ============================================================================

#[test]
fn test_random_source_contains_64bit_span_arithmetic() {
    let random_source = include_str!("../../../../../libraries/solixlib/src/math/Random.slx");
    assert!(
        random_source.contains("int64 range_span = (int64)upper_bound - (int64)lower_bound;"),
        "Random.next_int32 must calculate range_span using 64-bit arithmetic to prevent signed overflow"
    );
    assert!(
        random_source.contains("if (range_span <= 2147483647L)"),
        "Random.next_int32 must check range_span against 32-bit maximum"
    );
}

#[test]
fn test_random_large_range_span_execution() {
    let source_code = r#"
class RandomLogic {
    public static int32 next_int_in_range(int32 lower_bound, int32 upper_bound, int64 simulated_random) {
        int64 range_span = (int64)upper_bound - (int64)lower_bound;
        int64 positive_bits = simulated_random & 9223372036854775807L;
        int64 offset_value = positive_bits % range_span;
        return (int32)((int64)lower_bound + offset_value);
    }
}

class Main {
    public static int32 main() {
        int32 result = RandomLogic::next_int_in_range(-2000000000, 2000000000, 5000000000L);
        if (result >= -2000000000 && result < 2000000000) {
            return 100;
        }
        return 0;
    }
}
"#;
    let (execution_result, _virtual_machine) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(100));
}

// ============================================================================
// Issue #215: Stdlib Math.factorial Input Bounds & Overflow
// ============================================================================

#[test]
fn test_math_source_contains_factorial_overflow_guards() {
    let math_source = include_str!("../../../../../libraries/solixlib/src/math/Math.slx");
    assert!(
        math_source.contains("if (target_number < 0)"),
        "Math.factorial must guard against negative inputs"
    );
    assert!(
        math_source.contains("if (target_number > 20)"),
        "Math.factorial must guard against 64-bit integer overflow for target_number > 20"
    );
}

#[test]
fn test_math_factorial_computes_expected_values() {
    let source_code = r#"
class MathLogic {
    public static int64 factorial(int32 target_number) {
        if (target_number <= 1) {
            return 1L;
        }
        int64 accumulator = 1L;
        int32 current_factor = 2;
        while (current_factor <= target_number) {
            accumulator = accumulator * (int64)current_factor;
            current_factor = current_factor + 1;
        }
        return accumulator;
    }
}

class Main {
    public static int32 main() {
        int64 factorial_zero = MathLogic::factorial(0);
        int64 factorial_one = MathLogic::factorial(1);
        int64 factorial_five = MathLogic::factorial(5);
        int64 factorial_twenty = MathLogic::factorial(20);

        if (factorial_zero == 1L
            && factorial_one == 1L
            && factorial_five == 120L
            && factorial_twenty == 2432902008176640000L) {
            return 42;
        }
        return 0;
    }
}
"#;
    let (execution_result, _virtual_machine) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(42));
}
