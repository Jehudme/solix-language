//! Suite: OperatorDeclaration - Declarations (TESTS.md)

use solix_runtime::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_3_1_vector_addition_overload() {
    let code = r#"
        class Vector {
            public int32 x;
            public Vector(int32 x) { this.x = x; }
            public Vector operator+(Vector other) {
                return new Vector(this.x + other.x);
            }
        }

        class Main {
            public static int32 main() {
                Vector v1 = new Vector(10);
                Vector v2 = new Vector(20);
                Vector v3 = v1 + v2;
                return v3.x == 30 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_2_overloading_subtraction_and_equality() {
    let code = r#"
        class Complex {
            public int32 re;
            public int32 im;
            public Complex(int32 r, int32 i) { this.re = r; this.im = i; }

            public Complex operator-(Complex rhs) {
                return new Complex(this.re - rhs.re, this.im - rhs.im);
            }
            public bool operator==(Complex rhs) {
                return this.re == rhs.re && this.im == rhs.im;
            }
        }

        class Main {
            public static int32 main() {
                Complex c1 = new Complex(10, 5);
                Complex c2 = new Complex(4, 2);
                Complex res = c1 - c2;
                return (res == new Complex(6, 3)) ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_unsupported_operator_overload() {
    // Error: E0100 / E0220
    let code = r#"
        class Test {
            bool operator&&(Test other) { return true; }
        }
    "#;
    assert_compile_error(code, "cannot be overloaded");
}

#[test]
fn test_case_4_2_binary_operator_declared_wrong_arity() {
    // Error: E0220 (InvalidOperatorOverload)
    let code = r#"
        class Vector {
            public Vector operator+(Vector a, Vector b) { return a; }
        }
    "#;
    assert_compile_error(code, "E0220");
}

#[test]
fn test_case_4_3_operator_overload_declared_at_top_level() {
    // Error: E0118 (OperatorOutsideClass)
    let code = r#"
        public int32 operator+(int32 a, int32 b) {
            return a + b;
        }
    "#;
    assert_compile_error(code, "E0118");
}

#[test]
fn test_case_4_5_operator_overload_declared_inside_method_body() {
    // Error: E0118 (OperatorOutsideClass)
    let code = r#"
        class Main {
            public static void test() {
                operator+(int32 a) {}
            }
        }
    "#;
    assert_compile_error(code, "E0118");
}

#[test]
fn test_case_4_6_package_level_native_compatibility_and_free_functions() {
    let code = r#"
        class Main {
            public static int32 main() {
                non_existent_free_function();
                return 0;
            }
        }
    "#;
    assert_compile_error(code, "E0201");
}

#[test]
fn test_case_4_7_raylib_variadic_api_adapters() {
    let code = r#"
        native void trace_log(int32 level, string text);

        class Main {
            public static int32 main() {
                trace_log(1, "hello", 123);
                return 0;
            }
        }
    "#;
    assert_compile_error(code, "E0238");
}

#[test]
fn test_case_4_8_raylib_value_type_structural_equality() {
    let code = r#"
        class Vector2 {
            public float64 x;
            public float64 y;
            public Vector2(float64 x, float64 y) {
                this.x = x;
                this.y = y;
            }
            public bool operator==(Vector2 other) {
                return this.x == other.x && this.y == other.y;
            }
        }

        class Main {
            public static int32 main() {
                Vector2 first = new Vector2(10.0, 20.0);
                Vector2 second = new Vector2(10.0, 20.0);
                Vector2 third = new Vector2(10.0, 25.0);
                if (!(first == second)) return 1;
                if (first == third) return 2;
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}
