//! DoWhileStatement Conformance Tests (SPEC-0001 §12, TESTS.md Section 2.4).

#![forbid(unsafe_code)]

use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_3_1_guaranteed_initial_pass_with_false_condition() {
    let code = r#"
        int32 main() {
            int32 ran = 0;
            do {
                ran++;
            } while (false);
            return ran;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(1));
}

#[test]
fn test_case_3_2_multi_pass_iteration_and_condition_evaluation() {
    let code = r#"
        int32 main() {
            int32 sum = 0;
            int32 i = 1;
            do {
                sum += i;
                i++;
            } while (i <= 5);
            return sum == 15 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_accessing_body_variable_in_condition() {
    // Error: E0200 (UndefinedIdentifier)
    let code = r#"
        void test() {
            do {
                int32 inner = 10;
            } while (inner > 0);
        }
    "#;
    assert_compile_error(code, "inner");
}

#[test]
fn test_case_4_2_non_boolean_condition_in_do_while() {
    // Error: E0202 (TypeMismatch)
    let code = r#"
        void test() {
            do {} while (42);
        }
    "#;
    assert_compile_error(code, "bool");
}

#[test]
fn test_case_4_3_do_while_statement_placed_directly_in_class_body() {
    // Error: E0100 (ParsingError)
    let code = r#"
        class BadClass {
            do {} while (true);
        }
    "#;
    assert_compile_error(code, "class");
}
