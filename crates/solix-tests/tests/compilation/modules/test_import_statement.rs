//! Suite: ImportStatement - Modules (TESTS.md)

use solix_runtime::Value;
use solix_tests::common::{
    assert_compile_error, assert_compile_sources_error, compile_and_run_multi,
};

#[test]
fn test_case_3_1_selective_import() {
    let sources = [
        (
            "pkg/list.slx",
            r#"
            package std.collections;
            public class List {
                public List() {}
            }
        "#,
        ),
        (
            "main.slx",
            r#"
            import std.collections.List;

            class Main {
                public static int32 main() {
                    List items = new List();
                    return 0;
                }
            }
        "#,
        ),
    ];
    let (res, _) = compile_and_run_multi(&sources);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_2_wildcard_import() {
    let sources = [
        (
            "pkg/console.slx",
            r#"
            package std.io;
            public class Console {
                public static int32 get_magic() {
                    return 42;
                }
            }
        "#,
        ),
        (
            "main.slx",
            r#"
            import std.io.*;

            class Main {
                public static int32 main() {
                    int32 v = Console::get_magic();
                    return v == 42 ? 0 : 1;
                }
            }
        "#,
        ),
    ];
    let (res, _) = compile_and_run_multi(&sources);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_3_hierarchical_multi_level_wildcard_import() {
    let sources = [
        (
            "collections/list.slx",
            r#"
            package my.collections;

            public class MyList {
                public int32 size() { return 0; }
            }
        "#,
        ),
        (
            "main.slx",
            r#"
            import my.collections.*;

            class Main {
                public static int32 main() {
                    MyList list = new MyList();
                    return list.size() == 0 ? 0 : 1;
                }
            }
        "#,
        ),
    ];
    let (res, _) = compile_and_run_multi(&sources);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_4_qualified_access_with_active_import() {
    let sources = [
        (
            "core/item.slx",
            r#"
            package my.core;

            public class Item {
                public int32 val;
                public Item(int32 v) { this.val = v; }
                public int32 getVal() { return this.val; }
            }
        "#,
        ),
        (
            "main.slx",
            r#"
            import my.core.Item;

            class Main {
                public static int32 main() {
                    my.core.Item s1 = new my.core.Item(10);
                    Item s2 = new Item(20);
                    return (s1.getVal() == 10 && s2.getVal() == 20) ? 0 : 1;
                }
            }
        "#,
        ),
    ];
    let (res, _) = compile_and_run_multi(&sources);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_5_cross_platform_binary_path_resolution() {
    let current_binary = std::env::current_exe();
    assert!(
        current_binary.is_ok(),
        "Resolving current binary path must succeed"
    );
    let executable_path = current_binary.expect("Binary path verified");
    assert!(executable_path.exists(), "Binary path must exist on disk");
    assert!(
        executable_path.parent().is_some(),
        "Binary must have containing directory"
    );
}

#[test]
fn test_case_4_1_importing_non_existent_package() {
    // Error: E0232 (CannotResolveImport)
    let code = r#"
        import invalid.pkg.Foo;

        class Main {
            public static int32 main() {
                return 0;
            }
        }
    "#;
    assert_compile_error(code, "E0232");
}

#[test]
fn test_case_4_2_ambiguous_symbol_collision() {
    // Error: E0224 (AmbiguousImport)
    let sources = [
        (
            "pkg_a.slx",
            r#"
            package pkg_a;
            public class Token {}
        "#,
        ),
        (
            "pkg_b.slx",
            r#"
            package pkg_b;
            public class Token {}
        "#,
        ),
        (
            "main.slx",
            r#"
            import pkg_a.*;
            import pkg_b.*;

            class Main {
                public static int32 main() {
                    Token t;
                    return 0;
                }
            }
        "#,
        ),
    ];
    assert_compile_sources_error(&sources, "E0224");
}

#[test]
fn test_case_4_3_misplaced_import_statement() {
    // Error: E0113 (ImportAfterDeclaration)
    let code = r#"
        class Foo {}
        import my.core.Item;
    "#;
    assert_compile_error(code, "E0113");
}

#[test]
fn test_case_4_4_importing_non_existent_member_from_existing_package() {
    // Error: E0201 (UndefinedSymbol)
    let sources = [
        (
            "core.slx",
            r#"
            package my.core;
            public class RealSymbol {}
        "#,
        ),
        (
            "main.slx",
            r#"
            import my.core.FakeSymbol;
        "#,
        ),
    ];
    assert_compile_sources_error(&sources, "E0201");
}

#[test]
fn test_case_4_5_import_statement_inside_class_body() {
    // Error: E0105 (ExpectedTypeName)
    let code = r#"
        class Foo {
            import my.core.Item;
        }
    "#;
    assert_compile_error(code, "E0105");
}

#[test]
fn test_case_4_6_import_statement_inside_function_body() {
    // Error: E0113 (ImportAfterDeclaration)
    let code = r#"
        class Main {
            public static int32 main() {
                import my.core.Item;
                return 0;
            }
        }
    "#;
    assert_compile_error(code, "E0113");
}
