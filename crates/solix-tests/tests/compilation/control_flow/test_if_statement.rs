//! IfStatement Conformance Tests (SPEC-0001 §12, TESTS.md Section 2.7).

#![forbid(unsafe_code)]

use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_3_1_single_branch_if() {
    let code = r#"
        int32 main() {
            int32 x = 10;
            if (x > 5) {
                x = 20;
            }
            return x;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(20));
}

#[test]
fn test_case_3_2_if_else_chain() {
    let code = r#"
        int32 main() {
            int32 score = 85;
            int32 grade = 0;
            if (score >= 90) {
                grade = 1;
            } else if (score >= 80) {
                grade = 2;
            } else {
                grade = 3;
            }
            return grade;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(2));
}

#[test]
fn test_case_3_3_complex_short_circuit_logical_condition() {
    let code = r#"
        int32 main() {
            int32 a = 10;
            int32 b = 20;
            bool executed = false;
            if (a == 10 && (b == 20 || false)) {
                executed = true;
            }
            return executed ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_non_boolean_condition_type() {
    // Error: E0202 (TypeMismatch)
    let code = r#"
        void test() {
            int32 count = 1;
            if (count) {
            }
        }
    "#;
    assert_compile_error(code, "bool");
}

#[test]
fn test_case_4_2_incompatible_assignment_in_condition() {
    // Error: E0100 / E0202
    let code = r#"
        void test() {
            int32 x = 0;
            if (x = 5) {}
        }
    "#;
    assert_compile_error(code, "bool");
}

#[test]
fn test_case_4_3_if_statement_placed_directly_in_class_body() {
    // Error: E0100 (ParsingError)
    let code = r#"
        class BadClass {
            if (true) {}
        }
    "#;
    assert_compile_error(code, "class");
}
