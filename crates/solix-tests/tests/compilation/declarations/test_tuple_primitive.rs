//! Suite: TuplePrimitive - Declarations (TESTS.md)

use solix_runtime::{RuntimeErrorCode, Value};
use solix_tests::common::{assert_compile_error, assert_runtime_error, compile_and_run};

#[test]
fn test_case_55_1_direct_tuple_creation_and_constant_indexing() {
    let code = r#"
        class Main {
            public static int32 main() {
                tuple t = tuple(42, 'H', true);
                if (t.length != 3) return 1;
                int32 v0 = cast(t[0], int32);
                char v1 = cast(t[1], char);
                bool v2 = cast(t[2], bool);
                if (v0 != 42 || v1 != 'H' || !v2) return 2;
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_55_2_dynamic_loop_iteration_over_tuple() {
    let code = r#"
        class Main {
            public static int32 main() {
                tuple t = tuple(10, 20, 30);
                int32 sum = 0;
                for (int32 i = 0; i < t.length; i++) {
                    sum = sum + cast(t[i], int32);
                }
                return sum == 60 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_55_3_runtime_instanceof_with_primitives() {
    let code = r#"
        class Main {
            public static int32 main() {
                tuple t = tuple(42, true, 'X');
                if (!(t[0] instanceof int32)) return 1;
                if (t[0] instanceof bool) return 2;
                if (!(t[1] instanceof bool)) return 3;
                if (t[1] instanceof int32) return 4;
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_55_4_runtime_instanceof_with_classes_and_interfaces() {
    let code = r#"
        interface Worker {
            int32 work();
        }

        class Robot implements Worker {
            public int32 work() { return 100; }
        }

        class Person {}

        class Main {
            public static int32 main() {
                tuple t = tuple(new Robot(), new Person());
                if (!(t[0] instanceof Worker)) return 1;
                if (!(t[0] instanceof Robot)) return 2;
                if (t[0] instanceof Person) return 3;
                if (t[1] instanceof Worker) return 4;
                if (!(t[1] instanceof Person)) return 5;
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_55_5_function_template_lowering_with_args() {
    let code = r#"
        int32 sum_all<args...>(int32 first, args...) {
            int32 total = first;
            for (int32 i = 0; i < args.length; i++) {
                total = total + cast(args[i], int32);
            }
            return total;
        }

        int32 main() {
            int32 res = sum_all(10, 20, 30, 40);
            return res == 100 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_55_6_automatic_call_site_argument_bundling() {
    let code = r#"
        int32 count_args<args...>(int32 label, args...) {
            return args.length;
        }

        int32 main() {
            if (count_args(0) != 0) return 1;
            if (count_args(1, 100) != 1) return 2;
            if (count_args(3, 1, 'b', true) != 3) return 3;
            return 0;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_55_7_destructor_trigger_on_tuple_scope_exit() {
    let code = r#"
        class Tracker {
            public static int32 destroyed_count = 0;
            ~Tracker() {
                Tracker::destroyed_count = Tracker::destroyed_count + 1;
            }
        }

        class Helper {
            public static void test_scope() {
                tuple t = tuple(new Tracker(), new Tracker());
            }
        }

        class Main {
            public static int32 main() {
                Tracker::destroyed_count = 0;
                Helper::test_scope();
                return Tracker::destroyed_count == 2 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_55_8_destructor_trigger_on_tuple_reassignment() {
    let code = r#"
        class Tracker {
            public static int32 destroyed_count = 0;
            ~Tracker() {
                Tracker::destroyed_count = Tracker::destroyed_count + 1;
            }
        }

        class Main {
            public static int32 main() {
                Tracker::destroyed_count = 0;
                tuple t = tuple(new Tracker());
                if (Tracker::destroyed_count != 0) return 1;
                t = null;
                if (Tracker::destroyed_count != 1) return 2;
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_55_9_exception_stack_unwinding_raii_with_tuple() {
    let code = r#"
        class Exception {}

        class Resource {
            public static int32 released = 0;
            ~Resource() {
                Resource::released = Resource::released + 1;
            }
        }

        class Helper {
            public static void risky() {
                tuple t = tuple(new Resource());
                throw new Exception();
            }
        }

        class Main {
            public static int32 main() {
                Resource::released = 0;
                try {
                    Helper::risky();
                } catch (Exception e) {
                    if (Resource::released != 1) return 2;
                    return 0;
                }
                return 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_55_10_nested_tuples_recursive_destruction() {
    let code = r#"
        class Leaf {
            public static int32 count = 0;
            ~Leaf() {
                Leaf::count = Leaf::count + 1;
            }
        }

        class Helper {
            public static void test_nested() {
                tuple inner = tuple(new Leaf());
                tuple outer = tuple(inner);
            }
        }

        class Main {
            public static int32 main() {
                Leaf::count = 0;
                Helper::test_nested();
                return Leaf::count == 1 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_55_13_variadic_parameter_not_trailing() {
    // Error: E0136 (VariadicParameterNotLast)
    let code = r#"
        class Main {
            public static void invalid<args...>(args..., int32 extra) {}
            public static int32 main() { return 0; }
        }
    "#;
    assert_compile_error(code, "E0136");
}

#[test]
fn test_case_55_14_multiple_variadic_packs_in_same_signature() {
    // Error: E0136 (VariadicParameterNotLast)
    let code = r#"
        class Main {
            public static void invalid<args..., others...>(args..., others...) {}
            public static int32 main() { return 0; }
        }
    "#;
    assert_compile_error(code, "E0136");
}

#[test]
fn test_case_55_11_invalid_type_cast_throws_type_cast_exception() {
    let source_code = r#"
        class Main {
            public static int32 main() {
                tuple invalid_tuple = tuple("not an int");
                int32 extracted_val = cast(invalid_tuple[0], int32);
                return 0;
            }
        }
    "#;
    assert_runtime_error(source_code, RuntimeErrorCode::InvalidCast);
}

#[test]
fn test_case_55_12_out_of_bounds_index_access_throws_index_out_of_bounds_exception() {
    let source_code = r#"
        class Main {
            public static int32 main() {
                tuple bound_tuple = tuple(1, 2);
                int32 extracted_val = cast(bound_tuple[5], int32);
                return 0;
            }
        }
    "#;
    assert_runtime_error(source_code, RuntimeErrorCode::IndexOutOfBounds);
}

#[test]
fn test_case_55_15_incompatible_class_cast_on_tuple_element_throws_type_cast_exception() {
    let source_code = r#"
        class Animal {}
        class Dog extends Animal {}
        class Cat extends Animal {}

        class Main {
            public static int32 main() {
                tuple animal_tuple = tuple(new Dog());
                Cat invalid_cat = cast(animal_tuple[0], Cat);
                return 0;
            }
        }
    "#;
    assert_runtime_error(source_code, RuntimeErrorCode::InvalidCast);
}
