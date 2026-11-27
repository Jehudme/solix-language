//! VariableDeclarationStatement Conformance Tests (SPEC-0001 §12, TESTS.md Section 2.12).

#![forbid(unsafe_code)]

use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_3_1_primitive_and_reference_declarations() {
    let code = r#"
        int32 main() {
            int32 a = 1;
            float64 b = 2.5;
            bool c = true;
            int32[] nums = new int32[5];
            return a;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(1));
}

#[test]
fn test_case_3_2_polymorphic_upcasting() {
    let code = r#"
        class Animal {}
        class Cat extends Animal {}

        int32 main() {
            Animal a = new Cat();
            return 0;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_3_interface_binding() {
    let code = r#"
        interface Printable { void print(); }
        class Doc implements Printable { public void print() {} }

        int32 main() {
            Printable p = new Doc();
            return 0;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_4_multiple_declarations_on_single_line() {
    let code = r#"
        int32 main() {
            int32 a = 1, b = 2, c = 3;
            return (a + b + c) == 6 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_5_constant_local_variable_declaration() {
    let code = r#"
        int32 main() {
            const int32 LIMIT = 100;
            return LIMIT == 100 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_6_file_scope_global_variable_declaration() {
    let code = r#"
        int32 global_counter = 45;

        int32 main() {
            return global_counter == 45 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_type_mismatch_in_initializer() {
    // Error: E0202 (TypeMismatch)
    let code = r#"
        void test() {
            int32 x = "hello";
        }
    "#;
    assert_compile_error(code, "type");
}

#[test]
fn test_case_4_2_duplicate_declaration_in_same_scope() {
    // Error: E0201 (DuplicateSymbol)
    let code = r#"
        void test() {
            int32 score = 10;
            int32 score = 20;
        }
    "#;
    assert_compile_error(code, "already defined");
}

#[test]
fn test_case_4_3_unknown_type_name() {
    // Error: E0200 (UnknownType)
    let code = r#"
        void test() {
            NonExistentType obj = null;
        }
    "#;
    assert_compile_error(code, "NonExistentType");
}

#[test]
fn test_case_4_4_declaring_variable_as_void() {
    // Error: E0202 (TypeMismatch)
    let code = r#"
        void test() {
            void placeholder;
        }
    "#;
    assert_compile_error(code, "void");
}

#[test]
fn test_case_4_5_reassigning_const_local_variable() {
    // Error: E0202 / E0203 (ConstMutation)
    let code = r#"
        void test() {
            const int32 x = 10;
            x = 20;
        }
    "#;
    assert_compile_error(code, "const");
}

#[test]
fn test_case_4_6_assigning_null_to_primitive_type() {
    // Error: E0202 (TypeMismatch)
    let code = r#"
        void test() {
            int32 x = null;
        }
    "#;
    assert_compile_error(code, "null");
}

#[test]
fn test_case_4_7_solitary_variable_declaration_in_if_without_braces() {
    // Error: E0100 (ParsingError)
    let code = r#"
        void test() {
            if (true)
                int32 x = 5;
        }
    "#;
    assert_compile_error(code, "solitary");
}

#[test]
fn test_case_4_8_solitary_variable_declaration_in_while_without_braces() {
    // Error: E0100 (ParsingError)
    let code = r#"
        void test() {
            while (true)
                int32 x = 5;
        }
    "#;
    assert_compile_error(code, "solitary");
}

#[test]
fn test_uninitialized_local_read_emits_e0272() {
    // Regression test (#300.5): reading an uninitialized local variable must emit E0272.
    let direct_read = r#"
        int32 test() {
            int32 uninit;
            int32 copy = uninit;
            return copy;
        }
    "#;
    assert_compile_error(direct_read, "E0272");

    let one_branch = r#"
        int32 test(bool flag) {
            int32 value;
            if (flag) {
                value = 1;
            }
            return value;
        }
    "#;
    assert_compile_error(one_branch, "E0272");

    let loop_body_only = r#"
        int32 test(bool cond) {
            int32 acc;
            while (cond) {
                acc = 5;
            }
            return acc;
        }
    "#;
    assert_compile_error(loop_body_only, "E0272");

    let compound_inc = r#"
        int32 test() {
            int32 counter;
            counter++;
            return counter;
        }
    "#;
    assert_compile_error(compound_inc, "E0272");
}

#[test]
fn test_definite_assignment_across_branches_succeeds() {
    let source_code = r#"
        int32 resolve(bool flag) {
            int32 value;
            if (flag) {
                value = 10;
            } else {
                value = 20;
            }
            return value;
        }

        int32 resolve_early_return(bool flag) {
            int32 value;
            if (flag) {
                value = 30;
            } else {
                return 40;
            }
            return value;
        }

        int32 main() {
            if (resolve(true) != 10 || resolve(false) != 20) return 1;
            if (resolve_early_return(true) != 30 || resolve_early_return(false) != 40) return 2;
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}
