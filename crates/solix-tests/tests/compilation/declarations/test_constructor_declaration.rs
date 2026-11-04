//! Suite: ConstructorDeclaration - Declarations (TESTS.md)

use solix_runtime::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_3_1_chained_super_constructor() {
    let code = r#"
        class Base {
            public int32 id;
            public Base(int32 id) {
                this.id = id;
            }
        }

        class Sub extends Base {
            public Sub(int32 id) : super(id) {}
        }

        class Main {
            public static int32 main() {
                Sub s = new Sub(42);
                return s.id == 42 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_2_overloaded_constructors_varying_arity() {
    let code = r#"
        class Point {
            public int32 x;
            public int32 y;
            public Point() { this.x = 0; this.y = 0; }
            public Point(int32 x, int32 y) { this.x = x; this.y = y; }
        }

        class Main {
            public static int32 main() {
                Point p1 = new Point();
                Point p2 = new Point(10, 20);
                return (p1.x == 0 && p2.x == 10 && p2.y == 20) ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_3_protected_constructor() {
    let code = r#"
        class BaseAuth {
            protected BaseAuth() {}
        }

        class UserAuth extends BaseAuth {
            public UserAuth() : super() {}
        }

        class Main {
            public static int32 main() {
                UserAuth u = new UserAuth();
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_constructor_name_mismatch() {
    // Error: E0133 (ConstructorNameMismatch)
    let code = r#"
        class Widget {
            Gadget() {}
        }
    "#;
    assert_compile_error(code, "E0133");
}

#[test]
fn test_case_4_2_constructor_with_explicit_return_type() {
    // Error: E0105 or syntax error
    let code = r#"
        class Widget {
            void Widget() {}
        }
    "#;
    assert_compile_error(code, "return type");
}

#[test]
fn test_case_4_3_invoking_super_out_of_order() {
    // Error: E0218 (SuperConstructorMustBeFirst)
    let code = r#"
        class Base {}
        class Sub extends Base {
            public int32 val;
            public Sub() {
                this.val = 42;
                super();
            }
        }
    "#;
    assert_compile_error(code, "E0218");
}

#[test]
fn test_case_4_4_constructor_declared_outside_class() {
    // Error: E0119 (ConstructorOutsideClass)
    let code = r#"
        StandaloneConstructor() {
        }
    "#;
    assert_compile_error(code, "E0119");
}

#[test]
fn test_case_4_5_constructor_declared_inside_method_body() {
    // Error: E0119 or E0123
    let code = r#"
        class Main {
            public static void test() {
                MyClass() {}
            }
        }
    "#;
    assert_compile_error(code, "E01");
}
