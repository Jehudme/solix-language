//! Suite: AliasStatement - Modules (TESTS.md)

use solix_runtime::Value;
use solix_tests::common::{assert_compile_error, compile_and_run, compile_and_run_multi};

#[test]
fn test_case_3_1_primitive_synonym() {
    let code = r#"
        alias Byte = uint8;

        class Main {
            public static int32 main() {
                Byte b = 255;
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_2_parameterized_generic_alias() {
    let code = r#"
        class Map<K, V> {
            public Map() {}
        }
        alias IntMap<V> = Map<int32, V>;

        class Main {
            public static int32 main() {
                IntMap<int32> map = new IntMap<int32>();
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_3_class_and_qualified_type_alias() {
    let sources = [
        (
            "helper.slx",
            r#"
            package my.pkg;

            public class Helper {
                public int32 val;
                public Helper(int32 v) { this.val = v; }
                public int32 getVal() { return this.val; }
            }
        "#,
        ),
        (
            "main.slx",
            r#"
            import my.pkg.Helper;
            alias MyHelper = my.pkg.Helper;

            class Main {
                public static int32 main() {
                    MyHelper h = new MyHelper(42);
                    return h.getVal() == 42 ? 0 : 1;
                }
            }
        "#,
        ),
    ];
    let (res, _) = compile_and_run_multi(&sources);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_4_alias_in_method_signature() {
    let code = r#"
        alias ID = int64;

        class User {
            public ID id;
            public User() { this.id = 42; }
            public ID getId() { return this.id; }
        }

        class Main {
            public static int32 main() {
                User u = new User();
                return u.getId() == 42 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_circular_alias_definition() {
    // Error: E0230 (CircularAlias)
    let code = r#"
        alias A = B;
        alias B = A;
    "#;
    assert_compile_error(code, "E0230");
}

#[test]
fn test_case_4_2_generic_parameter_arity_mismatch() {
    // Error: E0216 (GenericArityMismatch)
    let code = r#"
        class Map<K, V> {}
        alias Pair<K, V> = Map<K, V>;

        class Main {
            public static int32 main() {
                Pair<int32> bad;
                return 0;
            }
        }
    "#;
    assert_compile_error(code, "E0216");
}

#[test]
fn test_case_4_3_aliasing_undeclared_type() {
    // Error: E0231 (UnresolvedAliasTarget)
    let code = r#"
        alias Bad = NonExistentType;
    "#;
    assert_compile_error(code, "E0231");
}

#[test]
fn test_case_4_4_duplicate_alias_in_same_scope() {
    // Error: E0202 (DuplicateSymbol)
    let code = r#"
        alias Value = int32;
        alias Value = float64;
    "#;
    assert_compile_error(code, "E0202");
}
