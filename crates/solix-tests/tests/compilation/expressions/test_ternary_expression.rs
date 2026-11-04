//! TernaryExpression Conformance Tests (SPEC-0001 §14, TESTS.md Section 4.13).

#![forbid(unsafe_code)]

use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_3_1_safe_guard_with_null_check() {
    let code = r#"
        class Holder {
            public int32 getVal() {
                return 42;
            }
        }

        int32 main() {
            Holder h = null;
            int32 len = (h != null) ? h.getVal() : 0;
            if (len != 0) {
                return 1;
            }

            h = new Holder();
            int32 len2 = (h != null) ? h.getVal() : 0;
            if (len2 != 42) {
                return 2;
            }

            return 0;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_2_nested_ternary_evaluation() {
    let code = r#"
        int32 classify(int32 x) {
            return x > 0 ? 1 : (x < 0 ? -1 : 0);
        }

        int32 main() {
            return (classify(10) == 1 && classify(-5) == -1 && classify(0) == 0) ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_non_boolean_condition() {
    // E0203: Ternary condition must be bool
    let code = r#"
        int32 main() {
            int32 res = 5 ? 1 : 2;
            return 0;
        }
    "#;
    assert_compile_error(code, "bool");
}

#[test]
fn test_case_4_2_mismatched_branch_types() {
    // E0203: Ternary branch types must match
    let code = r#"
        void test(bool cond) {
            int32 val = cond ? 42 : "string";
        }
    "#;
    assert_compile_error(code, "type");
}
