//! Integration tests for v2.0 second-sweep architectural and safety fixes (Issues #161 - #168).
//!
//! Validates:
//! 1. Issue #161: Unsigned arithmetic and comparison VM opcodes (`uint8`..`uint64`).
//! 2. Issue #162: Register window freeing in switch custom equality/inequality overloads.
//! 3. Issue #163: Cyclic interface and class inheritance safety (E0204, no compiler hang).
//! 4. Issue #164: Negative array length compile-time check (E0299) and runtime dimension trap.
//! 5. Issue #165: Assert statement string message validation (E0301) and catchable AssertionError.
//! 6. Issue #166: Generic type constraint error disambiguation (E0297).
//! 7. Issue #167: Enum constant binary/bitwise expressions and overflow bounds check.
//! 8. Issue #168: Iterative cycle collector traversal preventing thread stack overflow.

#![forbid(unsafe_code)]

use solix_runtime::context::RuntimeContext;
use solix_runtime::exceptions::errors::RuntimeErrorCode;
use solix_runtime::options::RuntimeOptions;
use solix_runtime::value::model::Value;
use solix_runtime::vm::VirtualMachine;
use solix_tests::common::{
    assert_compile_error, assert_runtime_error, compile_and_run, compile_source,
};

// ============================================================================
// Issue #161: Unsigned Arithmetic & Relational Comparisons
// ============================================================================

#[test]
fn test_unsigned_arithmetic_uint64_div_mod_lshr_and_comparisons() {
    let source_code = r#"
        class UnsignedMathRunner {
            public static int32 main() {
                uint64 max_unsigned = 18446744073709551614;
                uint64 half_val = max_unsigned / 2;
                if (half_val != 9223372036854775807) {
                    return 1;
                }

                uint64 remainder_val = max_unsigned % 10;
                if (remainder_val != 4) {
                    return 2;
                }

                uint64 shifted_val = max_unsigned >> 1;
                if (shifted_val != 9223372036854775807) {
                    return 3;
                }

                uint64 small_val = 100;
                if (!(small_val < max_unsigned)) {
                    return 4;
                }
                if (small_val > max_unsigned) {
                    return 5;
                }
                if (!(small_val <= max_unsigned)) {
                    return 6;
                }
                if (small_val >= max_unsigned) {
                    return 7;
                }

                uint32 unsigned_thirty_two = 4000000000;
                uint32 divided_thirty_two = unsigned_thirty_two / 2;
                if (divided_thirty_two != 2000000000) {
                    return 8;
                }

                return 0;
            }
        }
    "#;
    let (result_value, _) = compile_and_run(source_code);
    assert_eq!(result_value, Value::I32(0));
}

// ============================================================================
// Issue #162: Register Window Freeing in Switch Statement Overloads
// ============================================================================

