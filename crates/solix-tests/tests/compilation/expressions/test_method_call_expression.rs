//! MethodCallExpression Conformance Tests (SPEC-0001 §14, TESTS.md Section 4.11).

#![forbid(unsafe_code)]

use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_3_1_overloaded_method_selection() {
    let code = r#"
        class Printer {
            public int32 print(int32 x) { return 1; }
            public int32 print(float64 s) { return 2; }
        }

        int32 main() {
            Printer p = new Printer();
            int32 a = p.print(42);
            int32 b = p.print(3.14);
            if (a != 1 || b != 2) return 1;
            return 0;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_2_calling_inherited_superclass_method() {
    let code = r#"
        class BaseCalc {
            public int32 add(int32 a, int32 b) { return a + b; }
        }
        class AdvancedCalc extends BaseCalc {
            public int32 doubleAdd(int32 a, int32 b) {
                return this.add(a, b) * 2;
            }
        }

        int32 main() {
            AdvancedCalc calc = new AdvancedCalc();
            return calc.doubleAdd(3, 4) == 14 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_no_matching_overload() {
    let code = r#"
        class Printer {
            public void print(int32 x) {}
        }

        void test(Printer p) {
            p.print(true);
        }
    "#;
    assert_compile_error(code, "print");
}

#[test]
fn test_case_4_2_method_call_with_incorrect_argument_count() {
    let code = r#"
        class Calculator {
            public int32 compute(int32 a, int32 b) { return a + b; }
        }

        void test() {
            Calculator c = new Calculator();
            c.compute(10);
        }
    "#;
    assert_compile_error(code, "compute");
}
