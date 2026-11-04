//! Suite: PackageStatement - Modules (TESTS.md)

use solix_runtime::Value;
use solix_tests::common::{
    assert_compile_error, assert_compile_sources_error, compile_and_run, compile_and_run_multi,
};

#[test]
fn test_case_3_1_hierarchical_multi_level_package() {
    let code = r#"
        package std.collections.generic;

        public class CustomList {
            public CustomList() {}
        }

        class Main {
            public static int32 main() {
                CustomList list = new CustomList();
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_2_intra_package_unqualified_access() {
    let sources = [
        (
            "user.slx",
            r#"
            package app.models;

            public class User {
                public int32 id;
                public User() {
                    this.id = 42;
                }
            }
        "#,
        ),
        (
            "account.slx",
            r#"
            package app.models;

            public class Account {
                public User owner;
                public Account() {
                    this.owner = new User();
                }
            }

            class Main {
                public static int32 main() {
                    Account acc = new Account();
                    return acc.owner.id == 42 ? 0 : 1;
                }
            }
        "#,
        ),
    ];
    let (res, _) = compile_and_run_multi(&sources);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_3_package_isolation_across_unimported_namespaces() {
    let sources = [
        (
            "alpha.slx",
            r#"
            package alpha;
            public class Secret {
                public int32 code;
                public Secret() { this.code = 42; }
            }
        "#,
        ),
        (
            "beta.slx",
            r#"
            package beta;
            class Main {
                public static int32 main() {
                    Secret s = new Secret();
                    return 0;
                }
            }
        "#,
        ),
    ];
    // Error: E0201 (UndefinedSymbol)
    assert_compile_sources_error(&sources, "Secret");
}

#[test]
fn test_case_4_1_package_statement_not_first() {
    // Error: E0111 (PackageStatementNotFirst)
    let code = r#"
        class Dummy {}
        package app;
    "#;
    assert_compile_error(code, "E0111");
}

#[test]
fn test_case_4_2_duplicate_package_statement() {
    // Error: E0112 (DuplicatePackageStatement)
    let code = r#"
        package alpha;
        package beta;
    "#;
    assert_compile_error(code, "E0112");
}

#[test]
fn test_case_4_3_invalid_package_identifier_syntax() {
    // Error: E0102 (ExpectedIdentifier)
    let code = r#"
        package 123.invalid;
    "#;
    assert_compile_error(code, "E0102");
}

#[test]
fn test_case_4_4_package_statement_inside_class_body() {
    // Error: E0105 (ExpectedTypeName)
    let code = r#"
        class Foo {
            package invalid.placement;
        }
    "#;
    assert_compile_error(code, "E0105");
}

#[test]
fn test_case_4_5_package_statement_inside_function_body() {
    // Error: E0111 (PackageStatementNotFirst)
    let code = r#"
        class Main {
            public static int32 main() {
                package invalid.placement;
                return 0;
            }
        }
    "#;
    assert_compile_error(code, "E0111");
}
