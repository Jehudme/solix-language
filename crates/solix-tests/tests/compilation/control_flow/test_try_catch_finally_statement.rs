//! TryCatchFinallyStatement Conformance Tests (SPEC-0001 §12, TESTS.md Section 2.11).

#![forbid(unsafe_code)]

use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_3_1_specific_catch_hierarchy() {
    let code = r#"
        class Exception {}
        class CustomError extends Exception {}

        int32 main() {
            int32 result = 0;
            try {
                throw new CustomError();
            } catch (CustomError c) {
                result = 1;
            } catch (Exception e) {
                result = 2;
            }
            return result;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(1));
}

#[test]
fn test_case_3_2_try_block_with_only_finally_clause() {
    let code = r#"
        class Main {
            public static int32 cleanup_marker = 0;

            public static void work() {
                try {
                    Main::cleanup_marker += 10;
                } finally {
                    Main::cleanup_marker += 20;
                }
            }

            public static int32 main() {
                Main::work();
                return Main::cleanup_marker == 30 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_3_exception_in_catch_with_guaranteed_finally_execution() {
    let code = r#"
        class Exception {}

        class Main {
            public static bool finally_ran = false;

            public static void faulty() {
                try {
                    throw new Exception();
                } catch (Exception e) {
                    throw new Exception();
                } finally {
                    Main::finally_ran = true;
                }
            }

            public static int32 main() {
                try {
                    Main::faulty();
                } catch (Exception e) {}
                return Main::finally_ran ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_unreachable_catch_clause() {
    // Error: E0200 / E0202
    let code = r#"
        class Exception {}
        class SubErr extends Exception {}

        void test() {
            try {
                int32 x = 0;
            } catch (Exception e) {
            } catch (SubErr s) {
            }
        }
    "#;
    assert_compile_error(code, "catch");
}

#[test]
fn test_case_4_2_catching_non_exception_type() {
    // Error: E0202 (TypeMismatch)
    let code = r#"
        void test() {
            try {} catch (int32 x) {}
        }
    "#;
    assert_compile_error(code, "catch");
}

#[test]
fn test_case_4_3_duplicate_catch_clause_for_same_type() {
    // Error: E0201 (DuplicateSymbol)
    let code = r#"
        class Exception {}

        void test() {
            try {}
            catch (Exception e) {}
            catch (Exception e2) {}
        }
    "#;
    assert_compile_error(code, "catch");
}

#[test]
fn test_case_4_5_try_catch_statement_placed_directly_in_class_body() {
    // Error: E0100 (ParsingError)
    let code = r#"
        class BadClass {
            try {} catch (Exception e) {}
        }
    "#;
    assert_compile_error(code, "class");
}

#[test]
fn test_finally_with_return_override_and_no_stack_overflow() {
    let source_code = r#"
        int32 compute_value() {
            try {
                return 10;
            } finally {
                return 20;
            }
        }

        int32 main() {
            return compute_value();
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(20));
}

#[test]
fn test_return_expression_evaluated_before_finally() {
    let source_code = r#"
        class Main {
            public static int32 side_effect_counter = 0;

            public static int32 compute() {
                try {
                    return Main::side_effect_counter;
                } finally {
                    Main::side_effect_counter = Main::side_effect_counter + 100;
                }
            }

            public static int32 main() {
                int32 returned_value = Main::compute();
                if (returned_value == 0 && Main::side_effect_counter == 100) {
                    return 42;
                }
                return 0;
            }
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(42));
}

#[test]
fn test_loop_break_and_continue_only_unwinds_inner_finally() {
    let source_code = r#"
        class Main {
            public static int32 outer_finally_counter = 0;
            public static int32 inner_finally_counter = 0;

            public static int32 main() {
                try {
                    int32 loop_counter = 0;
                    while (loop_counter < 3) {
                        try {
                            loop_counter = loop_counter + 1;
                            if (loop_counter == 1) {
                                continue;
                            }
                            if (loop_counter == 2) {
                                break;
                            }
                        } finally {
                            Main::inner_finally_counter = Main::inner_finally_counter + 1;
                        }
                    }
                } finally {
                    Main::outer_finally_counter = Main::outer_finally_counter + 1;
                }

                if (Main::inner_finally_counter == 2 && Main::outer_finally_counter == 1) {
                    return 55;
                }
                return 0;
            }
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(55));
}

#[test]
fn test_catch_variable_reassignment_rejected() {
    let source_code = r#"
        class Exception {}

        void test_method() {
            try {
                throw new Exception();
            } catch (Exception caught_exception) {
                caught_exception = new Exception();
            }
        }
    "#;
    assert_compile_error(source_code, "E0210");
}

#[test]
fn test_try_return_local_preserved_when_finally_mutates_or_nullifies() {
    // Regression test (#300.3): returning a local object or primitive from `try` must snapshot
    // (and retain reference-typed values) before `finally` runs so mutating/nulling in `finally`
    // does not cause UAF, premature destruction, or corrupt primitive returns.
    let source_code = r#"
        class Token {
            public static int32 destroyed = 0;
            public int32 payload;

            public Token(int32 payload) {
                this.payload = payload;
            }

            ~Token() {
                Token::destroyed = Token::destroyed + 1;
            }
        }

        class Main {
            public static Token build_and_clear() {
                Token owned = new Token(88);
                try {
                    return owned;
                } finally {
                    owned = null;
                }
            }

            public static int32 primitive_snapshot() {
                int32 local_val = 77;
                try {
                    return local_val;
                } finally {
                    local_val = 999;
                }
            }

            public static int32 main() {
                Token::destroyed = 0;
                Token returned_tok = Main::build_and_clear();
                int32 destroyed_while_live = Token::destroyed;
                int32 read_payload = returned_tok.payload;
                returned_tok = null;
                int32 destroyed_after_drop = Token::destroyed;
                int32 prim_val = Main::primitive_snapshot();

                if (destroyed_while_live != 0 || destroyed_after_drop != 1) {
                    return 1;
                }
                if (read_payload != 88 || prim_val != 77) {
                    return 2;
                }
                return 0;
            }
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_nested_finally_early_return_still_runs_outer_finally() {
    // Regression test (#300.3): early return inside an inner `finally` block must still execute
    // enclosing outer `finally` blocks instead of skipping them due to stack truncation.
    let source_code = r#"
        class Main {
            public static int32 outer_runs = 0;
            public static int32 inner_runs = 0;

            public static int32 compute() {
                try {
                    try {
                        return 10;
                    } finally {
                        Main::inner_runs = Main::inner_runs + 1;
                        return 25;
                    }
                } finally {
                    Main::outer_runs = Main::outer_runs + 1;
                }
            }

            public static int32 main() {
                Main::outer_runs = 0;
                Main::inner_runs = 0;
                int32 val = Main::compute();
                if (val == 25 && Main::inner_runs == 1 && Main::outer_runs == 1) {
                    return 0;
                }
                return (val * 100) + (Main::inner_runs * 10) + Main::outer_runs;
            }
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}