#[test]
fn test_switch_statement_custom_equality_overload_many_cases_no_window_leak() {
    let source_code = r#"
        class TokenIdentifier {
            public int32 raw_identifier;

            public TokenIdentifier(int32 raw_identifier) {
                this.raw_identifier = raw_identifier;
            }

            public bool operator ==(TokenIdentifier other_token) {
                if (other_token == null) {
                    return false;
                }
                return this.raw_identifier == other_token.raw_identifier;
            }
        }

        int32 match_token(TokenIdentifier target_token) {
            TokenIdentifier token_1 = new TokenIdentifier(1);
            TokenIdentifier token_2 = new TokenIdentifier(2);
            TokenIdentifier token_3 = new TokenIdentifier(3);
            TokenIdentifier token_4 = new TokenIdentifier(4);
            TokenIdentifier token_5 = new TokenIdentifier(5);
            TokenIdentifier token_6 = new TokenIdentifier(6);
            TokenIdentifier token_7 = new TokenIdentifier(7);
            TokenIdentifier token_8 = new TokenIdentifier(8);
            TokenIdentifier token_9 = new TokenIdentifier(9);
            TokenIdentifier token_10 = new TokenIdentifier(10);
            TokenIdentifier token_11 = new TokenIdentifier(11);
            TokenIdentifier token_12 = new TokenIdentifier(12);
            TokenIdentifier token_13 = new TokenIdentifier(13);
            TokenIdentifier token_14 = new TokenIdentifier(14);
            TokenIdentifier token_15 = new TokenIdentifier(15);
            TokenIdentifier token_16 = new TokenIdentifier(16);
            TokenIdentifier token_17 = new TokenIdentifier(17);
            TokenIdentifier token_18 = new TokenIdentifier(18);
            TokenIdentifier token_19 = new TokenIdentifier(19);
            TokenIdentifier token_20 = new TokenIdentifier(20);
            TokenIdentifier token_21 = new TokenIdentifier(21);
            TokenIdentifier token_22 = new TokenIdentifier(22);
            TokenIdentifier token_23 = new TokenIdentifier(23);
            TokenIdentifier token_24 = new TokenIdentifier(24);
            TokenIdentifier token_25 = new TokenIdentifier(25);
            TokenIdentifier token_26 = new TokenIdentifier(26);
            TokenIdentifier token_27 = new TokenIdentifier(27);
            TokenIdentifier token_28 = new TokenIdentifier(28);
            TokenIdentifier token_29 = new TokenIdentifier(29);
            TokenIdentifier token_30 = new TokenIdentifier(30);

            switch (target_token) {
                case token_1: return 101;
                case token_2: return 102;
                case token_3: return 103;
                case token_4: return 104;
                case token_5: return 105;
                case token_6: return 106;
                case token_7: return 107;
                case token_8: return 108;
                case token_9: return 109;
                case token_10: return 110;
                case token_11: return 111;
                case token_12: return 112;
                case token_13: return 113;
                case token_14: return 114;
                case token_15: return 115;
                case token_16: return 116;
                case token_17: return 117;
                case token_18: return 118;
                case token_19: return 119;
                case token_20: return 120;
                case token_21: return 121;
                case token_22: return 122;
                case token_23: return 123;
                case token_24: return 124;
                case token_25: return 125;
                case token_26: return 126;
                case token_27: return 127;
                case token_28: return 128;
                case token_29: return 129;
                case token_30: return 130;
                default: return 999;
            }
        }

        int32 main() {
            TokenIdentifier twentieth_token = new TokenIdentifier(20);
            return match_token(twentieth_token);
        }
    "#;
    let (result_value, _) = compile_and_run(source_code);
    assert_eq!(result_value, Value::I32(120));
}

// ============================================================================
// Issue #163: Cyclic Interface & Class Inheritance Safety
// ============================================================================

#[test]
fn test_cyclic_interface_inheritance_rejected_with_e0204() {
    let source_code = r#"
        interface AlphaProtocol extends BetaProtocol {}
        interface BetaProtocol extends AlphaProtocol {}

        class ImplementationClass implements AlphaProtocol {}

        class Main {
            public static int32 main() {
                return 0;
            }
        }
    "#;
    assert_compile_error(source_code, "E0204");
}

#[test]
fn test_cyclic_class_inheritance_does_not_hang_compiler_and_rejects_e0204() {
    let source_code = r#"
        class FirstClass extends SecondClass {
            public int32 payload_first;
        }

        class SecondClass extends FirstClass {
            public int32 payload_second;
        }

        class Main {
            public static int32 main() {
                FirstClass instance_first = new FirstClass();
                return instance_first.payload_first;
            }
        }
    "#;
    assert_compile_error(source_code, "E0204");
}

// ============================================================================
// Issue #164: Negative Array Length Safety
// ============================================================================

#[test]
fn test_negative_array_length_rejected_at_compile_time_with_e0299() {
    let source_code = r#"
        class NegativeArrayCompileTest {
            public static int32 main() {
                int32[] buffer = new int32[-10];
                return buffer.length;
            }
        }
    "#;
    assert_compile_error(source_code, "E0299");
}

#[test]
fn test_runtime_negative_array_length_traps_with_index_out_of_bounds() {
    let source_code = r#"
        class NegativeArrayRuntimeTest {
            public static int32 main() {
                int32 dynamic_length = -5;
                int32[] buffer = new int32[dynamic_length];
                return buffer.length;
            }
        }
    "#;
    assert_runtime_error(source_code, RuntimeErrorCode::IndexOutOfBounds);
}

// ============================================================================
// Issue #165: Assert Statement String Message & Catchable AssertionError
// ============================================================================

#[test]
fn test_assert_statement_with_non_string_message_rejected_with_e0301() {
    let source_code = r#"
        class AssertNonStringTest {
            public static int32 main() {
                assert(true, 404);
                return 0;
            }
        }
    "#;
    assert_compile_error(source_code, "E0301");
}

