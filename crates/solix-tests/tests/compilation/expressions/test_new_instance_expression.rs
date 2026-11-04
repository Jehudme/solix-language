//! NewInstanceExpression Conformance Tests (SPEC-0001 §14, TESTS.md Section 4.12).

#![forbid(unsafe_code)]

use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_3_1_instantiation_with_overloaded_constructor() {
    let code = r#"
        class Item {
            public int32 val;
            public Item() { this.val = 0; }
            public Item(int32 v) { this.val = v; }
        }

        int32 main() {
            Item i1 = new Item();
            Item i2 = new Item(42);
            if (i1.val == 0 && i2.val == 42) {
                return 0;
            }
            return 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_2_instantiating_class_with_in_class_field_initializers() {
    let code = r#"
        class Config {
            public int32 timeout = 3000;
            public bool enabled = true;
        }

        int32 main() {
            Config c = new Config();
            return (c.timeout == 3000 && c.enabled == true) ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_instantiating_abstract_class() {
    let code = r#"
        abstract class Base {}

        int32 main() {
            Base b = new Base();
            return 0;
        }
    "#;
    assert_compile_error(code, "abstract");
}

#[test]
fn test_case_4_2_calling_non_existent_constructor_overload() {
    let code = r#"
        class Box {
            public Box(int32 w) {}
        }

        void test() {
            Box b = new Box("bad");
        }
    "#;
    assert_compile_error(code, "constructor");
}
