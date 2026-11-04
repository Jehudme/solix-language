//! BreakStatement Conformance Tests (SPEC-0001 §12, TESTS.md Section 2.2).

#![forbid(unsafe_code)]

use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_3_1_breaking_out_of_deep_nested_blocks() {
    let code = r#"
        class Res {
            public int32 id;
            public Res(int32 id) { this.id = id; }
        }
        int32 main() {
            int32 count = 0;
            while (true) {
                Res s1 = new Res(1);
                {
                    Res s2 = new Res(2);
                    count = 42;
                    break;
                }
            }
            return count;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(42));
}

#[test]
fn test_case_3_2_break_in_for_loop_preserves_state() {
    let code = r#"
        int32 main() {
            int32 sum = 0;
            for (int32 i = 0; i < 10; i++) {
                if (i == 5) {
                    break;
                }
                sum += i;
            }
            return sum == 10 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_3_break_inside_switch_statement() {
    let code = r#"
        int32 main() {
            int32 x = 2;
            int32 res = 0;
            switch (x) {
                case 1: res = 10; break;
                case 2: res = 20; break;
                default: res = 30; break;
            }
            return res == 20 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_break_outside_loop_or_switch() {
    // Error: E0100 / E0200
    let code = r#"
        void test() {
            int32 x = 10;
            break;
        }
    "#;
    assert_compile_error(code, "break");
}

#[test]
fn test_case_4_2_break_at_function_top_level() {
    // Error: E0100 / E0200
    let code = r#"
        void test() {
            break;
        }
    "#;
    assert_compile_error(code, "break");
}

#[test]
fn test_case_4_3_break_statement_placed_directly_in_class_body() {
    // Error: E0100 (ParsingError)
    let code = r#"
        class BadClass {
            break;
        }
    "#;
    assert_compile_error(code, "class body");
}

#[test]
fn test_case_4_4_break_inside_if_not_enclosed_in_loop_or_switch() {
    // Error: E0100 / E0200
    let code = r#"
        void test(bool flag) {
            if (flag) {
                break;
            }
        }
    "#;
    assert_compile_error(code, "break");
}
