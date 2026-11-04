//! IdentifierNode Conformance Tests (SPEC-0001 §14, TESTS.md Section 4.7).

#![forbid(unsafe_code)]

use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_3_1_local_resolution_precedence() {
    let code = r#"
        int32 main() {
            int32 val = 100;
            {
                int32 val = 200;
                return val;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(200));
}

#[test]
fn test_case_3_2_explicit_member_access_via_this_identifier() {
    let code = r#"
        class ScopeTest {
            public int32 val;
            public void setVal(int32 val) {
                this.val = val;
            }
        }
        int32 main() {
            ScopeTest s = new ScopeTest();
            s.setVal(42);
            return s.val;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(42));
}

#[test]
fn test_case_4_1_undefined_identifier() {
    // E0201: Undefined identifier
    let code = r#"
        void test() {
            int32 a = unknown_var;
        }
    "#;
    assert_compile_error(code, "unknown_var");
}

#[test]
fn test_case_4_2_accessing_local_variable_before_declaration() {
    // E0201: Undefined identifier
    let code = r#"
        void test() {
            x = 10;
            int32 x = 0;
        }
    "#;
    assert_compile_error(code, "x");
}

#[test]
fn test_case_4_3_using_this_keyword_outside_any_class() {
    // E0202: 'this' outside class
    let code = r#"
        void test() {
            int32 x = this.val;
        }
    "#;
    assert_compile_error(code, "this");
}

#[test]
fn test_case_4_4_using_this_keyword_inside_static_method() {
    // E0202: 'this' inside static method
    let code = r#"
        class Example {
            public static void staticMethod() {
                int32 x = this.val;
            }
        }
    "#;
    assert_compile_error(code, "this");
}
