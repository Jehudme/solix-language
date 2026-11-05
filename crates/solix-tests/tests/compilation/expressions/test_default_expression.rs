//! DefaultExpression Conformance Tests (SPEC-0001 §14, TESTS.md Part IV).

#![forbid(unsafe_code)]

use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_1_1_primitive_default_value_evaluation() {
    let code = r#"
        class Main {
            public static int32 main() {
                int32 i = default(int32);
                bool b = default(bool);
                float64 f = default(float64);
                char c = default(char);
                if (i == 0 && b == false && f == 0.0 && c == '\0') {
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
fn test_case_1_2_reference_type_default_values() {
    let code = r#"
        class Item {
            public int32 id;
        }

        class Main {
            public static int32 main() {
                Item item = default(Item);
                int32[] arr = default(int32[]);
                if (item == null && arr == null) {
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
fn test_case_1_3_function_pointer_default_value() {
    let code = r#"
        class Main {
            public static int32 main() {
                int32(*)(int32) fn = default(int32(*)(int32));
                if (fn == null) {
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
fn test_case_1_4_generic_container_slot_clearance_with_default() {
    let code = r#"
        class Box<T> {
            public T val;
            public Box(T v) { this.val = v; }
            public void clear() {
                this.val = default(T);
            }
        }

        class User {
            public int32 id;
        }

        class Main {
            public static int32 main() {
                Box<int32> intBox = new Box<int32>(42);
                intBox.clear();
                Box<User> userBox = new Box<User>(new User());
                userBox.clear();
                if (intBox.val == 0 && userBox.val == null) {
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
fn test_case_1_5_generic_function_returning_default() {
    let code = r#"
        public class Helpers {
            public static T getDefault<T>() {
                return default(T);
            }
        }

        public class Main {
            public static int32 main() {
                int32 defInt = Helpers.getDefault<int32>();
                bool defBool = Helpers.getDefault<bool>();
                if (defInt == 0 && defBool == false) {
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
fn test_case_2_1_default_of_void_type() {
    // E0203: Void cannot be used in default expression
    let code = r#"
        class Main {
            public static void test() {
                int32 x = default(void);
            }
        }
    "#;
    assert_compile_error(code, "void");
}

#[test]
fn test_case_2_2_default_of_undeclared_type() {
    // E0201: Unknown type in default expression
    let code = r#"
        class Main {
            public static void test() {
                int32 x = default(UnknownType);
            }
        }
    "#;
    assert_compile_error(code, "UnknownType");
}
