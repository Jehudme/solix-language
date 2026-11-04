//! MemberAccessExpression Conformance Tests (SPEC-0001 §14, TESTS.md Section 4.10).

#![forbid(unsafe_code)]

use solix_runtime::exceptions::RuntimeErrorCode;
use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, assert_runtime_error, compile_and_run};

#[test]
fn test_case_3_1_chained_member_access() {
    let code = r#"
        class Address {
            public int32 zip;
        }
        class Person {
            public Address addr;
        }

        int32 main() {
            Person p = new Person();
            p.addr = new Address();
            p.addr.zip = 90210;
            return p.addr.zip == 90210 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_2_static_field_access_on_class_name() {
    let code = r#"
        class MathConstants {
            public static int32 SCALE = 100;
        }

        int32 main() {
            return MathConstants::SCALE == 100 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_member_access_on_null_reference_runtime_fault() {
    let code = r#"
        class Address {
            public int32 zip;
        }
        class Person {
            public Address addr;
        }

        int32 main() {
            Person p = null;
            Address a = p.addr;
            return 0;
        }
    "#;
    assert_runtime_error(code, RuntimeErrorCode::NullReferenceAccess);
}

#[test]
fn test_case_4_2_accessing_non_existent_member_field() {
    let code = r#"
        class Empty {}

        void test() {
            Empty e = new Empty();
            int32 x = e.unknownField;
        }
    "#;
    assert_compile_error(code, "unknownField");
}

#[test]
fn test_case_4_4_using_super_keyword_in_non_derived_class() {
    let code = r#"
        class BaseOnly {
            public void test() {
                super.doSomething();
            }
        }
    "#;
    assert_compile_error(code, "super");
}
