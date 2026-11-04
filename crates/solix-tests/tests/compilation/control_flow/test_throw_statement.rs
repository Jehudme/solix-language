//! ThrowStatement Conformance Tests (SPEC-0001 §12, TESTS.md Section 2.10).

#![forbid(unsafe_code)]

use solix_runtime::{RuntimeContext, RuntimeError, RuntimeOptions, Value, VirtualMachine};
use solix_tests::common::{assert_compile_error, compile_and_run, compile_source};

#[test]
fn test_case_3_1_throw_handled_by_catch() {
    let code = r#"
        class Exception {}
        class MyException extends Exception {}

        int32 main() {
            int32 caught = 0;
            try {
                throw new MyException();
            } catch (MyException e) {
                caught = 1;
            }
            return caught;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(1));
}

#[test]
fn test_case_3_2_rethrowing_caught_exception_instance() {
    let code = r#"
        class Exception {}

        int32 main() {
            bool caught_outer = false;
            try {
                try {
                    throw new Exception();
                } catch (Exception e) {
                    throw e;
                }
            } catch (Exception outer) {
                caught_outer = true;
            }
            return caught_outer ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_throwing_primitive_value() {
    // Error: E0202 (TypeMismatch)
    let code = r#"
        void test() {
            throw 404;
        }
    "#;
    assert_compile_error(code, "throw");
}

#[test]
fn test_case_4_2_throwing_null_reference_at_runtime() {
    // Runtime Error: NullReferenceAccess
    let code = r#"
        class Exception {}
        int32 main() {
            Exception e = null;
            throw e;
            return 0;
        }
    "#;
    let program = compile_source(code);
    let options = RuntimeOptions::new().with_binary(program);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);
    let err = vm.run().unwrap_err();
    assert!(matches!(err, RuntimeError::NullReferenceAccess));
}

#[test]
fn test_case_4_3_throwing_uninstantiated_class_identifier() {
    // Error: E0200 / E0202
    let code = r#"
        class Exception {}
        void test() {
            throw Exception;
        }
    "#;
    assert_compile_error(code, "throw");
}

#[test]
fn test_case_4_5_throw_statement_placed_directly_in_class_body() {
    // Error: E0100 (ParsingError)
    let code = r#"
        class Exception {}
        class BadClass {
            throw new Exception();
        }
    "#;
    assert_compile_error(code, "class");
}
