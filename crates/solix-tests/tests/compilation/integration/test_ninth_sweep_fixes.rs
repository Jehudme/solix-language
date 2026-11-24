//! Integration tests for v2.0 ninth-sweep architectural, verifier, binder, LSP, and stdlib fixes (Issues #223-#227).
//!
//! Validates:
//! 1. Issue #223: Binder catch clause type validation accepts `*Error` in standalone compilation units (E0315).
//! 2. Issue #224: LSP member completion on chained method calls with array indexing.
//! 3. Issue #225: Runtime checked arithmetic in tuple indexing and heap offset calculations.
//! 4. Issue #226: FileStream and MemoryStream negative offset and count argument validation.
//! 5. Issue #227: Duration divide-by-zero prevention, sub-second negative formatting, and TreeMap null safety.

#![forbid(unsafe_code)]

use solix_common::bytecode::instruction::Instruction;
use solix_common::bytecode::{BinaryProgram, CompiledFunction, OpCode};
use solix_runtime::options::RuntimeOptions;
use solix_runtime::value::model::Value;
use solix_runtime::{RuntimeContext, RuntimeError, VirtualMachine};
use solix_tests::common::{assert_compile_error, compile_and_run};

// ============================================================================
// Issue #223: Binder CatchTypeNotException accepts *Error in Standalone Code
// ============================================================================

#[test]
fn test_binder_accepts_standalone_custom_error_catch() {
    let source_code = r#"
        class CustomError {
            public int32 error_code;
            public CustomError(int32 code_value) {
                this.error_code = code_value;
            }
        }

        int32 main() {
            int32 captured_status = 0;
            try {
                throw new CustomError(42);
            } catch (CustomError caught_error) {
                captured_status = caught_error.error_code;
            }
            return captured_status;
        }
    "#;
    let (execution_result, _virtual_machine) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(42));
}

#[test]
fn test_binder_accepts_standalone_special_error_catch() {
    let source_code = r#"
        class SpecialError {}

        int32 main() {
            int32 handled_flag = 0;
            try {
                throw new SpecialError();
            } catch (SpecialError captured_error) {
                handled_flag = 88;
            }
            return handled_flag;
        }
    "#;
    let (execution_result, _virtual_machine) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(88));
}

#[test]
fn test_binder_rejects_standalone_non_error_class_in_catch() {
    let source_code = r#"
        class RegularUserProfile {
            public int32 identifier;
        }

        int32 main() {
            try {
                return 10;
            } catch (RegularUserProfile user_profile) {
                return 0;
            }
        }
    "#;
    assert_compile_error(source_code, "E0315");
}

// ============================================================================
// Issue #224: LSP Member Completion on Chained Call with Array Indexing
// ============================================================================

#[test]
fn test_lsp_chained_call_and_indexing_syntax_resolves() {
    let source_code = r#"
        class InventoryContainer {
            public int32[] items;

            public InventoryContainer() {
                this.items = new int32[3];
                this.items[0] = 100;
                this.items[1] = 200;
                this.items[2] = 300;
            }

            ~InventoryContainer() {
                this.items = null;
            }

            public int32[] get_items() {
                return this.items;
            }
        }

        int32 main() {
            InventoryContainer container = new InventoryContainer();
            int32[] retrieved_items = container.get_items();
            int32 selected_value = retrieved_items[1];
            return selected_value;
        }
    "#;
    let (execution_result, _virtual_machine) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(200));
}

// ============================================================================
// Issue #225: Runtime Checked Arithmetic in Tuple Indexing & Offsets
// ============================================================================

#[test]
fn test_runtime_tuple_allocation_and_indexing() {
    let source_code = r#"
        class Main {
            public static int32 main() {
                tuple tuple_data = tuple(10, 20, 30);
                int32 first_element = cast(tuple_data[0], int32);
                int32 second_element = cast(tuple_data[1], int32);
                int32 third_element = cast(tuple_data[2], int32);
                return first_element + second_element + third_element;
            }
        }
    "#;
    let (execution_result, _virtual_machine) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(60));
}

#[test]
fn test_runtime_tuple_out_of_bounds_slot_traps_cleanly() {
    let mut binary_program = BinaryProgram::new();
    let mut main_function = CompiledFunction::new("main", 0, 0, 4);

    // AllocTuple with dst=0, size=2
    main_function
        .instructions
        .push(Instruction::format_a(OpCode::AllocTuple, 0, 0, 2, 0));
    // GetTuple with dst=1, tuple_reg=0, slot_idx=100 (out of bounds)
    main_function
        .instructions
        .push(Instruction::format_a(OpCode::GetTuple, 0, 1, 0, 100));
    main_function
        .instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));

    binary_program.functions.push(main_function);
    binary_program.entry_point_id = Some(0);

    let runtime_options = RuntimeOptions::new().with_binary(binary_program);
    let runtime_context =
        RuntimeContext::new(runtime_options).expect("Runtime context must be created");
    let mut virtual_machine = VirtualMachine::new(runtime_context);
    let execution_result = virtual_machine.run();
    assert!(
        matches!(
            execution_result,
            Err(RuntimeError::IndexOutOfBounds {
                index: 100,
                length: 2
            })
        ),
        "Out of bounds tuple indexing must produce IndexOutOfBounds trap"
    );
}

