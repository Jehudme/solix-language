//! Callee Frame Allocation Conformance Tests (TESTS.md Section 1).

#![forbid(unsafe_code)]

use solix_runtime::value::model::Value;
use solix_tests::common::compile_and_run;

#[test]
fn test_case_1_1_polymorphic_virtual_dispatch_with_variable_frame_sizes() {
    let code = r#"
        class Base {
            public virtual int32 compute() {
                return 1;
            }
        }

        class Derived extends Base {
            public override int32 compute() {
                int32 a = 10;
                int32 b = 20;
                int32 c = 30;
                return a + b + c;
            }
        }

        int32 main() {
            Base b = new Derived();
            return b.compute() == 60 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_1_2_deep_call_stack_and_recursion_integrity() {
    let code = r#"
        int32 fib(int32 n) {
            if (n <= 1) return n;
            return fib(n - 1) + fib(n - 2);
        }

        int32 main() {
            int32 res = fib(10);
            return res == 55 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_2_1_method_with_multiple_local_slots() {
    let code = r#"
        int32 sum_locals(int32 x) {
            int32 l1 = 1;
            int32 l2 = 2;
            int32 l3 = 3;
            int32 l4 = 4;
            int32 l5 = 5;
            return x + l1 + l2 + l3 + l4 + l5;
        }

        int32 main() {
            return sum_locals(10) == 25 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}
