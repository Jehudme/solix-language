//! TryCatchFinallyStatement Conformance Tests (SPEC-0001 §12, TESTS.md Section 2.11).

#![forbid(unsafe_code)]

use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_3_1_specific_catch_hierarchy() {
    let code = r#"
        class Exception {}
        class CustomError extends Exception {}

        int32 main() {
            int32 result = 0;
            try {
                throw new CustomError();
            } catch (CustomError c) {
                result = 1;
            } catch (Exception e) {
                result = 2;
            }
            return result;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(1));
}

#[test]
fn test_case_3_2_try_block_with_only_finally_clause() {
    let code = r#"
        class Main {
            public static int32 cleanup_marker = 0;

            public static void work() {
                try {
                    Main::cleanup_marker += 10;
                } finally {
                    Main::cleanup_marker += 20;
                }
            }

            public static int32 main() {
                Main::work();
                return Main::cleanup_marker == 30 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_3_exception_in_catch_with_guaranteed_finally_execution() {
    let code = r#"
        class Exception {}

        class Main {
            public static bool finally_ran = false;

            public static void faulty() {
                try {
                    throw new Exception();
                } catch (Exception e) {
                    throw new Exception();
                } finally {
                    Main::finally_ran = true;
                }
            }

            public static int32 main() {
                try {
                    Main::faulty();
                } catch (Exception e) {}
                return Main::finally_ran ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_unreachable_catch_clause() {
    // Error: E0200 / E0202
    let code = r#"
        class Exception {}
        class SubErr extends Exception {}

        void test() {
            try {
                int32 x = 0;
            } catch (Exception e) {
            } catch (SubErr s) {
            }
        }
    "#;
    assert_compile_error(code, "catch");
}

#[test]
fn test_case_4_2_catching_non_exception_type() {
    // Error: E0202 (TypeMismatch)
    let code = r#"
        void test() {
            try {} catch (int32 x) {}
        }
    "#;
    assert_compile_error(code, "catch");
}

#[test]
fn test_case_4_3_duplicate_catch_clause_for_same_type() {
    // Error: E0201 (DuplicateSymbol)
    let code = r#"
        class Exception {}

        void test() {
            try {}
            catch (Exception e) {}
            catch (Exception e2) {}
        }
    "#;
    assert_compile_error(code, "catch");
}

#[test]
fn test_case_4_5_try_catch_statement_placed_directly_in_class_body() {
    // Error: E0100 (ParsingError)
    let code = r#"
        class BadClass {
            try {} catch (Exception e) {}
        }
    "#;
    assert_compile_error(code, "class");
}
