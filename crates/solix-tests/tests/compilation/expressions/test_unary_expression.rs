//! UnaryExpression Conformance Tests (SPEC-0001 §14, TESTS.md Section 4.14).

#![forbid(unsafe_code)]

use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_3_1_postfix_vs_prefix() {
    let code = r#"
        int32 main() {
            int32 x = 5;
            int32 post = x++; // post = 5, x = 6
            int32 pre = ++x;  // pre = 7, x = 7
            if (post == 5 && pre == 7 && x == 7) {
                return 0;
            }
            return 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_2_unary_negation_on_numeric_expressions() {
    let code = r#"
        int32 main() {
            int32 x = 42;
            int32 neg = -x;
            return neg == -42 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_3_logical_not_on_boolean_variable() {
    let code = r#"
        int32 main() {
            bool active = false;
            bool inverted = !active;
            return inverted == true ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_4_extreme_signed_boundary_negation() {
    let code = r#"
        int32 main() {
            int32 min32 = -2147483648;
            int32 neg32 = -min32;
            int64 min64 = -9223372036854775808L;
            int64 neg64 = -min64;
            if (neg32 == -2147483648 && neg64 == -9223372036854775808L) {
                return 0;
            }
            return 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_increment_on_constant() {
    // E0107: Invalid operand for increment
    let code = r#"
        int32 main() {
            ++10;
            return 0;
        }
    "#;
    assert_compile_error(code, "increment");
}

#[test]
fn test_case_4_2_unary_minus_on_non_numeric_operand() {
    // E0203: Invalid operand for unary operator
    let code = r#"
        class NonNumeric {}

        void test() {
            NonNumeric obj = new NonNumeric();
            NonNumeric neg = -obj;
        }
    "#;
    assert_compile_error(code, "unary operator");
}
