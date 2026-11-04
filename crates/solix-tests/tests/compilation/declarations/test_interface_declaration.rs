//! Suite: InterfaceDeclaration - Declarations (TESTS.md)

use solix_runtime::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_3_1_multiple_interface_conformance() {
    let code = r#"
        interface Printable { void print(); }
        interface Serializable { void save(); }

        class Doc implements Printable, Serializable {
            public void print() {}
            public void save() {}
        }

        class Main {
            public static int32 main() {
                Doc d = new Doc();
                d.print();
                d.save();
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_2_interface_hierarchy_with_sub_interface_extension() {
    let code = r#"
        interface Reader {
            int32 read();
        }
        interface AdvancedReader extends Reader {
            void reset();
        }
        class BasicReader implements AdvancedReader {
            public int32 read() { return 10; }
            public void reset() {}
        }
        class Main {
            public static int32 main() {
                AdvancedReader reader_inst = new BasicReader();
                return reader_inst.read() == 10 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_3_polymorphic_method_invocation_via_interface() {
    let code = r#"
        interface Worker {
            int32 work();
        }
        class Robot implements Worker {
            public int32 work() { return 99; }
        }

        class Main {
            public static int32 main() {
                Worker w = new Robot();
                return w.work() == 99 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_4_nested_interface_declaration_inside_class() {
    let code = r#"
        class Button {
            public interface OnClickListener {
                int32 onClick();
            }
        }
        class MyListener implements Button::OnClickListener {
            public int32 onClick() { return 42; }
        }
        class Main {
            public static int32 main() {
                Button::OnClickListener listener = new MyListener();
                return listener.onClick() == 42 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_5_multi_interface_dynamic_dispatch_execution() {
    let code = r#"
        interface Worker { int32 work(); }
        interface Greeter { int32 greet(); }

        class Android implements Worker, Greeter {
            public int32 work() { return 10; }
            public int32 greet() { return 20; }
        }

        class Main {
            public static int32 main() {
                Android a = new Android();
                Worker w = a;
                Greeter g = a;
                if (w.work() != 10) return 1;
                if (g.greet() != 20) return 2;
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_6_order_independent_multi_interface_resolution() {
    let code = r#"
        interface FirstIface { int32 getA(); }
        interface SecondIface { int32 getB(); }

        class ClassAB implements FirstIface, SecondIface {
            public int32 getA() { return 1; }
            public int32 getB() { return 2; }
        }

        class ClassBA implements SecondIface, FirstIface {
            public int32 getA() { return 10; }
            public int32 getB() { return 20; }
        }

        class Main {
            public static int32 main() {
                FirstIface a1 = new ClassAB();
                SecondIface b1 = new ClassAB();
                FirstIface a2 = new ClassBA();
                SecondIface b2 = new ClassBA();

                if (a1.getA() != 1) return 1;
                if (b1.getB() != 2) return 2;
                if (a2.getA() != 10) return 3;
                if (b2.getB() != 20) return 4;
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_7_sub_interface_dynamic_dispatch_and_cast() {
    let code = r#"
        interface Reader { int32 read(); }
        interface AdvancedReader extends Reader { int32 reset(); }

        class StreamBuffer implements AdvancedReader {
            public int32 read() { return 42; }
            public int32 reset() { return 7; }
        }

        class Main {
            public static int32 main() {
                AdvancedReader advanced_reader = new StreamBuffer();
                Reader base_reader = advanced_reader;
                if (base_reader.read() != 42) return 1;
                if (advanced_reader.read() != 42) return 2;
                if (advanced_reader.reset() != 7) return 3;
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_8_class_inheritance_with_interface_implementation_override() {
    let code = r#"
        interface Action { int32 execute(); }

        class BaseTask implements Action {
            public int32 execute() { return 50; }
        }

        class DerivedTask extends BaseTask {
            public override int32 execute() { return 100; }
        }

        class Main {
            public static int32 main() {
                Action action_one = new BaseTask();
                Action action_two = new DerivedTask();
                if (action_one.execute() != 50) return 1;
                if (action_two.execute() != 100) return 2;
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_9_instanceof_check_for_interfaces() {
    let code = r#"
        interface Worker { int32 work(); }
        interface Sleeper { void sleep(); }

        class Human implements Worker {
            public int32 work() { return 1; }
        }

        class Main {
            public static int32 main() {
                Human h = new Human();
                Worker w = h;
                if (!(h instanceof Worker)) return 1;
                if (!(w instanceof Worker)) return 2;
                if (h instanceof Sleeper) return 3;
                if (w instanceof Sleeper) return 4;
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_interface_method_with_body() {
    // Error: E0100 (Syntax error / Interface method cannot have body)
    let code = r#"
        interface Reader {
            int32 read() { return 0; }
        }
    "#;
    assert_compile_error(code, "body");
}

#[test]
fn test_case_4_2_class_incompletely_implementing_interface() {
    // Error: E0206 (UnimplementedInterfaceMethod)
    let code = r#"
        interface Service {
            void start();
            void stop();
        }
        class IncompleteService implements Service {
            public void start() {}
        }
    "#;
    assert_compile_error(code, "E0206");
}

#[test]
fn test_case_4_3_interface_containing_state_fields() {
    // Error: E0126 (InterfaceField)
    let code = r#"
        interface BadInterface {
            int32 stateField;
        }
    "#;
    assert_compile_error(code, "E0126");
}

#[test]
fn test_case_4_4_interface_declared_inside_function_body() {
    // Error: E0116 (InterfaceInLocalScope)
    let code = r#"
        class Main {
            public static void test() {
                interface LocalInterface {}
            }
        }
    "#;
    assert_compile_error(code, "E0116");
}
