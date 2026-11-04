//! AssignmentExpression Conformance Tests (SPEC-0001 §14, TESTS.md Section 4.4).

#![forbid(unsafe_code)]

use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_3_1_chained_assignment() {
    let code = r#"
        int32 main() {
            int32 a = 0;
            int32 b = 0;
            int32 c = 0;
            a = b = c = 10;
            if (a != 10 || b != 10 || c != 10) return 1;
            return 0;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_2_compound_assignments() {
    let code = r#"
        int32 main() {
            int32 x = 10;
            x += 5;
            x -= 3;
            x *= 2;
            x /= 4;
            return x == 6 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_3_assigning_to_object_field_target() {
    let code = r#"
        class Box {
            public int32 weight;
        }

        int32 main() {
            Box b = new Box();
            b.weight = 50;
            return b.weight == 50 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_assigning_to_literal_rvalue() {
    // E0205: Invalid assignment target
    let code = r#"
        void test() {
            int32 x = 0;
            10 = x;
        }
    "#;
    assert_compile_error(code, "assignment target");
}

#[test]
fn test_case_4_2_compound_assignment_type_mismatch() {
    // E0203: Cannot apply compound operator '+=' to mismatched types
    let code = r#"
        void test() {
            int32 x = 10;
            x += "text";
        }
    "#;
    assert_compile_error(code, "mismatch");
}
