//! ExpressionStatement Conformance Tests (SPEC-0001 §12, TESTS.md Section 2.5).

#![forbid(unsafe_code)]

use solix_runtime::{RuntimeContext, RuntimeError, RuntimeOptions, Value, VirtualMachine};
use solix_tests::common::{assert_compile_error, compile_and_run, compile_source};

#[test]
fn test_case_3_1_method_calls_and_assignments() {
    let code = r#"
        int32 main() {
            int32 x = 0;
            x = 10;
            x++;
            return x;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(11));
}

#[test]
fn test_case_3_2_immediate_temporary_reclamation() {
    let code = r#"
        class Res {
            public int32 val;
            public Res(int32 v) { this.val = v; }
        }
        Res generate() { return new Res(42); }
        int32 main() {
            generate();
            return 0;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_3_chained_fluent_method_calls() {
    let code = r#"
        class Builder {
            public int32 val;
            public Builder add(int32 x) { this.val += x; return this; }
        }

        int32 main() {
            Builder b = new Builder();
            b.add(10).add(20).add(30);
            return b.val == 60 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_missing_semicolon() {
    // Error: E0100 (ParsingError)
    let code = r#"
        void test() {
            int32 x = 0;
            x = 10
        }
    "#;
    assert_compile_error(code, ";");
}

#[test]
fn test_case_4_2_method_call_on_null_reference_runtime_fault() {
    // Runtime Error: NullReferenceAccess
    let code = r#"
        class Res {
            public int32 get_val() { return 10; }
        }
        int32 main() {
            Res r = null;
            return r.get_val();
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
fn test_case_4_3_incomplete_expression_statement() {
    // Error: E0100 (ParsingError)
    let code = r#"
        void test() {
            int32 x = ;
        }
    "#;
    assert_compile_error(code, "expression");
}

#[test]
fn test_case_4_4_expression_statement_directly_in_class_body() {
    // Error: E0100 (ParsingError)
    let code = r#"
        class BadClass {
            10 + 20;
        }
    "#;
    assert_compile_error(code, "class");
}
