//! ArrayLiteralExpression Conformance Tests (SPEC-0001 §14, TESTS.md Section 4.3).

#![forbid(unsafe_code)]

use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_3_1_dual_literal_syntax() {
    let code = r#"
        int32 main() {
            int32[] a = [10, 20];
            int32[] b = {10, 20};
            if (a[0] != b[0] || a[1] != b[1]) return 1;
            return 0;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_2_nested_2d_array_literal() {
    let code = r#"
        int32 main() {
            int32[][] matrix = {{1, 2}, {3, 4}};
            return (matrix[0][0] + matrix[1][1]) == 5 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_incompatible_literal_elements() {
    // E0203: Incompatible types in array literal
    let code = r#"
        void test() {
            int32[] arr = [10, "text"];
        }
    "#;
    assert_compile_error(code, "Incompatible types in array literal");
}

#[test]
fn test_case_4_2_array_literal_with_mixed_incompatible_types() {
    // E0203: Incompatible types in array literal
    let code = r#"
        void test() {
            int32[] arr = {1, "two", true};
        }
    "#;
    assert_compile_error(code, "Incompatible types in array literal");
}