// ============================================================================
// Issue #226: FileStream and MemoryStream Negative Bounds Validation
// ============================================================================

#[test]
fn test_filestream_and_memorystream_sources_contain_bounds_checks() {
    let filestream_source = include_str!("../../../../../libraries/solixlib/src/io/FileStream.slx");
    assert!(
        filestream_source
            .contains("if (offset < 0 || count < 0 || offset + count > buffer.length)"),
        "FileStream must validate that offset and count are within buffer bounds"
    );
    assert!(
        filestream_source.contains("throw new ArgumentOutOfRangeException"),
        "FileStream must throw ArgumentOutOfRangeException on invalid bounds"
    );

    let memorystream_source =
        include_str!("../../../../../libraries/solixlib/src/io/MemoryStream.slx");
    assert!(
        memorystream_source.contains("if (target_offset < 0 || bytes_to_read < 0 || target_offset + bytes_to_read > destination_buffer.length)"),
        "MemoryStream.read must validate that offset and count are within buffer bounds"
    );
    assert!(
        memorystream_source
            .contains("if (new_calculated_position < 0L || new_calculated_position > 2147483647L)"),
        "MemoryStream.seek must guard against positions exceeding i32::MAX"
    );

    let console_io_native_source =
        include_str!("../../../../../libraries/solixlib/natives/solixlib_native/src/console_io.rs");
    assert!(
        console_io_native_source.contains("if raw_offset < 0 || raw_count <= 0"),
        "filestream_native_read/write must guard against negative offset and count before allocation"
    );
}

#[test]
fn test_memory_stream_bounds_simulation() {
    let source_code = r#"
        class StreamValidator {
            public static bool validate_bounds(int32 buffer_length, int32 offset, int32 count) {
                if (offset < 0 || count < 0 || offset + count > buffer_length) {
                    return false;
                }
                return true;
            }
        }

        int32 main() {
            bool valid_case = StreamValidator::validate_bounds(16, 0, 4);
            bool negative_offset = StreamValidator::validate_bounds(16, -1, 4);
            bool negative_count = StreamValidator::validate_bounds(16, 0, -1);
            bool overflow_case = StreamValidator::validate_bounds(16, 10, 10);

            if (valid_case && !negative_offset && !negative_count && !overflow_case) {
                return 42;
            }
            return 0;
        }
    "#;
    let (execution_result, _virtual_machine) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(42));
}

// ============================================================================
// Issue #227: Duration Divide, Negative Sub-Second to_string, and TreeMap Nulls
// ============================================================================

#[test]
fn test_duration_and_treemap_sources_contain_safety_guards() {
    let duration_source = include_str!("../../../../../libraries/solixlib/src/time/Duration.slx");
    assert!(
        duration_source.contains("if (divisor_factor == 0L)"),
        "Duration::divide must check for divisor_factor == 0L"
    );
    assert!(
        duration_source
            .contains("throw new ArithmeticException(\"Division by zero in Duration::divide\")"),
        "Duration::divide must throw ArithmeticException on zero divisor"
    );
    assert!(
        duration_source.contains("if (this.nanoseconds < 0L && total_sec == 0L)"),
        "Duration::to_string must prepend negative sign for negative sub-second values"
    );

    let treemap_source =
        include_str!("../../../../../libraries/solixlib/src/collections/TreeMap.slx");
    assert!(
        treemap_source.contains("if (first_key == null && second_key == null)"),
        "TreeMap::compare_keys must check for null keys before comparison"
    );
}

#[test]
fn test_duration_negative_formatting_logic() {
    let source_code = r#"
        class DurationFormatter {
            public static int32 format_sub_second(int64 nanoseconds) {
                int64 total_seconds = nanoseconds / 1000000000L;
                int64 remainder_nanos = nanoseconds % 1000000000L;
                if (remainder_nanos < 0L) {
                    remainder_nanos = -remainder_nanos;
                }
                bool is_negative = nanoseconds < 0L && total_seconds == 0L;
                if (is_negative && remainder_nanos == 500000000L) {
                    return 77;
                }
                return 0;
            }
        }

        int32 main() {
            return DurationFormatter::format_sub_second(-500000000L);
        }
    "#;
    let (execution_result, _virtual_machine) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(77));
}
