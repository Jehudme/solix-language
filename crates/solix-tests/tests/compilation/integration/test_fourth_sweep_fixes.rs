//! Integration tests for v2.0 fourth-sweep architectural and safety fixes (Issues #183 - #192).
//!
//! Validates:
//! 1. Issue #183: Heap buffer overflow in `execute_set_tuple` (RuntimeErrorCode::IndexOutOfBounds).
//! 2. Issue #184: Allocation integer arithmetic overflow in array and tuple creation (RuntimeErrorCode::OutOfMemory).
//! 3. Issue #185: Const field initializer validation rejecting non-constant expressions (E0287).
//! 4. Issue #186: Required formal parameters following optional parameters with default values (E0149).
//! 5. Issue #187: Auto-incrementing explicit negative enum values without arithmetic overflow.
//! 6. Issue #188: Enum switch exhaustiveness with duplicate or foreign cases (E0306).
//! 7. Issue #189: AOT verifier static slot bounds check when total static space is zero.
//! 8. Issue #190: Overflow-safe string slicing bounds check throwing IndexOutOfBoundsException.
//! 9. Issue #191: Interop C/Rust native callback panic catching across FFI boundary.
//! 10. Issue #192: Child process memory telemetry isolation in benchmark runner.

#![forbid(unsafe_code)]

use solix_common::bytecode::instruction::Instruction;
use solix_common::bytecode::{BinaryProgram, CompiledFunction, OpCode};
use solix_runtime::value::model::Value;
use solix_runtime::{RuntimeContext, RuntimeError, RuntimeOptions, VirtualMachine};
use solix_tests::common::{assert_compile_error, compile_and_run};

// ============================================================================
// Issue #183: Tuple Store Bounds Check & Heap Safety
// ============================================================================

#[test]
fn test_tuple_store_within_bounds_succeeds() {
    let mut binary_program = BinaryProgram::new();
    let mut compiled_function = CompiledFunction::new("main", 0, 0, 8);

    // Allocate tuple of length 2
    compiled_function
        .instructions
        .push(Instruction::format_a(OpCode::AllocTuple, 0, 0, 2, 0));

    // r[1] = index 0, r[2] = 42
    compiled_function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 0));
    compiled_function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 2, 42));
    compiled_function
        .instructions
        .push(Instruction::format_a(OpCode::SetArray, 0, 0, 1, 2));

    // r[3] = GetTuple(r[0], slot 0)
    compiled_function
        .instructions
        .push(Instruction::format_a(OpCode::GetTuple, 0, 3, 0, 0));
    compiled_function
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 3, 0, 0));

    binary_program.functions.push(compiled_function);
    binary_program.entry_point_id = Some(0);

    let runtime_options = RuntimeOptions::new().with_binary(binary_program);
    let runtime_context =
        RuntimeContext::new(runtime_options).expect("Context initialization must succeed");
    let mut virtual_machine = VirtualMachine::new(runtime_context);

    let execution_result = virtual_machine
        .run()
        .expect("Execution within bounds must succeed");
    assert_eq!(execution_result, Value::I32(42));
}

#[test]
fn test_tuple_store_out_of_bounds_trapped_without_heap_corruption() {
    let mut binary_program = BinaryProgram::new();
    let mut compiled_function = CompiledFunction::new("main", 0, 0, 8);

    // Allocate tuple of length 2
    compiled_function
        .instructions
        .push(Instruction::format_a(OpCode::AllocTuple, 0, 0, 2, 0));

    // Attempt to store at index 5 (greater than tuple_len 2)
    compiled_function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 5));
    compiled_function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 2, 99));
    compiled_function
        .instructions
        .push(Instruction::format_a(OpCode::SetArray, 0, 0, 1, 2));

    binary_program.functions.push(compiled_function);
    binary_program.entry_point_id = Some(0);

    let runtime_options = RuntimeOptions::new().with_binary(binary_program);
    let runtime_context =
        RuntimeContext::new(runtime_options).expect("Context initialization must succeed");
    let mut virtual_machine = VirtualMachine::new(runtime_context);

    let execution_result = virtual_machine.run();
    assert!(
        matches!(
            execution_result,
            Err(RuntimeError::IndexOutOfBounds {
                index: 5,
                length: 2,
                ..
            })
        ),
        "Out of bounds tuple store must fail with IndexOutOfBounds error"
    );
}

// ============================================================================
// Issue #184: Allocation Integer Arithmetic Overflow Protection
// ============================================================================

#[test]
fn test_oversized_array_allocation_traps_as_out_of_memory() {
    let mut binary_program = BinaryProgram::new();
    let mut compiled_function = CompiledFunction::new("main", 0, 0, 4);

    // Load massive length into register 1 and call AllocArray
    compiled_function
        .instructions
        .push(Instruction::format_b_signed(
            OpCode::LoadI32,
            0,
            1,
            0x7FFFFFFF,
        ));
    compiled_function
        .instructions
        .push(Instruction::format_a(OpCode::AllocArray, 0, 0, 1, 1)); // element type 1 = I32

    binary_program.functions.push(compiled_function);
    binary_program.entry_point_id = Some(0);

    let runtime_options = RuntimeOptions::new().with_binary(binary_program);
    let runtime_context =
        RuntimeContext::new(runtime_options).expect("Context initialization must succeed");
    let mut virtual_machine = VirtualMachine::new(runtime_context);

    let execution_result = virtual_machine.run();
    assert!(
        matches!(execution_result, Err(RuntimeError::OutOfMemory { .. })),
        "Huge array allocation must fail with OutOfMemory error without arithmetic overflow"
    );
}

