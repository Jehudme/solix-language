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
                    my::core::Item s1 = new my::core::Item(10);
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
fn test_case_3_6_package_direct_symbol_import() {
    let pkg_code = r#"
        package test_pkg.io;

        public class Logger {
            public static int32 get_status() {
                return 0;
            }
        }
    "#;
    let main_code = r#"
        import test_pkg.io.Logger;

        class Main {
            public static int32 main() {
                return Logger::get_status();
            }
        }
    "#;
    let (execution_result, _) =
        compile_and_run_multi(&[("test_pkg/io.slx", pkg_code), ("main.slx", main_code)]);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_case_3_7_package_wildcard_import() {
    let pkg_code = r#"
        package test_pkg.io;

        public class Logger {
            public static int32 get_status() {
                return 0;
            }
        }
    "#;
    let main_code = r#"
        import test_pkg.io.*;

        class Main {
            public static int32 main() {
                return Logger::get_status();
            }
        }
    "#;
    let (execution_result, _) =
        compile_and_run_multi(&[("test_pkg/io.slx", pkg_code), ("main.slx", main_code)]);
    assert_eq!(execution_result, Value::I32(0));
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

#[test]
fn test_case_4_7_importing_symbol_from_subpackage_prefix_fails() {
    let pkg_code = r#"
        package test_pkg.io;
        public class Logger {}
    "#;
    let code = r#"
        import test_pkg.println;

        class Main {
            public static int32 main() {
                return 0;
            }
        }
    "#;
    assert_compile_sources_error(&[("pkg.slx", pkg_code), ("main.slx", code)], "E0232");
}

#[test]
fn test_case_4_8_wildcard_import_of_subpackage_prefix_fails() {
    let pkg_code = r#"
        package test_pkg.io;
        public class Logger {}
    "#;
    let code = r#"
        import test_pkg.*;

        class Main {
            public static int32 main() {
                return 0;
            }
        }
    "#;
    assert_compile_sources_error(&[("pkg.slx", pkg_code), ("main.slx", code)], "E0232");
}

#[test]
fn test_case_4_9_importing_unknown_member_from_package_fails() {
    let pkg_code = r#"
        package test_pkg.io;
        public class Logger {}
    "#;
    let code = r#"
        import test_pkg.io.nonexistent_symbol;

        class Main {
            public static int32 main() {
                return 0;
            }
        }
    "#;
    assert_compile_sources_error(&[("pkg.slx", pkg_code), ("main.slx", code)], "E0201");
}

#[test]
fn test_case_4_10_importing_intermediate_prefix_of_user_package_fails() {
    // Error: E0232 (CannotResolveImport) - 'my.core' is not a package when only 'my.core.sub' is declared
    let sources = [
        (
            "sub.slx",
            r#"
            package my.core.sub;
            public class DeepItem {}
        "#,
        ),
        (
            "main.slx",
            r#"
            import my.core.DeepItem;

            class Main {
                public static int32 main() {
                    return 0;
                }
            }
        "#,
        ),
    ];
    assert_compile_sources_error(&sources, "E0232");
}
