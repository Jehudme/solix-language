//! Suite: ClassDeclaration - Declarations (TESTS.md)

use solix_runtime::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_3_1_single_inheritance_and_dynamic_dispatch() {
    let code = r#"
        class Animal {
            public virtual int32 sound() {
                return 1;
            }
        }

        class Cat extends Animal {
            public override int32 sound() {
                return 2;
            }
        }

        class Main {
            public static int32 main() {
                Animal pet = new Cat();
                return pet.sound() == 2 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_2_multi_level_inheritance_chain() {
    let code = r#"
        class GrandParent {
            public int32 a;
        }
        class Parent extends GrandParent {
            public int32 b;
        }
        class Child extends Parent {
            public int32 c;
        }

        class Main {
            public static int32 main() {
                Child obj = new Child();
                obj.a = 1;
                obj.b = 2;
                obj.c = 3;
                return (obj.a + obj.b + obj.c) == 6 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_3_abstract_class_extension_and_implementation() {
    let code = r#"
        abstract class Shape {
            public abstract int32 getArea();
        }

        class Square extends Shape {
            public int32 side;
            public Square(int32 s) { this.side = s; }
            public override int32 getArea() { return this.side * this.side; }
        }

        class Main {
            public static int32 main() {
                Shape s = new Square(5);
                return s.getArea() == 25 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_4_class_access_modifiers() {
    let code = r#"
        public class ExportedService {
            public int32 serve() { return 100; }
        }

        internal class InternalHelper {
            public int32 help() { return 200; }
        }

        class Main {
            public static int32 main() {
                ExportedService es = new ExportedService();
                InternalHelper ih = new InternalHelper();
                return (es.serve() + ih.help()) == 300 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_6_nested_class_declaration_inside_class() {
    let code = r#"
        public class Outer {
            public class Inner {
                public int32 val;
                public Inner(int32 initial_val) {
                    this.val = initial_val;
                }
            }

            public static int32 run_check() {
                Inner inner_instance = new Inner(42);
                return inner_instance.val;
            }
        }

        public class Main {
            public static int32 main() {
                return Outer::run_check() == 42 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_circular_class_inheritance() {
    // Error: E0204 (CyclicInheritance)
    let code = r#"
        class A extends B {}
        class B extends A {}
    "#;
    assert_compile_error(code, "E0204");
}

#[test]
fn test_case_4_2_unimplemented_abstract_method() {
    // Error: E0206 (UnimplementedInterfaceMethod)
    let code = r#"
        abstract class Shape {
            public abstract float64 area();
        }

        class Circle extends Shape {}
    "#;
    assert_compile_error(code, "E0206");
}

#[test]
fn test_case_4_3_multiple_class_inheritance_disallowed() {
    // Error: E0104 (ExpectedClosingDelimiter or syntax error)
    let code = r#"
        class A {}
        class B {}
        class C extends A, B {}
    "#;
    assert_compile_error(code, "Expected");
}

#[test]
fn test_case_4_4_extending_non_class_or_primitive_type() {
    // Error: E0205 (InvalidBaseClass)
    let code = r#"
        class Invalid extends int32 {}
    "#;
    assert_compile_error(code, "E0205");
}

#[test]
fn test_case_4_5_instantiating_abstract_class_directly() {
    // Error: E0217 (CannotInstantiateAbstractType)
    let code = r#"
        abstract class AbstractBase {}

        class Main {
            public static int32 main() {
                AbstractBase a = new AbstractBase();
                return 0;
            }
        }
    "#;
    assert_compile_error(code, "E0217");
}

#[test]
fn test_case_4_6_class_declared_inside_function_body() {
    // Error: E0114 (ClassInLocalScope)
    let code = r#"
        class Main {
            public static int32 main() {
                class LocalClass {
                    int32 x;
                }
                return 0;
            }
        }
    "#;
    assert_compile_error(code, "E0114");
}

#[test]
fn test_case_4_7_class_declared_inside_control_flow_block() {
    // Error: E0114 (ClassInLocalScope)
    let code = r#"
        class Main {
            public static int32 main() {
                if (true) {
                    class BlockClass {}
                }
                return 0;
            }
        }
    "#;
    assert_compile_error(code, "E0114");
}
