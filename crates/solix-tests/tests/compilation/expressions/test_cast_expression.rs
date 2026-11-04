//! CastExpression Conformance Tests (SPEC-0001 §14, TESTS.md Section 4.6).

#![forbid(unsafe_code)]

use solix_runtime::exceptions::RuntimeErrorCode;
use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, assert_runtime_error, compile_and_run};

#[test]
fn test_case_3_1_valid_downcast() {
    let code = r#"
        class Animal {
            public virtual int32 speak() { return 1; }
        }
        class Dog extends Animal {
            public override int32 speak() { return 7; }
        }

        int32 main() {
            Animal a = new Dog();
            Dog d = (Dog)a;
            return d.speak();
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(7));
}

#[test]
fn test_case_3_2_numeric_widening_and_narrowing_conversions() {
    let code = r#"
        int32 main() {
            int32 small = 42;
            int64 big = (int64)small;
            int8 tiny = (int8)small;
            return (big == 42 && tiny == 42) ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_4_casting_null_literal_to_reference_type() {
    let code = r#"
        class Person {}

        int32 main() {
            Person p = (Person)null;
            return p == null ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_bad_downcast_runtime_fault() {
    let code = r#"
        class Animal {
            public virtual int32 speak() { return 1; }
        }
        class Dog extends Animal {
            public override int32 speak() { return 7; }
        }
        class Cat extends Animal {
            public override int32 speak() { return 3; }
        }

        int32 main() {
            Animal a = new Cat();
            Dog d = (Dog)a;
            return 0;
        }
    "#;
    assert_runtime_error(code, RuntimeErrorCode::InvalidCast);
}

#[test]
fn test_case_4_2_compile_time_rejection_of_unrelated_class_cast() {
    // E0207: Cannot cast between unrelated types 'Cat' and 'Dog'
    let code = r#"
        class Cat {}
        class Dog {}

        void test() {
            Cat c = new Cat();
            Dog d = (Dog)c;
        }
    "#;
    assert_compile_error(code, "unrelated");
}

#[test]
fn test_case_4_4_cast_expression_with_non_type_identifier() {
    let code = r#"
        void test(int32 x) {
            int32 y = (123)x;
        }
    "#;
    assert_compile_error(code, "E0193");
}
