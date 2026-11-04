//! SizeOfExpression Conformance Tests (SPEC-0001 §14, TESTS.md Part IV Suite 38).

#![forbid(unsafe_code)]

use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_1_1_primitive_type_sizing() {
    let code = r#"
        int32 main() {
            int32 s1 = sizeof(int8);
            int32 s2 = sizeof(int16);
            int32 s4 = sizeof(int32);
            int32 s8 = sizeof(int64);
            int32 sf4 = sizeof(float32);
            int32 sf8 = sizeof(float64);
            int32 sb = sizeof(bool);
            int32 sc = sizeof(char);
            if (s1 == 1 && s2 == 2 && s4 == 4 && s8 == 8 && sf4 == 4 && sf8 == 8 && sb == 1 && sc == 1) {
                return 0;
            }
            return 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_1_2_class_type_compile_time_sizing() {
    let code = r#"
        class Point {
            int32 x;
            int32 y;
        }
        int32 main() {
            int32 sz = sizeof(Point);
            return sz >= 16 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_2_1_dynamic_instance_sizing_on_heap_objects() {
    let code = r#"
        class Item {
            int32 a;
            int32 b;
        }
        int32 main() {
            Item item = new Item();
            int32 sz = sizeof(item);
            return sz >= 16 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_1_sizeof_on_undeclared_identifier() {
    // E0201: Undefined symbol in sizeof
    let code = r#"
        void test() {
            int32 s = sizeof(NonExistentType);
        }
    "#;
    assert_compile_error(code, "NonExistentType");
}
