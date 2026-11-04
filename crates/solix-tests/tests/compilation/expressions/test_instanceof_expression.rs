//! InstanceOfExpression Conformance Tests (SPEC-0001 §14, TESTS.md Section 4.8).

#![forbid(unsafe_code)]

use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_3_1_safe_null_evaluation() {
    let code = r#"
        class Animal {}
        class Dog extends Animal {}

        int32 main() {
            Animal a = null;
            bool check = a instanceof Dog;
            return check ? 1 : 0;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_2_instanceof_subclass_evaluates_true_for_superclass() {
    let code = r#"
        class Base {}
        class Sub extends Base {}

        int32 main() {
            Base obj = new Sub();
            if (obj instanceof Base && obj instanceof Sub) {
                return 0;
            }
            return 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_3_primitive_array_evaluates_false_for_class_instanceof() {
    let code = r#"
        class Dog {}

        int32 main() {
            int32[] arr = new int32[5];
            arr[0] = 1;
            bool check = arr instanceof Dog;
            return check ? 1 : 0;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_primitive_target() {
    let code = r#"
        void test() {
            bool b = 10 instanceof int32;
        }
    "#;
    assert_compile_error(code, "instanceof");
}

#[test]
fn test_case_4_2_instanceof_with_undeclared_type_name() {
    let code = r#"
        class Object {}
        void test(Object o) {
            bool b = o instanceof NonExistentClass;
        }
    "#;
    assert_compile_error(code, "NonExistentClass");
}
