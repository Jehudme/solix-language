//! ContinueStatement Conformance Tests (SPEC-0001 §12, TESTS.md Section 2.3).

#![forbid(unsafe_code)]

use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_3_1_continue_advances_loop_variable() {
    let code = r#"
        int32 main() {
            int32 hits = 0;
            for (int32 i = 0; i < 6; i++) {
                if (i % 2 == 0) continue;
                hits++;
            }
            return hits;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(3));
}

#[test]
fn test_case_3_2_continue_in_while_loop() {
    let code = r#"
        int32 main() {
            int32 i = 0;
            int32 count = 0;
            while (i < 10) {
                i++;
                if (i % 2 == 0) {
                    continue;
                }
                count++;
            }
            return count == 5 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_3_continue_cleans_up_scoped_reference_registers() {
    let source_code = r#"
        class ScopedNode {
            public int32 value;
            public ScopedNode(int32 initial_val) {
                this.value = initial_val;
            }
        }

        int32 main() {
            int32 sum = 0;
            for (int32 iteration = 0; iteration < 8; iteration++) {
                ScopedNode node = new ScopedNode(iteration);
                if (iteration % 2 == 0) {
                    continue;
                }
                sum += node.value;
            }
            return sum;
        }
    "#;
    let (return_value, _) = compile_and_run(source_code);
    assert_eq!(return_value, Value::I32(1 + 3 + 5 + 7));
}

#[test]
fn test_case_4_1_continue_outside_loop() {
    // Error: E0100 / E0200
    let code = r#"
        void test() {
            continue;
        }
    "#;
    assert_compile_error(code, "continue");
}

#[test]
fn test_case_4_2_continue_inside_switch_not_in_loop() {
    // Error: E0100 / E0200
    let code = r#"
        void test(int32 x) {
            switch (x) {
                case 1:
                    continue;
            }
        }
    "#;
    assert_compile_error(code, "continue");
}

#[test]
fn test_case_4_3_continue_statement_placed_directly_in_class_body() {
    // Error: E0100 (ParsingError)
    let code = r#"
        class BadClass {
            continue;
        }
    "#;
    assert_compile_error(code, "class body");
}

#[test]
fn test_case_4_4_continue_inside_if_not_enclosed_in_loop() {
    // Error: E0100 / E0200
    let code = r#"
        void test(bool flag) {
            if (flag) {
                continue;
            }
        }
    "#;
    assert_compile_error(code, "continue");
}
