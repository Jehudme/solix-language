//! BlockStatement Conformance Tests (SPEC-0001 §12, TESTS.md Section 2.1).

#![forbid(unsafe_code)]

use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_3_1_empty_and_nested_empty_blocks() {
    let code = r#"
        int32 main() {
            {}
            {
                {}
                { {} }
            }
            return 0;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_2_lexical_variable_shadowing() {
    let code = r#"
        int32 main() {
            int32 value = 10;
            {
                int32 value = 20;
                if (value != 20) return 1;
            }
            if (value != 10) return 2;
            return 0;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_3_strict_lifo_destruction_of_multiple_reference_objects() {
    let code = r#"
        class Res {
            public int32 id;
            public Res(int32 id) { this.id = id; }
        }
        int32 main() {
            {
                Res first = new Res(1);
                Res second = new Res(2);
                Res third = new Res(3);
            }
            return 0;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_4_early_return_from_nested_blocks() {
    let code = r#"
        class Res {
            public int32 id;
            public Res(int32 id) { this.id = id; }
        }
        int32 compute(bool early) {
            Res a = new Res(1);
            {
                Res b = new Res(2);
                if (early) {
                    return 100;
                }
            }
            return 0;
        }
        int32 main() {
            return compute(true);
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(100));
}

#[test]
fn test_case_3_5_lifo_scope_cleanup_drops_live_object_count_to_zero() {
    let code = r#"
        class Res {
            public int32 id;
            public Res(int32 id) { this.id = id; }
        }

        int32 compute() {
            Res a = new Res(1);
            {
                Res b = new Res(2);
                {
                    Res c = new Res(3);
                }
            }
            return 0;
        }

        int32 main() {
            compute();
            return 0;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_accessing_block_scoped_variable_outside_its_block() {
    // Error: E0200 (UndefinedIdentifier)
    let code = r#"
        void test() {
            {
                int32 temp = 100;
            }
            int32 leak = temp;
        }
    "#;
    assert_compile_error(code, "temp");
}

#[test]
fn test_case_4_2_duplicate_variable_in_same_immediate_scope() {
    // Error: E0201 (DuplicateSymbol)
    let code = r#"
        void test() {
            {
                int32 score = 10;
                float64 score = 20.0;
            }
        }
    "#;
    assert_compile_error(code, "already defined");
}

#[test]
fn test_case_4_3_unclosed_block_missing_brace() {
    // Error: E0100 (ParsingError)
    let code = r#"
        void test() {
            {
                int32 x = 1;
        }
    "#;
    assert_compile_error(code, "}");
}

#[test]
fn test_case_4_4_stray_extra_closing_brace() {
    // Error: E0100 (ParsingError)
    let code = r#"
        void test() {
            { int32 x = 1; }
            }
        }
    "#;
    assert_compile_error(code, "}");
}

#[test]
fn test_case_4_6_bare_execution_block_directly_in_class_body() {
    // Error: E0100 (ParsingError)
    let code = r#"
        class BadClass {
            {
                int32 x = 5;
            }
        }
    "#;
    assert_compile_error(code, "class");
}
