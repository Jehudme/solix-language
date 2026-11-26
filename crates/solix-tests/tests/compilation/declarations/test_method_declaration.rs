//! Suite: MethodDeclaration - Declarations (TESTS.md)

use solix_runtime::Value;
use solix_tests::common::{assert_compile_error, compile_and_run, compile_source_get_diagnostics};

#[test]
fn test_case_3_1_virtual_method_overriding() {
    let code = r#"
        class Parent {
            public virtual int32 get_val() { return 1; }
        }
        class Child extends Parent {
            public override int32 get_val() { return 2; }
        }

        class Main {
            public static int32 main() {
                Parent p = new Child();
                return p.get_val() == 2 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_2_static_utility_method_invocation() {
    let code = r#"
        class MathUtil {
            public static int32 add(int32 a, int32 b) {
                return a + b;
            }
        }

        class Main {
            public static int32 main() {
                return MathUtil::add(20, 22) == 42 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_3_method_overloading_with_multiple_types() {
    let code = r#"
        class Calculator {
            public int32 compute(int32 x) { return x * 2; }
            public float64 compute(float64 x) { return x * 2.0; }
        }

        class Main {
            public static int32 main() {
                Calculator c = new Calculator();
                return (c.compute(10) == 20 && c.compute(1.5) == 3.0) ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_4_protected_method_accessible_in_subclass() {
    let code = r#"
        class BaseWorker {
            protected int32 getCode() { return 100; }
        }
        class DerivedWorker extends BaseWorker {
            public int32 test() { return this.getCode(); }
        }

        class Main {
            public static int32 main() {
                DerivedWorker w = new DerivedWorker();
                return w.test() == 100 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_abstract_method_with_body() {
    // Error: E0100 / E0202
    let code = r#"
        abstract class Base {
            abstract void run() {}
        }
    "#;
    assert_compile_error(code, "body");
}

#[test]
fn test_case_4_2_calling_private_method_from_outside_class() {
    // Error: E0208 (PrivateAccessViolation)
    let code = r#"
        class Encapsulated {
            private void secret() {}
        }

        class Main {
            public static void test() {
                Encapsulated e = new Encapsulated();
                e.secret();
            }
        }
    "#;
    assert_compile_error(code, "E0208");
}

#[test]
fn test_case_4_3_missing_return_statement_in_non_void_method() {
    // Error: E0212 (MissingReturn)
    let code = r#"
        class Main {
            public static int32 badMethod(bool flag) {
                if (flag) {
                    return 1;
                }
            }
        }
    "#;
    assert_compile_error(code, "E0212");
}

#[test]
fn test_case_4_4_incompatible_override_signature_return_type() {
    // Error: E0207 (MethodOverrideMismatch)
    let code = r#"
        class SuperClass {
            public virtual int32 getValue() { return 0; }
        }
        class SubClass extends SuperClass {
            public override string getValue() { return "bad"; }
        }
    "#;
    assert_compile_error(code, "E0207");
}

#[test]
fn test_case_4_7_nested_method_declaration_inside_another_method() {
    // Error: E0123 (MethodInsideMethod)
    let code = r#"
        class Main {
            public static void outer() {
                void inner() {}
            }
        }
    "#;
    assert_compile_error(code, "E0123");
}

#[test]
fn test_case_4_8_missing_return_suppressed_when_method_has_syntax_errors() {
    let code = r#"
        class Main {
            public static int32 badMethod() {
                whilje(true) {
                };
            }
        }
    "#;
    let diagnostics = compile_source_get_diagnostics(code);
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic_item| diagnostic_item.error_code() == "E0158"),
        "Must emit E0158 for whilje keyword typo, got: {diagnostics:?}"
    );
    assert!(
        !diagnostics
            .iter()
            .any(|diagnostic_item| diagnostic_item.error_code() == "E0212"),
        "Must NOT emit false cascading E0212 (MissingReturn) when method has syntax errors, got: {diagnostics:?}"
    );
}