#[test]
fn test_assert_statement_failure_throws_catchable_assertion_error() {
    let source_code = r#"
        class AssertionError {}

        int32 main() {
            try {
                assert(2 + 2 == 5, "Math consistency check failed");
                return 1;
            } catch (AssertionError error_instance) {
                return 42;
            }
        }
    "#;
    let (result_value, _) = compile_and_run(source_code);
    assert_eq!(result_value, Value::I32(42));
}

// ============================================================================
// Issue #166: Generic Type Constraint Error Code E0297
// ============================================================================

#[test]
fn test_generic_type_constraint_violation_emits_e0297_not_e0264() {
    let source_code = r#"
        class BaseConstraint {}

        class GenericContainer<ElementType: BaseConstraint> {
            public ElementType content_value;
        }

        int32 main() {
            GenericContainer<int32> container = new GenericContainer<int32>();
            return 0;
        }
    "#;
    assert_compile_error(source_code, "E0297");
}

// ============================================================================
// Issue #167: Enum Constant Binary/Bitwise Expressions & Overflow Guard
// ============================================================================

#[test]
fn test_enum_constant_binary_and_bitwise_expressions() {
    let source_code = r#"
        enum AccessFlags {
            None = 0,
            Read = 1 << 0,
            Write = 1 << 1,
            Execute = 1 << 2,
            ReadWrite = (1 << 0) | (1 << 1),
            Mask = 0xFF & 0x0F,
            Calculation = (10 * 5) - 8,
        }

        class EnumBinaryTestRunner {
            public static int32 main() {
                if (AccessFlags::Read != 1) {
                    return 1;
                }
                if (AccessFlags::Write != 2) {
                    return 2;
                }
                if (AccessFlags::Execute != 4) {
                    return 3;
                }
                if (AccessFlags::ReadWrite != 3) {
                    return 4;
                }
                if (AccessFlags::Mask != 15) {
                    return 5;
                }
                if (AccessFlags::Calculation != 42) {
                    return 6;
                }
                return 0;
            }
        }
    "#;
    let (result_value, _) = compile_and_run(source_code);
    assert_eq!(result_value, Value::I32(0));
}

#[test]
fn test_enum_auto_increment_overflow_diagnostic() {
    let source_code = r#"
        enum OverflowEnum {
            MaximumValue = 9223372036854775807,
            OverflowedValue,
        }

        class Main {
            public static int32 main() {
                return 0;
            }
        }
    "#;
    assert_compile_error(source_code, "E0246");
}

// ============================================================================
// Issue #168: Iterative Cycle Collector Traversal Without Native Stack Overflow
// ============================================================================

#[test]
fn test_cycle_collector_deep_linear_graph_iterative_no_stack_overflow() {
    let source_code = r#"
        class DeepCycleNode {
            public DeepCycleNode next_pointer;
            public int32 node_index;

            public DeepCycleNode(int32 node_index) {
                this.node_index = node_index;
                this.next_pointer = null;
            }
        }

        void create_deep_circular_graph(int32 chain_length) {
            DeepCycleNode root_node = new DeepCycleNode(0);
            DeepCycleNode current_node = root_node;
            int32 current_index = 1;
            while (current_index < chain_length) {
                DeepCycleNode child_node = new DeepCycleNode(current_index);
                current_node.next_pointer = child_node;
                current_node = child_node;
                current_index = current_index + 1;
            }
            current_node.next_pointer = root_node;
        }

        int32 main() {
            create_deep_circular_graph(1000);
            return 0;
        }
    "#;

    let compiled_program = compile_source(source_code);
    let options = RuntimeOptions::new()
        .with_binary(compiled_program)
        .with_cycle_collector(true)
        .with_cycle_threshold(2000);
    let context = RuntimeContext::new(options).expect("Runtime context construction must succeed");
    let mut virtual_machine = VirtualMachine::new(context);

    let execution_result = virtual_machine.run();
    assert_eq!(
        execution_result.expect("main must execute cleanly"),
        Value::I32(0)
    );

    let collected_blocks_count = virtual_machine
        .collect_cycles()
        .expect("Cycle collection must succeed without native stack overflow");
    assert!(
        collected_blocks_count >= 1000,
        "Cycle collector must reclaim deep circular graph, got: {collected_blocks_count}"
    );
}
