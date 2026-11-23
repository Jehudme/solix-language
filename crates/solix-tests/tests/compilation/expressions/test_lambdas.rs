//! Lambdas and Closures Conformance Tests (SPEC-0001 §14, TESTS.md Section 1).

#![forbid(unsafe_code)]

use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_1_1_stateless_lambda_expression() {
    let code = r#"
        class Main {
            public static int32 main() {
                int32(*)(int32, int32) add = [](int32 a, int32 b) => a + b;
                return add(10, 20) == 30 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_1_2_lambda_capturing_primitive_local_variable() {
    let code = r#"
        class Main {
            public static int32 main() {
                int32 factor = 5;
                int32(*)(int32) mult = [factor](int32 x) => x * factor;
                return mult(6) == 30 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_1_3_multi_parameter_lambda_with_block_body_and_explicit_return_type() {
    let code = r#"
        class Main {
            public static int32 main() {
                int32 base = 100;
                int32(*)(int32, int32) compute = [base](int32 a, int32 b) : int32 {
                    int32 sum = a + b;
                    return base + sum;
                };
                return compute(20, 3) == 123 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_1_4_lambda_capturing_object_reference_with_arc_tracking() {
    let code = r#"
        class Counter {
            public int32 val;
            public Counter(int32 v) { this.val = v; }
        }

        class Main {
            public static int32 main() {
                Counter c = new Counter(42);
                int32(*)() getVal = [c]() => c.val;
                return getVal() == 42 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_1_5_returning_closure_from_function_escaping_stack_frame() {
    let code = r#"
        class Main {
            public static int32(*)(int32) makeAdder(int32 x) {
                return [x](int32 y) => x + y;
            }

            public static int32 main() {
                int32(*)(int32) addTen = Main::makeAdder(10);
                return addTen(25) == 35 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_1_6_capturing_this_in_instance_method() {
    let code = r#"
        class Multiplier {
            public int32 factor;
            public Multiplier(int32 f) { this.factor = f; }

            public int32(*)(int32) getMultiplier() {
                return [this](int32 x) => x * this.factor;
            }
        }

        class Main {
            public static int32 main() {
                Multiplier m = new Multiplier(7);
                int32(*)(int32) fn = m.getMultiplier();
                return fn(6) == 42 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_1_7_reassigning_closure_variable_in_a_loop_arc_recycling() {
    let code = r#"
        class Main {
            public static int32 main() {
                int32(*)(int32) fn = [](int32 x) => x;
                for (int32 i = 0; i < 50; ++i) {
                    int32 cur = i;
                    fn = [cur](int32 x) => x + cur;
                }
                return fn(10) == 59 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_1_8_nested_closures_with_deep_capture_hierarchy() {
    let code = r#"
        class Main {
            public static int32 main() {
                int32 a = 10;
                int32(*)(int32) outer = [a](int32 b) : int32 {
                    int32(*)(int32) inner = [a, b](int32 c) => a + b + c;
                    return inner(30);
                };
                return outer(20) == 60 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_2_1_undefined_variable_in_capture_list() {
    let code = r#"
        class Main {
            public static void test() {
                int32(*)() fn = [nonExistentVar]() => 0;
            }
        }
    "#;
    assert_compile_error(code, "nonExistentVar");
}

#[test]
fn test_case_2_2_capturing_this_outside_instance_method() {
    let code = r#"
        class Main {
            public static void test() {
                int32(*)() fn = [this]() => 0;
            }
        }
    "#;
    assert_compile_error(code, "this");
}

#[test]
fn test_case_2_3_lambda_signature_incompatible_with_target_function_pointer() {
    let code = r#"
        class Main {
            public static void test() {
                int32(*)(int32, int32) fn = [](int32 a) => a;
            }
        }
    "#;
    assert_compile_error(code, "mismatch");
}

#[test]
fn test_lambda_block_body_return_inference() {
    let source_code = r#"
        int32 invoke_function(int32(*)(int32) callback, int32 argument_value) {
            return callback(argument_value);
        }

        int32 main() {
            int32(*)(int32) doubler = (int32 input_value) => {
                return input_value * 2;
            };
            return invoke_function(doubler, 21);
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(42));
}

#[test]
fn test_lambda_returned_from_function() {
    let source_code = r#"
        class Factory {
            public static int32(*)(int32) make_adder(int32 increment_amount) {
                return [increment_amount](int32 base_value) => {
                    return base_value + increment_amount;
                };
            }

            public static int32 main() {
                int32(*)(int32) add_ten = Factory::make_adder(10);
                return add_ten(32);
            }
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(42));
}
