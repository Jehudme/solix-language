//! BinaryExpression Conformance Tests (SPEC-0001 §14, TESTS.md Section 4.5).

#![forbid(unsafe_code)]

use solix_runtime::exceptions::RuntimeErrorCode;
use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, assert_runtime_error, compile_and_run};

#[test]
fn test_case_3_1_short_circuit_logical_and() {
    let code = r#"
        int32 main() {
            bool result = false && (10 / 0 == 0);
            return result ? 1 : 0;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_2_short_circuit_logical_or() {
    let code = r#"
        class Main {
            static int32 side_effects = 0;
            static bool get_false() { Main::side_effects = Main::side_effects + 1; return false; }
            static bool get_true() { Main::side_effects = Main::side_effects + 1; return true; }

            public static int32 main() {
                bool res = Main::get_true() || Main::get_false();
                return (res && Main::side_effects == 1) ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_3_relational_comparisons() {
    let code = r#"
        int32 main() {
            int32 a = 10;
            int32 b = 20;
            if (a < b && a <= 10 && b > a && b >= 20) {
                return 0;
            }
            return 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_4_extreme_boundary_arithmetic_and_signed_limits() {
    let code = r#"
        int32 main() {
            int64 min64 = -9223372036854775808L;
            int64 divResult = min64 / -1;
            int64 modResult = min64 % -1;

            int64 max64 = 9223372036854775807L;
            int64 divMax = max64 / -1;

            int32 min32 = -2147483648;
            int32 div32 = min32 / 2;

            if (divResult == min64 && modResult == 0 &&
                divMax == -max64 && div32 == -1073741824) {
                return 0;
            }
            return 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_division_by_zero_runtime_fault() {
    let code = r#"
        int32 main() {
            int32 x = 10 / 0;
            return x;
        }
    "#;
    assert_runtime_error(code, RuntimeErrorCode::DivideByZero);
}

#[test]
fn test_case_4_2_incompatible_arithmetic_types() {
    // E0203: Operator '+' incompatible types
    let code = r#"
        void test() {
            int32 x = 10 + true;
        }
    "#;
    assert_compile_error(code, "+");
}
