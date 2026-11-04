//! Suite: EnumDeclaration - Declarations (TESTS.md)

use solix_runtime::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_3_1_enum_equality_and_switch() {
    let code = r#"
        enum Color {
            RED,
            GREEN,
            BLUE
        }

        class Main {
            public static int32 main() {
                Color c = Color::RED;
                if (c == Color::RED) {
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
fn test_case_3_2_enum_as_method_parameter_and_return_value() {
    let code = r#"
        enum State { PENDING, ACTIVE, CLOSED }

        class Task {
            public State state;
            public void setState(State s) { this.state = s; }
            public State getState() { return this.state; }
        }

        class Main {
            public static int32 main() {
                Task t = new Task();
                t.setState(State::ACTIVE);
                return t.getState() == State::ACTIVE ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_3_nested_enum_declaration_inside_class() {
    let code = r#"
        class Window {
            public enum State { MINIMIZED, MAXIMIZED, NORMAL }
            public State state;
        }

        class Main {
            public static int32 main() {
                Window w = new Window();
                w.state = Window::State::MAXIMIZED;
                return w.state == Window::State::MAXIMIZED ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_duplicate_enum_member() {
    // Error: E0202 (DuplicateSymbol)
    let code = r#"
        enum State {
            READY,
            READY
        }
    "#;
    assert_compile_error(code, "E0202");
}

#[test]
fn test_case_4_2_implicit_integer_assignment_to_enum() {
    // Error: E0203 (TypeMismatch)
    let code = r#"
        enum Status { OK, FAIL }

        class Main {
            public static void test() {
                Status s = 1;
            }
        }
    "#;
    assert_compile_error(code, "E0203");
}

#[test]
fn test_case_4_3_comparing_incompatible_enum_types() {
    // Error: E0203 (TypeMismatch)
    let code = r#"
        enum Fruit { APPLE, ORANGE }
        enum Animal { CAT, DOG }

        class Main {
            public static void test() {
                bool b = (Fruit::APPLE == Animal::CAT);
            }
        }
    "#;
    assert_compile_error(code, "E0203");
}

#[test]
fn test_case_4_4_enum_declared_inside_function_body() {
    // Error: E0115 (EnumInLocalScope)
    let code = r#"
        void test() {
            enum LocalState { ON, OFF }
        }
    "#;
    assert_compile_error(code, "E0115");
}