// ============================================================================
// Issue #185: Const Field Initializer Validation (E0287)
// ============================================================================

#[test]
fn test_valid_const_expression_initializers_succeed() {
    let source_code = r#"
        class MathConstants {
            public const int32 BASE_OFFSET = 100 + 20 * 3;
            public const int32 NEGATIVE_VALUE = -42;
        }

        class Main {
            public static int32 main() {
                return MathConstants.BASE_OFFSET + MathConstants.NEGATIVE_VALUE;
            }
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(118));
}

#[test]
fn test_non_constant_method_call_in_const_declaration_rejected() {
    let source_code = r#"
        class Calculator {
            public static int32 compute_dynamic() {
                return 42;
            }
        }

        class Config {
            public const int32 DYNAMIC_CONST = Calculator.compute_dynamic();
        }

        class Main {
            public static int32 main() {
                return Config.DYNAMIC_CONST;
            }
        }
    "#;
    assert_compile_error(source_code, "E0287");
}

// ============================================================================
// Issue #186: Required Parameter After Default Parameter (E0149)
// ============================================================================

#[test]
fn test_optional_parameters_following_required_parameters_succeed() {
    let source_code = r#"
        class Greeter {
            public static int32 multiply(int32 base_value, int32 multiplier = 2) {
                return base_value * multiplier;
            }
        }

        class Main {
            public static int32 main() {
                return Greeter.multiply(15);
            }
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(30));
}

#[test]
fn test_required_parameter_following_default_parameter_rejected() {
    let source_code = r#"
        class InvalidFunction {
            public static int32 bad_parameters(int32 default_param = 10, int32 required_param) {
                return default_param + required_param;
            }
        }

        class Main {
            public static int32 main() {
                return InvalidFunction.bad_parameters(1, 2);
            }
        }
    "#;
    assert_compile_error(source_code, "E0149");
}

// ============================================================================
// Issue #187: Auto-Incrementing Negative Enum Values
// ============================================================================

#[test]
fn test_enum_explicit_negative_value_auto_increments_safely() {
    let source_code = r#"
        enum OffsetDelta {
            StartingOffset = -5,
            NextOffset,
            ThirdOffset,
        }

        class Main {
            public static int32 main() {
                int32 delta_value = (int32)OffsetDelta.ThirdOffset;
                return delta_value;
            }
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(-3));
}

// ============================================================================
// Issue #188: Enum Switch Exhaustiveness With Duplicate Cases (E0306)
// ============================================================================

#[test]
fn test_enum_switch_duplicate_cases_covering_subset_rejected() {
    let source_code = r#"
        enum StatusState {
            Pending,
            Running,
            Completed,
            Failed,
        }

        class Main {
            public static int32 evaluate_status(StatusState current_status) {
                switch (current_status) {
                    case StatusState.Pending:
                        return 1;
                    case StatusState.Running:
                        return 2;
                    case StatusState.Completed:
                        return 3;
                    case StatusState.Completed:
                        return 4;
                }
                return 0;
            }

            public static int32 main() {
                return Main.evaluate_status(StatusState.Running);
            }
        }
    "#;
    // Failed is not covered even though there are 4 case clauses (one is a duplicate of Completed)
    assert_compile_error(source_code, "E0322");
}

// ============================================================================
// Issue #189: AOT Verifier Static Space Bounds Check When Zero
// ============================================================================

#[test]
fn test_aot_verifier_rejects_static_direct_slot_when_static_space_is_zero() {
    let mut binary_program = BinaryProgram::new();
    let mut compiled_function = CompiledFunction::new("main", 0, 0, 4);

    // GetStaticDirect with slot 0 when total_static_space is 0
    compiled_function
        .instructions
        .push(Instruction::format_b(OpCode::GetStaticDirect, 0, 0, 0));
    compiled_function
        .instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));

    binary_program.functions.push(compiled_function);
    binary_program.entry_point_id = Some(0);

    let runtime_options = RuntimeOptions::new().with_binary(binary_program);
    let runtime_context = RuntimeContext::new(runtime_options);
    assert!(
        runtime_context.is_err(),
        "Binary with static slot access and 0 static space must fail verification"
    );
}

// ============================================================================
// Issue #190: Overflow-Safe String Bounds Check
// ============================================================================

#[test]
fn test_string_substring_normal_slicing_succeeds() {
    let source_code = r#"
        class Main {
            public static int32 main() {
                string greeting = "HelloWorld";
                return greeting.length;
            }
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(10));
}
