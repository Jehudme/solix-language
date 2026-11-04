//! WhileStatement Conformance Tests (SPEC-0001 §12, TESTS.md Section 2.13).

#![forbid(unsafe_code)]

use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_3_1_standard_counted_loop() {
    let code = r#"
        int32 main() {
            int32 total = 0;
            int32 i = 1;
            while (i <= 5) {
                total = total + i;
                i++;
            }
            return total;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(15));
}

#[test]
fn test_case_3_2_while_loop_with_break_and_continue() {
    let code = r#"
        int32 main() {
            int32 sum = 0;
            int32 i = 0;
            while (true) {
                i++;
                if (i % 2 == 0) continue;
                if (i > 5) break;
                sum = sum + i;
            }
            return sum;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(9));
}

#[test]
fn test_case_3_3_while_loop_with_complex_short_circuit_condition() {
    let code = r#"
        int32 main() {
            int32 count = 0;
            int32 limit = 5;
            while (count < 10 && count < limit) {
                count++;
            }
            return count == 5 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_non_boolean_loop_condition() {
    // Error: E0202 (TypeMismatch)
    let code = r#"
        void test() {
            int32 x = 5;
            while (x) {
                x--;
            }
        }
    "#;
    assert_compile_error(code, "bool");
}

#[test]
fn test_case_4_2_while_loop_missing_condition_parentheses() {
    // Error: E0100 (ParsingError)
    let code = r#"
        void test() {
            while true {}
        }
    "#;
    assert_compile_error(code, "E0144");
}

#[test]
fn test_case_4_3_while_loop_placed_directly_in_class_body() {
    // Error: E0100 (ParsingError)
    let code = r#"
        class BadClass {
            while (true) {}
        }
    "#;
    assert_compile_error(code, "class");
}
