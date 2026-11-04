//! Suite: ExternDeclarations - Part V (TESTS.md)

#![forbid(unsafe_code)]

use solix_common::bytecode::extract_api_manifest;
use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
use solix_compilation::manifest_to_source;
use solix_tests::common::{assert_compile_error, compile_source};

#[test]
fn test_case_56_1_extern_function_prototype_compilation() {
    let code = r#"
        extern int32 external_add(int32 a, int32 b);

        class Main {
            public static int32 main() {
                int32 result = external_add(10, 20);
                return result;
            }
        }
    "#;
    let binary = compile_source(code);
    assert!(
        !binary
            .functions
            .iter()
            .any(|f| f.symbol.contains("external_add"))
    );
    assert!(binary.functions.iter().any(|f| f.symbol.contains("main")));
}

#[test]
fn test_case_56_2_extern_static_variable_compilation() {
    let code = r#"
        extern int32 GLOBAL_FLAG;

        class Main {
            public static int32 main() {
                return GLOBAL_FLAG;
            }
        }
    "#;
    let binary = compile_source(code);
    assert_eq!(binary.static_space_size, 0); // No local static slot allocated
    assert!(
        !binary
            .static_symbols_map
            .iter()
            .any(|(s, _)| s == "GLOBAL_FLAG")
    );
}

#[test]
fn test_case_56_3_extern_class_with_methods_and_fields() {
    let code = r#"
        extern class Calculator {
            public int32 base_val;
            public Calculator(int32 init);
            public int32 compute(int32 multiplier);
        }

        class Main {
            public static int32 main() {
                Calculator calc = new Calculator(5);
                return calc.compute(3);
            }
        }
    "#;
    let binary = compile_source(code);
    assert!(
        !binary
            .classes
            .iter()
            .any(|c| c.symbol.contains("Calculator"))
    );
    assert!(binary.classes.iter().any(|c| c.symbol.contains("Main")));
}

#[test]
fn test_case_56_4_api_manifest_extraction_and_source_synthesis() {
    let lib_code = r#"
        package math;

        public int32 add(int32 a, int32 b) {
            return a + b;
        }

        public class Vector2 {
            public float32 x;
            public float32 y;
            public Vector2(float32 x, float32 y) {
                this.x = x;
                this.y = y;
            }
            public float32 length_squared() {
                return this.x * this.x + this.y * this.y;
            }
        }
    "#;

    let options = CompilationOptions::new()
        .with_source("lib.slx", lib_code)
        .with_package_metadata("math", "1.2.0", "Solix Math Library");
    let mut context = CompilationContext::new(options);
    context.execute().expect("Emit failed");
    let artifacts = context.emission_artifacts.as_ref().unwrap();

    // 1. Extract JSON manifest
    let json_manifest =
        extract_api_manifest(&artifacts.program).expect("Manifest extraction failed");
    assert!(json_manifest.contains("\"package_name\": \"math\""));
    assert!(json_manifest.contains("\"package_version\": \"1.2.0\""));

    // 2. Synthesize interface source
    let (file_name, generated_source) =
        manifest_to_source(&json_manifest).expect("Source synthesis failed");
    assert_eq!(file_name, "math@1.2.0.slx");
    assert!(generated_source.contains("add(int32 a, int32 b);"));
    assert!(generated_source.contains("class Vector2"));

    // 3. Downstream consumer compiles against synthesized source
    let app_code = r#"
        class Main {
            public static int32 main() {
                int32 sum = add(5, 10);
                Vector2 vec = new Vector2(3.0, 4.0);
                return sum;
            }
        }
    "#;
    let consumer_options = CompilationOptions::new()
        .with_source(&file_name, &generated_source)
        .with_source("app.slx", app_code);
    let mut consumer_context = CompilationContext::new(consumer_options);
    consumer_context.execute().expect("Emit consumer failed");
    let consumer_artifacts = consumer_context.emission_artifacts.as_ref().unwrap();
    assert!(
        consumer_artifacts
            .program
            .functions
            .iter()
            .any(|f| f.symbol.contains("main"))
    );
    assert!(
        !consumer_artifacts
            .program
            .functions
            .iter()
            .any(|f| f.symbol.contains("add"))
    );
    assert!(
        !consumer_artifacts
            .program
            .functions
            .iter()
            .any(|f| f.symbol.contains("Vector2"))
    );
}

#[test]
fn test_case_56_5_negative_extern_function_with_body() {
    // Error: E0137 (ExternWithBody)
    let code = r#"
        extern int32 bad_extern(int32 x) {
            return x * 2;
        }
    "#;
    assert_compile_error(code, "E0137");
}

#[test]
fn test_case_56_5b_negative_extern_method_with_body() {
    // Error: E0137 (ExternWithBody)
    let code = r#"
        class Foo {
            extern int32 bad_method() {
                return 42;
            }
        }
    "#;
    assert_compile_error(code, "E0137");
}

#[test]
fn test_case_56_5c_negative_extern_constructor_with_body() {
    // Error: E0137 (ExternWithBody)
    let code = r#"
        class Foo {
            extern Foo() {
            }
        }
    "#;
    assert_compile_error(code, "E0137");
}

#[test]
fn test_case_56_6_negative_extern_variable_with_initializer() {
    // Error: E0154 (ExternWithInitializer)
    let code = r#"
        extern int32 initial_val = 100;
    "#;
    assert_compile_error(code, "E0154");
}

#[test]
fn test_case_56_6b_negative_extern_class_field_with_initializer() {
    // Error: E0154 (ExternWithInitializer)
    let code = r#"
        extern class Foo {
            public int32 x = 42;
        }
    "#;
    assert_compile_error(code, "E0154");
}

#[test]
fn test_case_56_7_negative_extern_inside_local_function() {
    // Error: E0155 (ExternInsideFunction)
    let code = r#"
        class Main {
            public static int32 main() {
                extern int32 local_extern;
                return 0;
            }
        }
    "#;
    assert_compile_error(code, "E0155");
}

#[test]
fn test_case_56_8_negative_extern_combined_with_native() {
    // Error: E0156 (InvalidExternModifier)
    let code = r#"
        extern native int32 conflicting_fn();
    "#;
    assert_compile_error(code, "E0156");
}

#[test]
fn test_case_56_9_negative_extern_combined_with_inline() {
    // Error: E0156 (InvalidExternModifier)
    let code = r#"
        extern inline int32 conflicting_inline();
    "#;
    assert_compile_error(code, "E0156");
}

#[test]
fn test_case_56_10_negative_extern_combined_with_abstract() {
    // Error: E0156 (InvalidExternModifier)
    let code = r#"
        class Base {
            extern abstract void conflicting_abstract();
        }
    "#;
    assert_compile_error(code, "E0156");
}
