//! Language Intrinsics (assert and exit) Conformance Tests (TESTS.md Section 50).

#![forbid(unsafe_code)]

use solix_runtime::exceptions::RuntimeErrorCode;
use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, assert_runtime_error, compile_and_run};

#[test]
fn test_case_50_1_basic_boolean_assertion_passing_silently() {
    let code = r#"
        class Main {
            public static int32 main() {
                assert(true);
                assert(1 + 1 == 2);
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_50_2_assertion_with_custom_textual_message_passing_silently() {
    let code = r#"
        class Main {
            public static int32 main() {
                assert(10 > 5, "Arithmetic consistency holds");
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_50_3_failed_assertion_throws_assertion_error_runtime_exception() {
    let code = r#"
        class Main {
            public static int32 main() {
                assert(2 + 2 == 5, "Math verification failed");
                return 0;
            }
        }
    "#;
    assert_runtime_error(code, RuntimeErrorCode::UserException);
}

#[test]
fn test_case_50_4_immediate_process_termination_via_exit_code_intrinsics() {
    let code = r#"
        class Main {
            public static int32 main() {
                exit(42);
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(42));
}

#[test]
fn test_case_50_5_assert_condition_non_boolean_type_mismatch() {
    let code = r#"
        class Main {
            public static int32 main() {
                assert(123);
                return 0;
            }
        }
    "#;
    assert_compile_error(code, "bool");
}

#[test]
fn test_case_50_6_exit_code_non_integer_type_mismatch() {
    let code = r#"
        class Main {
            public static int32 main() {
                exit("fatal");
                return 0;
            }
        }
    "#;
    assert_compile_error(code, "int32");
}
