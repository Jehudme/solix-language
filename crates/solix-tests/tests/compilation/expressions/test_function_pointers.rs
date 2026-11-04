//! Function Pointers Conformance Tests (SPEC-0001 §14, TESTS.md Section 1).

#![forbid(unsafe_code)]

use solix_runtime::exceptions::RuntimeErrorCode;
use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, assert_runtime_error, compile_and_run};

#[test]
fn test_case_1_1_direct_assignment_and_invocation() {
    let code = r#"
        class MathUtils {
            public static int32 add(int32 a, int32 b) {
                return a + b;
            }
        }

        int32 main() {
            int32(*)(int32, int32) op = MathUtils::add;
            return op(10, 20) == 30 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_1_2_null_initialization_and_reassignment() {
    let code = r#"
        class MathUtils {
            public static int32 mul(int32 a, int32 b) {
                return a * b;
            }
        }

        int32 main() {
            int32(*)(int32, int32) op = null;
            if (op != null) return 1;
            op = MathUtils::mul;
            if (op == null) return 2;
            return op(6, 7) == 42 ? 0 : 3;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_1_3_higher_order_function_passing() {
    let code = r#"
        class Operations {
            public static int32 sub(int32 a, int32 b) {
                return a - b;
            }

            public static int32 apply(int32 a, int32 b, int32(*)(int32, int32) fn) {
                return fn(a, b);
            }
        }

        int32 main() {
            int32 res = Operations::apply(50, 18, Operations::sub);
            return res == 32 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_1_4_type_aliasing_with_function_pointers() {
    let code = r#"
        alias BinaryOp = int32(*)(int32, int32);

        class Calc {
            public static int32 add(int32 a, int32 b) {
                return a + b;
            }
        }

        int32 main() {
            BinaryOp op = Calc::add;
            return op(100, 200) == 300 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_1_5_returning_function_pointer_from_method() {
    let code = r#"
        class Factory {
            public static int32 double_val(int32 x) {
                return x * 2;
            }

            public static int32(*)(int32) get_fn() {
                return Factory::double_val;
            }
        }

        int32 main() {
            int32(*)(int32) f = Factory::get_fn();
            return f(21) == 42 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_2_1_invoking_null_function_pointer_throws_null_pointer_exception() {
    let code = r#"
        int32 main() {
            int32(*)(int32, int32) op = null;
            return op(1, 2);
        }
    "#;
    assert_runtime_error(code, RuntimeErrorCode::NullReferenceAccess);
}

#[test]
fn test_case_2_2_parameter_count_mismatch() {
    let code = r#"
        class MathUtils {
            public static int32 add(int32 a, int32 b) {
                return a + b;
            }
        }

        void test() {
            int32(*)(int32) op = MathUtils::add;
        }
    "#;
    assert_compile_error(code, "mismatch");
}

#[test]
fn test_case_2_3_return_type_mismatch() {
    let code = r#"
        class MathUtils {
            public static int32 add(int32 a, int32 b) {
                return a + b;
            }
        }

        void test() {
            void(*)(int32, int32) op = MathUtils::add;
        }
    "#;
    assert_compile_error(code, "mismatch");
}

#[test]
fn test_case_2_4_address_of_non_static_method_error() {
    let code = r#"
        class Greeter {
            public void greet() {}
        }

        void test() {
            void(*)() f = Greeter::greet;
        }
    "#;
    assert_compile_error(code, "greet");
}

#[test]
fn test_case_2_5_sizeof_function_pointer_type() {
    let code = r#"
        int32 main() {
            int32 sz = sizeof(int32(*)(int32, int32));
            return sz == 8 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}
