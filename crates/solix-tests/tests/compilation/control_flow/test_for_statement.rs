//! ForStatement Conformance Tests (SPEC-0001 §12, TESTS.md Section 2.6).

#![forbid(unsafe_code)]

use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_3_1_standard_for_loop_with_continue() {
    let code = r#"
        int32 main() {
            int32 evens = 0;
            for (int32 i = 0; i < 10; i++) {
                if (i % 2 != 0) continue;
                evens++;
            }
            return evens;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(5));
}

#[test]
fn test_case_3_2_empty_header_clauses_for_infinite_loop_with_break() {
    let code = r#"
        int32 main() {
            int32 count = 0;
            for (;;) {
                count++;
                if (count == 5) {
                    break;
                }
            }
            return count == 5 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_3_nested_for_loops_for_matrix_summation() {
    let code = r#"
        int32 main() {
            int32 total = 0;
            for (int32 i = 0; i < 3; i++) {
                for (int32 j = 0; j < 3; j++) {
                    total += 1;
                }
            }
            return total == 9 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_induction_variable_leakage() {
    // Error: E0200 (UndefinedIdentifier)
    let code = r#"
        void test() {
            for (int32 i = 0; i < 5; i++) {}
            int32 leak = i;
        }
    "#;
    assert_compile_error(code, "i");
}

#[test]
fn test_case_4_2_non_boolean_condition_in_for_loop() {
    // Error: E0202 (TypeMismatch)
    let code = r#"
        void test() {
            for (int32 i = 0; 100; i++) {}
        }
    "#;
    assert_compile_error(code, "bool");
}

#[test]
fn test_case_4_3_for_loop_placed_directly_in_class_body() {
    // Error: E0100 (ParsingError)
    let code = r#"
        class BadClass {
            for (int32 i = 0; i < 10; i++) {}
        }
    "#;
    assert_compile_error(code, "class");
}
