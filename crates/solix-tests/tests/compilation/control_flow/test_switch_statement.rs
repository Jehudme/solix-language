//! SwitchStatement Conformance Tests (SPEC-0001 §12, TESTS.md Section 2.9).

#![forbid(unsafe_code)]

use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_3_1_switch_with_explicit_break() {
    let code = r#"
        int32 main() {
            int32 value = 2;
            int32 res = 0;
            switch (value) {
                case 1: res = 10; break;
                case 2: res = 20; break;
                default: res = 30; break;
            }
            return res;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(20));
}

#[test]
fn test_case_3_2_switch_on_strongly_typed_enum() {
    let code = r#"
        enum Status { PENDING, APPROVED, REJECTED }

        int32 evaluate(Status s) {
            switch (s) {
                case Status.PENDING: return 1;
                case Status.APPROVED: return 2;
                case Status.REJECTED: return 3;
                default: return 0;
            }
        }

        int32 main() {
            return evaluate(Status.APPROVED) == 2 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_duplicate_case_constant() {
    // Error: E0201 (DuplicateSymbol / DuplicateCase)
    let code = r#"
        void test(int32 x) {
            switch (x) {
                case 1: break;
                case 1: break;
            }
        }
    "#;
    assert_compile_error(code, "Duplicate");
}

#[test]
fn test_case_4_2_variable_expression_in_case_label() {
    // Error: E0100 / E0200
    let code = r#"
        void test(int32 x, int32 dynamicVal) {
            switch (x) {
                case dynamicVal: break;
            }
        }
    "#;
    assert_compile_error(code, "constant");
}

#[test]
fn test_case_4_4_switch_statement_placed_directly_in_class_body() {
    // Error: E0100 (ParsingError)
    let code = r#"
        class BadClass {
            switch (1) { case 1: break; }
        }
    "#;
    assert_compile_error(code, "class");
}
