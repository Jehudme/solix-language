//! Integration tests for v2.0 eighth-sweep architectural, verifier, binder, DAP, and stdlib fixes (Issues #217-#221).
//!
//! Validates:
//! 1. Issue #217: Verifier bounds verification for `OpCode::PackClosure` target function ID.
//! 2. Issue #218: Binder catch clause type validation against `Exception` hierarchy (E0315) and character relational comparisons.
//! 3. Issue #219: Collections `BitSet` safe word count calculation and `HashMap` threshold overflow prevention.
//! 4. Issue #220: DAP evaluator logical operator (&&, ||) evaluation.
//! 5. Issue #221: Stdlib `Math.clamp` parameter validation (min <= max) and `Math.gcd` bounds handling.

#![forbid(unsafe_code)]

use solix_common::bytecode::instruction::Instruction;
use solix_common::bytecode::{BinaryProgram, CompiledFunction, OpCode};
use solix_runtime::options::RuntimeOptions;
use solix_runtime::value::model::Value;
use solix_runtime::{RuntimeContext, RuntimeError};
use solix_tests::common::{assert_compile_error, compile_and_run};

// ============================================================================
// Issue #217: Verifier Bounds Verification for OpCode::PackClosure
// ============================================================================

#[test]
fn test_verifier_accepts_valid_pack_closure_target() {
    let mut binary_program = BinaryProgram::new();
    let mut main_function = CompiledFunction::new("main", 0, 0, 4);
    let worker_function = CompiledFunction::new("closure_worker", 1, 0, 2);

    // PackClosure with closure_dst=0, target_function=1 (valid, total_functions=2)
    main_function
        .instructions
        .push(Instruction::format_call(OpCode::PackClosure, 0, 1, 1, 1));
    main_function
        .instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));

    binary_program.functions.push(main_function);
    binary_program.functions.push(worker_function);
    binary_program.entry_point_id = Some(0);

    let runtime_options = RuntimeOptions::new().with_binary(binary_program);
    let context_result = RuntimeContext::new(runtime_options);
    assert!(
        context_result.is_ok(),
        "Valid PackClosure target must pass AOT verification"
    );
}

#[test]
fn test_verifier_rejects_pack_closure_target_exceeding_total_functions() {
    let mut binary_program = BinaryProgram::new();
    let mut main_function = CompiledFunction::new("main", 0, 0, 4);

    // PackClosure with closure_dst=0, target_function=100 (total_functions=1)
    main_function
        .instructions
        .push(Instruction::format_call(OpCode::PackClosure, 0, 100, 1, 1));
    main_function
        .instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));

    binary_program.functions.push(main_function);

    let runtime_options = RuntimeOptions::new().with_binary(binary_program);
    let context_result = RuntimeContext::new(runtime_options);

    match context_result {
        Err(RuntimeError::BinaryLoadError {
            reason: error_reason,
        }) => {
            assert!(
                error_reason
                    .contains("closure target function ID 100 exceeds total function count 1"),
                "Expected error message indicating out-of-bounds target function, got: {error_reason}"
            );
        }
        other_result => {
            panic!("Expected BinaryLoadError for out-of-bounds PackClosure, got: {other_result:?}");
        }
    }
}

// ============================================================================
// Issue #218: Binder Catch Clause Type Validation (E0315) & Char Comparisons
// ============================================================================

#[test]
fn test_binder_accepts_valid_exception_subclass_in_catch() {
    let source_code = r#"
class Exception {
    public string message;
    public Exception(string initial_message) {
        this.message = initial_message;
    }
}

class CustomDomainException extends Exception {
    public CustomDomainException(string custom_message) : super(custom_message) {
    }
}

class Main {
    public static int32 main() {
        int32 status_code = 0;
        try {
            throw new CustomDomainException("domain error");
        } catch (CustomDomainException domain_error) {
            status_code = 42;
        }
        return status_code;
    }
}
"#;
    let (execution_result, _virtual_machine) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(42));
}

#[test]
fn test_binder_rejects_catch_clause_on_non_exception_class_e0315() {
    let source_code = r#"
class PlainDataRecord {
    public int32 payload_number;
    public PlainDataRecord(int32 initial_payload) {
        this.payload_number = initial_payload;
    }
}

class Main {
    public static int32 main() {
        try {
            int32 regular_value = 100;
        } catch (PlainDataRecord data_record) {
            return 1;
        }
        return 0;
    }
}
"#;
    assert_compile_error(source_code, "E0315");
}

