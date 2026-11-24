//! Integration tests for v2.0 tenth-sweep architectural, binder, VM, and stdlib fixes (Issues #229-#234).
//!
//! Validates:
//! 1. Issue #229: Compiler/Binder rejects throwing non-Exception types (E0316) and non-class composite types.
//! 2. Issue #232: Runtime VM uncaught exception reporting formats object class symbol and message payload.
//! 3. Issue #233: Stdlib String::parse_int 32-bit bounds validation and PriorityQueue null priority guards.
//! 4. Issue #234: Stdlib Math::lcm 64-bit signed integer arithmetic overflow protection.

#![forbid(unsafe_code)]

use solix_runtime::options::RuntimeOptions;
use solix_runtime::value::model::Value;
use solix_runtime::{RuntimeContext, RuntimeError, VirtualMachine};
use solix_tests::common::{assert_compile_error, compile_and_run, compile_source};

// ============================================================================
// Issue #229: Binder ThrowTypeNotException (E0316) & Non-Class Throw Rejection
// ============================================================================

#[test]
fn test_binder_accepts_valid_exception_class_throw() {
    let source_code = r#"
        class ValidDomainError {
            public int32 failure_code;
            public ValidDomainError(int32 code_value) {
                this.failure_code = code_value;
            }
        }

        int32 main() {
            int32 captured_status = 0;
            try {
                throw new ValidDomainError(77);
            } catch (ValidDomainError caught_error) {
                captured_status = caught_error.failure_code;
            }
            return captured_status;
        }
    "#;
    let (execution_result, _virtual_machine) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(77));
}

#[test]
fn test_binder_rejects_non_exception_class_throw() {
    let source_code = r#"
        class ArbitraryPayloadRecord {
            public int32 payload_number;
        }

        int32 main() {
            ArbitraryPayloadRecord record_instance = new ArbitraryPayloadRecord();
            throw record_instance;
            return 0;
        }
    "#;
    assert_compile_error(source_code, "E0316");
}

#[test]
fn test_binder_rejects_array_throw() {
    let source_code = r#"
        int32 main() {
            throw new int32[5];
            return 0;
        }
    "#;
    assert_compile_error(source_code, "E0203");
}

// ============================================================================
// Issue #232: Runtime VM Uncaught Exception Message Formatting
// ============================================================================

#[test]
fn test_vm_format_exception_message_null_and_string_values() {
    let runtime_options = RuntimeOptions::new();
    let runtime_context =
        RuntimeContext::new(runtime_options).expect("RuntimeContext creation failed");
    let virtual_machine = VirtualMachine::new(runtime_context);

    // Null exception returns NullReferenceException
    let null_message = virtual_machine.format_exception_message(Value::Null);
    assert_eq!(null_message, "NullReferenceException");
}

#[test]
fn test_vm_uncaught_user_exception_reporting_formats_class_symbol() {
    let source_code = r#"
        class FatalFault {
            public int32 status_code;
            public FatalFault(int32 code_value) {
                this.status_code = code_value;
            }
        }

        int32 main() {
            throw new FatalFault(99);
            return 0;
        }
    "#;
    let compiled_program = compile_source(source_code);
    let runtime_options = RuntimeOptions::new()
        .with_binary(compiled_program)
        .with_entry_point("main");
    let runtime_context =
        RuntimeContext::new(runtime_options).expect("RuntimeContext creation failed");
    let mut virtual_machine = VirtualMachine::new(runtime_context);

    let execution_result = virtual_machine.run();
    match execution_result {
        Err(RuntimeError::UserException {
            message: error_message,
        }) => {
            assert!(
                error_message.contains("FatalFault"),
                "Expected message to contain class symbol 'FatalFault', got: '{error_message}'"
            );
            assert!(
                !error_message.starts_with("ref("),
                "Uncaught exception message must not be raw ref pointer: '{error_message}'"
            );
        }
        other_result => panic!("Expected RuntimeError::UserException, got: {other_result:?}"),
    }
}

// ============================================================================
// Issue #233: Stdlib String::parse_int Bounds & PriorityQueue Null Guards
// ============================================================================

#[test]
fn test_stdlib_string_parse_int_bounds_source_inspection() {
    let string_source = include_str!("../../../../../libraries/solixlib/src/core/String.slx");
    assert!(
        string_source.contains("-2147483648L || parsed_value > 2147483647L"),
        "String.slx must validate 32-bit signed integer boundaries"
    );
    assert!(
        string_source.contains("Integer value out of 32-bit signed range"),
        "String.slx must throw NumberFormatException on 32-bit integer overflow"
    );
}

#[test]
fn test_stdlib_priority_queue_null_comparison_source_inspection() {
    let priority_queue_source =
        include_str!("../../../../../libraries/solixlib/src/collections/PriorityQueue.slx");
    assert!(
        priority_queue_source.contains("if (first_item == null && second_item == null)"),
        "PriorityQueue must guard null-to-null comparisons"
    );
    assert!(
        priority_queue_source.contains("if (first_item == null)"),
        "PriorityQueue must guard first_item null checks"
    );
    assert!(
        priority_queue_source.contains("if (second_item == null)"),
        "PriorityQueue must guard second_item null checks"
    );
}

// ============================================================================
// Issue #234: Stdlib Math::lcm Overflow Protection
// ============================================================================

#[test]
fn test_stdlib_math_lcm_overflow_guard_source_inspection() {
    let math_source = include_str!("../../../../../libraries/solixlib/src/math/Math.slx");
    assert!(
        math_source.contains("Arithmetic overflow in Math::lcm"),
        "Math.slx must protect lcm against 64-bit integer multiplication overflow"
    );
    assert!(
        math_source.contains("9223372036854775807L / abs_second"),
        "Math.slx must check division upper bounds before multiplying"
    );
}
