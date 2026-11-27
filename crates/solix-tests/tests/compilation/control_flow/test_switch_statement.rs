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

#[test]
fn test_switch_continue_inside_loop() {
    let source_code = r#"
        int32 main() {
            int32 loop_counter = 0;
            int32 accumulated_value = 0;
            while (loop_counter < 5) {
                loop_counter = loop_counter + 1;
                switch (loop_counter) {
                    case 2:
                        continue;
                    case 4:
                        continue;
                    default:
                        accumulated_value = accumulated_value + loop_counter;
                }
            }
            return accumulated_value;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(9));
}

#[test]
fn test_grouped_empty_case_labels_fallthrough_and_definite_return() {
    // Regression test (#300.4): grouped empty case labels (`case 1: case 2: ...` and
    // `case 3: default: ...`) must execute the shared body directly and satisfy definite return
    // and definite assignment checks.
    let source_code = r#"
        int32 classify(int32 code) {
            switch (code) {
                case 1:
                case 2:
                    return 5;
                case 3:
                case 4:
                default:
                    return 9;
            }
        }

        int32 assign_grouped(int32 code) {
            int32 bucket;
            switch (code) {
                case 10:
                case 20:
                    bucket = 77;
                    break;
                default:
                    bucket = 88;
                    break;
            }
            return bucket;
        }

        int32 main() {
            if (classify(1) != 5 || classify(2) != 5) return 1;
            if (classify(3) != 9 || classify(4) != 9 || classify(99) != 9) return 2;
            if (assign_grouped(10) != 77 || assign_grouped(20) != 77) return 3;
            if (assign_grouped(99) != 88) return 4;
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}
