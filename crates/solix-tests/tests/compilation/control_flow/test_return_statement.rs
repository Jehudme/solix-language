//! ReturnStatement Conformance Tests (SPEC-0001 §12, TESTS.md Section 2.8).

#![forbid(unsafe_code)]

use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_3_1_early_return_from_nested_scopes() {
    let code = r#"
        class Res {
            public int32 id;
            public Res(int32 id) { this.id = id; }
        }
        int32 find(bool fast) {
            Res a = new Res(1);
            {
                Res b = new Res(2);
                if (fast) return 1;
            }
            return 0;
        }
        int32 main() {
            return find(true);
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(1));
}

#[test]
fn test_case_3_2_return_from_inside_try_finally_executing_finally_first() {
    let code = r#"
        class Main {
            public static int32 result = 0;

            public static int32 compute() {
                try {
                    return 42;
                } finally {
                    Main::result = 100;
                }
            }

            public static int32 main() {
                int32 val = Main::compute();
                return (val == 42 && Main::result == 100) ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_missing_return_value_in_non_void_method() {
    // Error: E0202 (TypeMismatch)
    let code = r#"
        int32 get_val() {
            return;
        }
    "#;
    assert_compile_error(code, "return");
}

#[test]
fn test_case_4_2_return_type_mismatch() {
    // Error: E0202 (TypeMismatch)
    let code = r#"
        int32 get_num() {
            return "text";
        }
    "#;
    assert_compile_error(code, "return");
}

#[test]
fn test_case_4_3_returning_value_from_void_method() {
    // Error: E0202 (TypeMismatch)
    let code = r#"
        void test() {
            return 42;
        }
    "#;
    assert_compile_error(code, "void");
}

#[test]
fn test_case_4_5_return_statement_placed_at_file_top_level() {
    // Error: E0100 (ParsingError)
    let code = r#"
        return 0;
    "#;
    assert_compile_error(code, "return");
}

#[test]
fn test_case_4_6_return_statement_placed_directly_in_class_body() {
    // Error: E0100 (ParsingError)
    let code = r#"
        class BadClass {
            return 0;
        }
    "#;
    assert_compile_error(code, "class body");
}