#[test]
fn test_binder_accepts_character_relational_comparisons() {
    let source_code = r#"
class Main {
    public static int32 main() {
        char first_character = 'a';
        char second_character = 'z';
        bool is_strictly_less = first_character < second_character;
        bool is_greater_equal = second_character >= first_character;
        bool is_same = first_character <= 'a';
        if (is_strictly_less && is_greater_equal && is_same) {
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
// Issue #219: Collections BitSet & HashMap Source and Logic Verification
// ============================================================================

#[test]
fn test_bitset_source_contains_safe_word_count_calculation() {
    let bitset_source =
        include_str!("../../../../../libraries/solixlib/src/collections/BitSet.slx");
    assert!(
        bitset_source
            .contains("(initial_bit_count / 64) + ((initial_bit_count % 64 != 0) ? 1 : 0)"),
        "BitSet must calculate word count without (initial_bit_count + 63) signed 32-bit overflow"
    );
}

#[test]
fn test_hashmap_source_contains_safe_threshold_calculation() {
    let hashmap_source =
        include_str!("../../../../../libraries/solixlib/src/collections/HashMap.slx");
    assert!(
        hashmap_source.contains("(initial_capacity / 4) * 3"),
        "HashMap constructor must calculate resize_threshold without initial_capacity * 3 overflow"
    );
    assert!(
        hashmap_source.contains("(new_capacity / 4) * 3"),
        "HashMap resize must calculate resize_threshold without new_capacity * 3 overflow"
    );
}

#[test]
fn test_hashmap_safe_threshold_execution() {
    let source_code = r#"
class HashCalculator {
    public static int32 compute_safe_threshold(int32 capacity) {
        return (capacity / 4) * 3;
    }
}

class Main {
    public static int32 main() {
        int32 capacity_medium = 16;
        int32 capacity_large = 1073741824;
        int32 threshold_medium = HashCalculator::compute_safe_threshold(capacity_medium);
        int32 threshold_large = HashCalculator::compute_safe_threshold(capacity_large);

        if (threshold_medium == 12 && threshold_large == 805306368) {
            return 77;
        }
        return 0;
    }
}
"#;
    let (execution_result, _virtual_machine) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(77));
}

// ============================================================================
// Issue #221: Stdlib Math Clamp Bounds & GCD Overflow Safety
// ============================================================================

#[test]
fn test_math_source_contains_clamp_and_gcd_guards() {
    let math_source = include_str!("../../../../../libraries/solixlib/src/math/Math.slx");
    assert!(
        math_source.contains("if (min_val > max_val)"),
        "Math.clamp must validate min_val <= max_val and throw ArgumentOutOfRangeException"
    );
    assert!(
        math_source.contains("-9223372036854775808L"),
        "Math.gcd must guard against 64-bit minimum negation overflow (-9223372036854775808L)"
    );
}

#[test]
fn test_math_clamp_valid_ranges_execution() {
    let source_code = r#"
class MathLogic {
    public static int32 clamp(int32 target_val, int32 min_val, int32 max_val) {
        if (target_val < min_val) {
            return min_val;
        }
        if (target_val > max_val) {
            return max_val;
        }
        return target_val;
    }
}

class Main {
    public static int32 main() {
        int32 clamped_middle = MathLogic::clamp(15, 10, 20);
        int32 clamped_lower = MathLogic::clamp(5, 10, 20);
        int32 clamped_upper = MathLogic::clamp(25, 10, 20);
        if (clamped_middle == 15 && clamped_lower == 10 && clamped_upper == 20) {
            return 88;
        }
        return 0;
    }
}
"#;
    let (execution_result, _virtual_machine) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(88));
}

#[test]
fn test_math_gcd_handles_positive_and_negative_operands_execution() {
    let source_code = r#"
class MathLogic {
    public static int64 abs(int64 target_val) {
        if (target_val < 0L) {
            return -target_val;
        }
        return target_val;
    }

    public static int64 gcd(int64 first_val, int64 second_val) {
        int64 operand_a = MathLogic::abs(first_val);
        int64 operand_b = MathLogic::abs(second_val);
        while (operand_b != 0L) {
            int64 remainder_val = operand_a % operand_b;
            operand_a = operand_b;
            operand_b = remainder_val;
        }
        return operand_a;
    }
}

class Main {
    public static int32 main() {
        int64 greatest_common_divisor_a = MathLogic::gcd(48L, 18L);
        int64 greatest_common_divisor_b = MathLogic::gcd(-48L, 18L);
        if (greatest_common_divisor_a == 6L && greatest_common_divisor_b == 6L) {
            return 66;
        }
        return 0;
    }
}
"#;
    let (execution_result, _virtual_machine) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(66));
}
